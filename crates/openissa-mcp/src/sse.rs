use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::server::{JsonRpcRequest, McpServer};
use openissa_core::error::{OpenIssaError, Result};

type SessionsMap = Arc<Mutex<HashMap<String, mpsc::UnboundedSender<String>>>>;

pub struct SseServer {
    mcp: Arc<McpServer>,
    sessions: SessionsMap,
}

impl SseServer {
    pub fn new(mcp: Arc<McpServer>) -> Self {
        Self {
            mcp,
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Bind and serve HTTP SSE + JSON-RPC daemon on the specified address
    pub async fn run(&self, addr: SocketAddr) -> Result<()> {
        let listener = TcpListener::bind(addr).await.map_err(|e| {
            OpenIssaError::Internal(format!("Failed to bind TCP listener on {}: {}", addr, e))
        })?;

        info!("OpenISSA MCP SSE Server listening on http://{}", addr);
        info!("  - SSE Endpoint:   http://{}/sse", addr);
        info!("  - Message Post:   http://{}/message?sessionId=<id>", addr);
        info!("  - Health Status:  http://{}/health", addr);

        loop {
            match listener.accept().await {
                Ok((stream, peer_addr)) => {
                    let mcp = Arc::clone(&self.mcp);
                    let sessions = Arc::clone(&self.sessions);

                    tokio::spawn(async move {
                        if let Err(err) =
                            Self::handle_connection(stream, peer_addr, mcp, sessions).await
                        {
                            warn!("Connection error with {}: {}", peer_addr, err);
                        }
                    });
                }
                Err(e) => {
                    error!("TCP accept failed: {}", e);
                }
            }
        }
    }

    async fn handle_connection(
        mut stream: TcpStream,
        peer: SocketAddr,
        mcp: Arc<McpServer>,
        sessions: SessionsMap,
    ) -> Result<()> {
        let mut buffer = [0u8; 8192];
        let bytes_read = stream
            .read(&mut buffer)
            .await
            .map_err(|e| OpenIssaError::Internal(e.to_string()))?;

        if bytes_read == 0 {
            return Ok(());
        }

        let request_str = String::from_utf8_lossy(&buffer[..bytes_read]);
        let mut lines = request_str.lines();
        let request_line = lines.next().unwrap_or("");
        let mut parts = request_line.split_whitespace();
        let method = parts.next().unwrap_or("");
        let path = parts.next().unwrap_or("/");

        match (method, path) {
            // CORS preflight
            ("OPTIONS", _) => {
                let response = "HTTP/1.1 204 No Content\r\n\
                    Access-Control-Allow-Origin: *\r\n\
                    Access-Control-Allow-Methods: GET, POST, OPTIONS\r\n\
                    Access-Control-Allow-Headers: Content-Type, Authorization\r\n\
                    Content-Length: 0\r\n\r\n";
                stream
                    .write_all(response.as_bytes())
                    .await
                    .map_err(|e| OpenIssaError::Internal(e.to_string()))?;
            }

            // Health check
            ("GET", "/health") | ("GET", "/") => {
                let body = format!(
                    "{{\"status\":\"ok\",\"server\":\"openissa-mcp\",\"version\":\"{}\"}}",
                    env!("CARGO_PKG_VERSION")
                );
                let response = format!(
                    "HTTP/1.1 200 OK\r\n\
                    Content-Type: application/json\r\n\
                    Access-Control-Allow-Origin: *\r\n\
                    Content-Length: {}\r\n\r\n{}",
                    body.len(),
                    body
                );
                stream
                    .write_all(response.as_bytes())
                    .await
                    .map_err(|e| OpenIssaError::Internal(e.to_string()))?;
            }

            // SSE stream subscription
            ("GET", p) if p.starts_with("/sse") => {
                let session_id = Uuid::new_v4().to_string();
                let (tx, mut rx) = mpsc::unbounded_channel::<String>();

                {
                    let mut lock = sessions.lock().await;
                    lock.insert(session_id.clone(), tx);
                }

                info!(
                    "SSE client connected from {} (Session: {})",
                    peer, session_id
                );

                // Initial SSE header
                let header = "HTTP/1.1 200 OK\r\n\
                    Content-Type: text/event-stream\r\n\
                    Cache-Control: no-cache\r\n\
                    Connection: keep-alive\r\n\
                    Access-Control-Allow-Origin: *\r\n\r\n";
                stream
                    .write_all(header.as_bytes())
                    .await
                    .map_err(|e| OpenIssaError::Internal(e.to_string()))?;

                // Emit initial endpoint event per MCP spec
                let endpoint_msg = format!(
                    "event: endpoint\r\ndata: /message?sessionId={}\r\n\r\n",
                    session_id
                );
                stream
                    .write_all(endpoint_msg.as_bytes())
                    .await
                    .map_err(|e| OpenIssaError::Internal(e.to_string()))?;

                // Stream loop
                while let Some(msg) = rx.recv().await {
                    let sse_event = format!("event: message\r\ndata: {}\r\n\r\n", msg);
                    if stream.write_all(sse_event.as_bytes()).await.is_err() {
                        break;
                    }
                    let _ = stream.flush().await;
                }

                // Cleanup session
                let mut lock = sessions.lock().await;
                lock.remove(&session_id);
                info!("SSE client disconnected (Session: {})", session_id);
            }

            // POST JSON-RPC message
            ("POST", p) if p.starts_with("/message") => {
                // Extract session_id from query string if present
                let session_id = if let Some(idx) = p.find("sessionId=") {
                    let rest = &p[idx + 10..];
                    let end = rest.find('&').unwrap_or(rest.len());
                    Some(&rest[..end])
                } else {
                    None
                };

                // Extract body after \r\n\r\n
                let body = if let Some(idx) = request_str.find("\r\n\r\n") {
                    &request_str[idx + 4..]
                } else {
                    ""
                };

                let response_body = if let Ok(rpc_req) =
                    serde_json::from_str::<JsonRpcRequest>(body)
                {
                    if let Some(rpc_res) = mcp.handle_request(rpc_req).await {
                        let serialized = serde_json::to_string(&rpc_res).unwrap_or_default();

                        // If a matching SSE session is open, broadcast to it
                        if let Some(sid) = session_id {
                            let lock = sessions.lock().await;
                            if let Some(tx) = lock.get(sid) {
                                let _ = tx.send(serialized.clone());
                            }
                        }

                        serialized
                    } else {
                        "{\"jsonrpc\":\"2.0\",\"result\":null}".to_string()
                    }
                } else {
                    "{\"jsonrpc\":\"2.0\",\"error\":{\"code\":-32700,\"message\":\"Parse error\"}}"
                        .to_string()
                };

                let http_response = format!(
                    "HTTP/1.1 200 OK\r\n\
                    Content-Type: application/json\r\n\
                    Access-Control-Allow-Origin: *\r\n\
                    Content-Length: {}\r\n\r\n{}",
                    response_body.len(),
                    response_body
                );
                stream
                    .write_all(http_response.as_bytes())
                    .await
                    .map_err(|e| OpenIssaError::Internal(e.to_string()))?;
            }

            _ => {
                let not_found = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";
                stream
                    .write_all(not_found.as_bytes())
                    .await
                    .map_err(|e| OpenIssaError::Internal(e.to_string()))?;
            }
        }

        Ok(())
    }
}
