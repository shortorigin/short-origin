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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeTarget {
    Desktop,
    Wasm,
}

impl RuntimeTarget {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Desktop => "desktop",
            Self::Wasm => "wasm",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EnvironmentProfile {
    Development,
    Production,
}

impl EnvironmentProfile {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Development => "development",
            Self::Production => "production",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UiLogEvent {
    pub timestamp: String,
    pub level: String,
    pub target: String,
    pub event: String,
    pub operation: String,
    pub component: String,
    pub runtime_target: RuntimeTarget,
    pub environment: EnvironmentProfile,
    pub trace: Option<TraceContext>,
    pub session_id: Option<String>,
    pub window_id: Option<String>,
    pub app_id: Option<String>,
    pub action: Option<String>,
    pub effect: Option<String>,
    pub host_strategy: Option<String>,
    pub capability: Option<String>,
    pub schema_version: Option<u32>,
    pub error_category: Option<String>,
    pub error_code: Option<String>,
    pub error_visibility: Option<String>,
}
