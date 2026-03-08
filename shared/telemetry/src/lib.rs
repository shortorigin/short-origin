use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

macro_rules! trace_id_type {
    ($name:ident) => {
        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            #[must_use]
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(self.as_str())
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self::new(value)
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self::new(value)
            }
        }
    };
}

trace_id_type!(CorrelationId);
trace_id_type!(CausationId);
trace_id_type!(DecisionRef);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TraceContext {
    pub correlation_id: CorrelationId,
    pub causation_id: Option<CausationId>,
    pub decision_ref: Option<DecisionRef>,
}

impl TraceContext {
    #[must_use]
    pub fn new() -> Self {
        Self {
            correlation_id: CorrelationId::new(Uuid::new_v4().to_string()),
            causation_id: None,
            decision_ref: None,
        }
    }

    #[must_use]
    pub fn with_decision_ref(mut self, decision_ref: impl Into<DecisionRef>) -> Self {
        self.decision_ref = Some(decision_ref.into());
        self
    }

    #[must_use]
    pub fn with_causation_id(mut self, causation_id: impl Into<CausationId>) -> Self {
        self.causation_id = Some(causation_id.into());
        self
    }
}

impl Default for TraceContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{CorrelationId, DecisionRef, TraceContext};

    #[test]
    fn trace_context_uses_typed_ids() {
        let trace = TraceContext::new()
            .with_causation_id("cause-1")
            .with_decision_ref("decision-1");

        assert!(!trace.correlation_id.as_str().is_empty());
        assert_eq!(trace.causation_id, Some("cause-1".into()));
        assert_eq!(trace.decision_ref, Some(DecisionRef::new("decision-1")));
    }

    #[test]
    fn typed_ids_round_trip_through_serde() {
        let trace = TraceContext {
            correlation_id: CorrelationId::new("corr-1"),
            causation_id: None,
            decision_ref: Some(DecisionRef::new("decision-1")),
        };

        let serialized = serde_json::to_string(&trace).expect("serialize trace context");
        assert!(serialized.contains("\"corr-1\""));
        let deserialized: TraceContext =
            serde_json::from_str(&serialized).expect("deserialize trace context");
        assert_eq!(deserialized, trace);
    }
}
