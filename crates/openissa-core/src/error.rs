use thiserror::Error;

/// Core domain errors for OpenISSA operations.
#[derive(Error, Debug)]
pub enum OpenIssaError {
    #[error("Budget exhausted: {reason}")]
    BudgetExhausted { reason: String },

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("SSRF violation: target IP {ip} is in a blocked subnet")]
    SsrfBlocked { ip: String },

    #[error("Fetch error for {url}: {message}")]
    FetchError { url: String, message: String },

    #[error("Decompression bomb detected: expansion ratio exceeds limit")]
    DecompressionBomb,

    #[error("Payload too large: {size} bytes exceeds maximum allowed {max}")]
    PayloadTooLarge { size: usize, max: usize },

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Protocol error: {0}")]
    ProtocolError(String),

    #[error("Human verification required for {url}")]
    HumanVerificationRequired { url: String },

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, OpenIssaError>;
