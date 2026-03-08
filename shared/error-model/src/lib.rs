use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ParseErrorContext {
    pub source_name: String,
    pub parser: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExternalErrorContext {
    pub system: String,
    pub operation: Option<String>,
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
    #[error("parse error in `{}`: {details}", .context.source_name)]
    ParseError {
        context: ParseErrorContext,
        details: String,
    },
    #[error(
        "external failure in `{}`{}: {details}",
        .context.system,
        .context
            .operation
            .as_ref()
            .map(|operation| format!(" during `{operation}`"))
            .unwrap_or_default()
    )]
    ExternalFailure {
        context: ExternalErrorContext,
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
            context: ParseErrorContext {
                source_name: source_name.into(),
                parser: None,
            },
            details: details.into(),
        }
    }

    #[must_use]
    pub fn parse_with_parser(
        source_name: impl Into<String>,
        parser: impl Into<String>,
        details: impl Into<String>,
    ) -> Self {
        Self::ParseError {
            context: ParseErrorContext {
                source_name: source_name.into(),
                parser: Some(parser.into()),
            },
            details: details.into(),
        }
    }

    #[must_use]
    pub fn external(
        system: impl Into<String>,
        operation: impl Into<Option<String>>,
        details: impl Into<String>,
    ) -> Self {
        Self::ExternalFailure {
            context: ExternalErrorContext {
                system: system.into(),
                operation: operation.into(),
            },
            details: details.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::InstitutionalError;

    #[test]
    fn parse_helper_preserves_structured_context() {
        let error = InstitutionalError::parse_with_parser("config.toml", "toml", "bad key");
        let InstitutionalError::ParseError { context, details } = error else {
            panic!("expected parse error");
        };
        assert_eq!(context.source_name, "config.toml");
        assert_eq!(context.parser.as_deref(), Some("toml"));
        assert_eq!(details, "bad key");
    }

    #[test]
    fn external_helper_preserves_operation() {
        let error = InstitutionalError::external(
            "surrealdb",
            Some("select workflow_execution".to_string()),
            "record missing",
        );
        let InstitutionalError::ExternalFailure { context, details } = error else {
            panic!("expected external failure");
        };
        assert_eq!(context.system, "surrealdb");
        assert_eq!(
            context.operation.as_deref(),
            Some("select workflow_execution")
        );
        assert_eq!(details, "record missing");
    }
}
