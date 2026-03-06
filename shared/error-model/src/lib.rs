use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InstitutionalError {
    #[error("approval missing for action `{action}`")]
    ApprovalMissing { action: String },
    #[error("identity violation for actor `{actor}`")]
    IdentityViolation { actor: String },
    #[error("invariant violation `{invariant}`")]
    InvariantViolation { invariant: String },
    #[error("not found `{resource}`")]
    NotFound { resource: String },
    #[error("parse error in `{source_name}`: {details}")]
    ParseError {
        source_name: String,
        details: String,
    },
    #[error("policy denied: {reason}")]
    PolicyDenied { reason: String },
}

pub type InstitutionalResult<T> = Result<T, InstitutionalError>;

impl InstitutionalError {
    #[must_use]
    pub fn parse(source_name: impl Into<String>, details: impl Into<String>) -> Self {
        Self::ParseError {
            source_name: source_name.into(),
            details: details.into(),
        }
    }
}
