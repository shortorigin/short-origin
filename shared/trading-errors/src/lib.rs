use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TradingError {
    #[error("invalid input: {details}")]
    InvalidInput { details: String },
    #[error("serialization error: {details}")]
    Serialization { details: String },
    #[error("not found: {resource}")]
    NotFound { resource: String },
    #[error("conflict: {details}")]
    Conflict { details: String },
    #[error("limit breached: {details}")]
    LimitBreached { details: String },
    #[error("runtime policy violation: {details}")]
    RuntimePolicyViolation { details: String },
    #[error("guest trap during `{operation}`: {details}")]
    GuestTrap { operation: String, details: String },
    #[error("replay violation: {details}")]
    ReplayViolation { details: String },
    #[error("parse error in `{source_name}`: {details}")]
    Parse {
        source_name: String,
        details: String,
    },
}

pub type TradingResult<T> = Result<T, TradingError>;

impl From<serde_json::Error> for TradingError {
    fn from(error: serde_json::Error) -> Self {
        Self::Serialization {
            details: error.to_string(),
        }
    }
}
