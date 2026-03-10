/// Shared decision taxonomy re-exported from the contract layer.
pub use contracts::DecisionClassV1 as DecisionClass;
/// Shared lifecycle state re-exported from the contract layer.
pub use contracts::DecisionStateV1 as DecisionState;
/// Shared recommendation-status surface re-exported from the contract layer.
pub use contracts::RecommendationStatusV1 as RecommendationStatus;
/// Canonical decision identifier re-exported from the identity layer.
pub use identity::DecisionId;

/// Probability score normalized to the inclusive range `0.0..=1.0`.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct ProbabilityScore(f64);

impl ProbabilityScore {
    /// Creates a normalized probability score.
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(normalize_unit(value))
    }

    /// Returns the normalized floating-point value.
    #[must_use]
    pub fn value(self) -> f64 {
        self.0
    }
}

impl From<f64> for ProbabilityScore {
    fn from(value: f64) -> Self {
        Self::new(value)
    }
}

/// Risk score normalized to the inclusive range `0.0..=1.0`.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct RiskScore(f64);

impl RiskScore {
    /// Creates a normalized risk score.
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(normalize_unit(value))
    }

    /// Returns the normalized floating-point value.
    #[must_use]
    pub fn value(self) -> f64 {
        self.0
    }
}

impl From<f64> for RiskScore {
    fn from(value: f64) -> Self {
        Self::new(value)
    }
}

/// Utility score preserved as a finite floating-point value.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct UtilityScore(f64);

impl UtilityScore {
    /// Creates a finite utility score.
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(normalize_finite(value))
    }

    /// Returns the floating-point value.
    #[must_use]
    pub fn value(self) -> f64 {
        self.0
    }
}

impl From<f64> for UtilityScore {
    fn from(value: f64) -> Self {
        Self::new(value)
    }
}

/// Confidence score normalized to the inclusive range `0.0..=1.0`.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct ConfidenceScore(f64);

impl ConfidenceScore {
    /// Creates a normalized confidence score.
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(normalize_unit(value))
    }

    /// Returns the normalized floating-point value.
    #[must_use]
    pub fn value(self) -> f64 {
        self.0
    }
}

impl From<f64> for ConfidenceScore {
    fn from(value: f64) -> Self {
        Self::new(value)
    }
}

/// Captures a concrete constraint violation discovered during evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstraintViolation {
    /// Identifier of the violated constraint.
    pub constraint_id: String,
    /// Human-readable reason for the violation.
    pub message: String,
}

fn normalize_unit(value: f64) -> f64 {
    if value.is_finite() {
        round_stable(value.clamp(0.0, 1.0))
    } else {
        0.0
    }
}

fn normalize_finite(value: f64) -> f64 {
    if value.is_finite() {
        round_stable(value)
    } else {
        0.0
    }
}

fn round_stable(value: f64) -> f64 {
    const SCALE: f64 = 1_000_000_000_000.0;

    let rounded = (value * SCALE).round() / SCALE;
    if rounded == -0.0 {
        0.0
    } else {
        rounded
    }
}
