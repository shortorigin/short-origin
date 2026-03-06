use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TraceContext {
    pub correlation_id: String,
    pub causation_id: Option<String>,
    pub decision_ref: Option<String>,
}

impl TraceContext {
    #[must_use]
    pub fn new() -> Self {
        Self {
            correlation_id: Uuid::new_v4().to_string(),
            causation_id: None,
            decision_ref: None,
        }
    }

    #[must_use]
    pub fn with_decision_ref(mut self, decision_ref: impl Into<String>) -> Self {
        self.decision_ref = Some(decision_ref.into());
        self
    }
}

impl Default for TraceContext {
    fn default() -> Self {
        Self::new()
    }
}
