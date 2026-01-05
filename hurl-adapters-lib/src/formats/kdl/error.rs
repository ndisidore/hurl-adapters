use thiserror::Error;

/// Errors that can occur during KDL to Hurl translation.
#[derive(Debug, Error)]
pub enum TranslationError {
    #[error("Missing required field '{field}' in node '{node}'")]
    MissingRequiredField { node: String, field: String },

    #[error("Invalid HTTP method: '{0}'")]
    InvalidMethod(String),

    #[error("Invalid URL: '{0}'")]
    InvalidUrl(String),

    #[error("Invalid predicate: '{0}'")]
    InvalidPredicate(String),

    #[error("Invalid body: {reason}")]
    InvalidBody { reason: String },

    #[error("Invalid hex string: {reason}")]
    InvalidHex { reason: String },

    #[error("Duplicate step name: '{0}'")]
    DuplicateStepName(String),

    #[error("Unknown variable reference '{{{{variable}}}}' - step '{step}' not found or has no such capture")]
    UnknownReference { variable: String, step: String },

    #[error("Invalid KDL structure: {0}")]
    InvalidStructure(String),

    #[error("KDL parse error: {0}")]
    KdlError(#[from] kdl::KdlError),
}

pub type Result<T> = std::result::Result<T, TranslationError>;
