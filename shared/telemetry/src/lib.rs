#[cfg(any(feature = "browser-tracing", feature = "native-tracing"))]
mod bootstrap;

use serde::{Deserialize, Serialize};
use std::fmt;
#[cfg(any(feature = "browser-tracing", feature = "native-tracing", test))]
use std::sync::atomic::{AtomicBool, Ordering};
use uuid::Uuid;

#[cfg(feature = "browser-tracing")]
pub use bootstrap::bootstrap_browser_tracing;
#[cfg(feature = "native-tracing")]
pub use bootstrap::bootstrap_native_tracing;

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
pub enum RuntimeTarget {
    Browser,
    DesktopTauri,
    Cargo,
}

impl RuntimeTarget {
    #[must_use]
    pub const fn as_str(&self) -> &str {
        match self {
            Self::Browser => "browser",
            Self::DesktopTauri => "desktop-tauri",
            Self::Cargo => "cargo",
        }
    }
}

impl fmt::Display for RuntimeTarget {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EnvironmentProfile {
    Development,
    Stage,
    Production,
    Test,
    Ci,
    Custom(String),
}

impl EnvironmentProfile {
    #[must_use]
    pub fn from_optional(value: Option<&str>, fallback: Self) -> Self {
        value.map_or(fallback, Self::parse)
    }

    #[must_use]
    pub fn parse(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "dev" | "development" | "local" => Self::Development,
            "stage" | "staging" => Self::Stage,
            "prod" | "production" => Self::Production,
            "test" | "testing" => Self::Test,
            "ci" => Self::Ci,
            other => Self::Custom(other.to_string()),
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Development => "dev",
            Self::Stage => "stage",
            Self::Production => "production",
            Self::Test => "test",
            Self::Ci => "ci",
            Self::Custom(value) => value.as_str(),
        }
    }

    #[must_use]
    pub fn default_log_filter(&self) -> &'static str {
        match self {
            Self::Development | Self::Test | Self::Ci => "info",
            Self::Stage | Self::Production | Self::Custom(_) => "warn",
        }
    }
}

impl fmt::Display for EnvironmentProfile {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TracingBootstrapConfig {
    pub component: String,
    pub runtime_target: RuntimeTarget,
    pub environment: EnvironmentProfile,
    pub default_filter: String,
    pub json_output: bool,
    pub enable_tokio_console: bool,
}

impl TracingBootstrapConfig {
    #[must_use]
    pub fn browser(component: impl Into<String>, environment: EnvironmentProfile) -> Self {
        let default_filter = environment.default_log_filter().to_string();
        Self {
            component: component.into(),
            runtime_target: RuntimeTarget::Browser,
            environment,
            default_filter,
            json_output: false,
            enable_tokio_console: false,
        }
    }

    #[must_use]
    pub fn native_json(
        component: impl Into<String>,
        runtime_target: RuntimeTarget,
        environment: EnvironmentProfile,
    ) -> Self {
        let default_filter = environment.default_log_filter().to_string();
        Self {
            component: component.into(),
            runtime_target,
            environment,
            default_filter,
            json_output: true,
            enable_tokio_console: false,
        }
    }

    #[must_use]
    pub fn with_default_filter(mut self, default_filter: impl Into<String>) -> Self {
        self.default_filter = default_filter.into();
        self
    }

    #[must_use]
    pub fn with_tokio_console(mut self, enable_tokio_console: bool) -> Self {
        self.enable_tokio_console = enable_tokio_console;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TracingBootstrapState {
    Installed,
    AlreadyInstalled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TracingBootstrapError {
    message: String,
}

impl TracingBootstrapError {
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for TracingBootstrapError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for TracingBootstrapError {}

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

#[cfg(any(feature = "browser-tracing", feature = "native-tracing", test))]
pub(crate) fn install_once(
    installed: &AtomicBool,
    install: impl FnOnce() -> Result<(), TracingBootstrapError>,
) -> Result<TracingBootstrapState, TracingBootstrapError> {
    if installed
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Ok(TracingBootstrapState::AlreadyInstalled);
    }

    match install() {
        Ok(()) => Ok(TracingBootstrapState::Installed),
        Err(error) => {
            installed.store(false, Ordering::SeqCst);
            Err(error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CorrelationId, DecisionRef, EnvironmentProfile, RuntimeTarget, TraceContext,
        TracingBootstrapConfig, TracingBootstrapState, install_once,
    };
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

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

    #[test]
    fn environment_profile_parses_common_aliases() {
        assert_eq!(
            EnvironmentProfile::parse("dev"),
            EnvironmentProfile::Development
        );
        assert_eq!(
            EnvironmentProfile::parse("stage"),
            EnvironmentProfile::Stage
        );
        assert_eq!(
            EnvironmentProfile::parse("prod"),
            EnvironmentProfile::Production
        );
        assert_eq!(EnvironmentProfile::parse("ci"), EnvironmentProfile::Ci);
        assert_eq!(
            EnvironmentProfile::parse("sandbox"),
            EnvironmentProfile::Custom("sandbox".to_string())
        );
    }

    #[test]
    fn tracing_bootstrap_config_builders_set_expected_defaults() {
        let browser =
            TracingBootstrapConfig::browser("ui/crates/site", EnvironmentProfile::Development);
        assert_eq!(browser.runtime_target, RuntimeTarget::Browser);
        assert!(!browser.json_output);
        assert_eq!(browser.default_filter, "info");

        let native = TracingBootstrapConfig::native_json(
            "ui/crates/desktop_tauri",
            RuntimeTarget::DesktopTauri,
            EnvironmentProfile::Production,
        )
        .with_tokio_console(true);
        assert_eq!(native.runtime_target, RuntimeTarget::DesktopTauri);
        assert!(native.json_output);
        assert_eq!(native.default_filter, "warn");
        assert!(native.enable_tokio_console);
    }

    #[test]
    fn install_once_only_runs_installer_once() {
        let flag = AtomicBool::new(false);
        let calls = AtomicUsize::new(0);

        let first = install_once(&flag, || {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })
        .expect("first install");
        let second = install_once(&flag, || {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })
        .expect("second install");

        assert_eq!(first, TracingBootstrapState::Installed);
        assert_eq!(second, TracingBootstrapState::AlreadyInstalled);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}
