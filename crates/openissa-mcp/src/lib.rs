//! OpenISSA MCP: Model Context Protocol server exposing web research and investigation tools.

pub mod orchestrator;
pub mod server;
pub mod sse;

pub use orchestrator::{ResearchOrchestrator, ResearchResult};
pub use server::{JsonRpcRequest, JsonRpcResponse, McpServer};
pub use sse::SseServer;
