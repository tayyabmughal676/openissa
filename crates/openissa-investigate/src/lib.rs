//! OpenISSA Investigate: Web Lab experiment runner with SSRF guardrails.

pub mod firewall;

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::{Client, Method};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub use firewall::NetworkFirewall;
use openissa_core::error::{OpenIssaError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointTestSpec {
    pub url: String,
    pub method: String,
    pub headers: Option<HashMap<String, String>>,
    pub params: Option<HashMap<String, String>>,
    pub body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointTestResult {
    pub url: String,
    pub status: u16,
    pub response_headers: HashMap<String, String>,
    pub body: String,
    pub duration_ms: u64,
}

pub struct EndpointTester {
    client: Client,
}

impl EndpointTester {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .user_agent("OpenISSA-WebLab/0.1 (+https://github.com/tayyabmughal676/openissa)")
                .redirect(reqwest::redirect::Policy::limited(3))
                .build()
                .unwrap_or_default(),
        }
    }

    /// Execute a controlled HTTP experiment against an API endpoint with SSRF verification.
    pub async fn execute_test(&self, spec: EndpointTestSpec) -> Result<EndpointTestResult> {
        // Validate URL against SSRF firewall
        let validated_url = NetworkFirewall::validate_url(&spec.url)?;

        let method = match spec.method.to_uppercase().as_str() {
            "GET" => Method::GET,
            "POST" => Method::POST,
            "PUT" => Method::PUT,
            "PATCH" => Method::PATCH,
            "DELETE" => Method::DELETE,
            "HEAD" => Method::HEAD,
            _ => {
                return Err(OpenIssaError::Internal(format!(
                    "Unsupported HTTP method: {}",
                    spec.method
                )))
            }
        };

        let mut req = self.client.request(method, validated_url.as_str());

        if let Some(params) = &spec.params {
            req = req.query(params);
        }

        if let Some(headers) = &spec.headers {
            let mut header_map = HeaderMap::new();
            for (k, v) in headers {
                if let (Ok(name), Ok(val)) = (
                    HeaderName::from_bytes(k.as_bytes()),
                    HeaderValue::from_str(v),
                ) {
                    header_map.insert(name, val);
                }
            }
            req = req.headers(header_map);
        }

        if let Some(body) = &spec.body {
            req = req.body(body.clone());
        }

        let start = Instant::now();
        let resp = req.send().await.map_err(|e| OpenIssaError::FetchError {
            url: spec.url.clone(),
            message: e.to_string(),
        })?;

        let status = resp.status().as_u16();
        let mut response_headers = HashMap::new();
        for (k, v) in resp.headers() {
            if let Ok(val) = v.to_str() {
                response_headers.insert(k.to_string(), val.to_string());
            }
        }

        let body = resp.text().await.map_err(|e| OpenIssaError::FetchError {
            url: spec.url.clone(),
            message: e.to_string(),
        })?;

        Ok(EndpointTestResult {
            url: spec.url,
            status,
            response_headers,
            body,
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }
}

impl Default for EndpointTester {
    fn default() -> Self {
        Self::new()
    }
}
