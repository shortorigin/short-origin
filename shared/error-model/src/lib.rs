use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCategory {
    User,
    Runtime,
    Host,
    Validation,
    Io,
    Invariant,
}

impl ErrorCategory {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Runtime => "runtime",
            Self::Host => "host",
            Self::Validation => "validation",
            Self::Io => "io",
            Self::Invariant => "invariant",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ErrorVisibility {
    UserSafe,
    Internal,
}

impl ErrorVisibility {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UserSafe => "user_safe",
            Self::Internal => "internal",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ErrorMetadata {
    pub category: ErrorCategory,
    pub visibility: ErrorVisibility,
    pub code: String,
    pub operation: Option<String>,
}

impl ErrorMetadata {
    #[must_use]
    pub fn new(
        category: ErrorCategory,
        visibility: ErrorVisibility,
        code: impl Into<String>,
    ) -> Self {
        Self {
            category,
            visibility,
            code: code.into(),
            operation: None,
        }
    }

    #[must_use]
    pub fn with_operation(mut self, operation: impl Into<String>) -> Self {
        self.operation = Some(operation.into());
        self
    }
}

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
