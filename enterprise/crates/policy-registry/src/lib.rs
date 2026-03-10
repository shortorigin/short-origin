use error_model::{InstitutionalError, InstitutionalResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InstitutionalCharter {
    pub version: String,
    pub mission: String,
    pub sovereign_objective: String,
}

/// Describes one approval threshold tier in the governance catalog.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApprovalThreshold {
    pub tier: String,
    pub minimum_human_approvals: usize,
    pub description: String,
}

/// Aggregates the approval-threshold catalog for policy evaluation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApprovalThresholdCatalog {
    pub version: String,
    pub thresholds: Vec<ApprovalThreshold>,
}

/// Collects the invariant statements that remain authoritative across the repo.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InstitutionalInvariantCatalog {
    pub version: String,
    pub invariants: Vec<String>,
}

/// Defines the governed vocabulary available to the decision architecture.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DecisionGovernanceCatalog {
    /// Version of the decision-governance catalog.
    pub version: String,
    /// Allowed decision classes.
    pub decision_classes: Vec<String>,
    /// Allowed risk tiers.
    pub risk_tiers: Vec<String>,
    /// Allowed approval requirements.
    pub approval_requirements: Vec<String>,
    /// Allowed reversibility classes.
    pub reversibility_classes: Vec<String>,
}

const CHARTER: &str = include_str!("../../../policies/institutional_charter.toml");
const APPROVAL_THRESHOLDS: &str = include_str!("../../../policies/approval_thresholds.toml");
const INVARIANTS: &str = include_str!("../../../policies/invariants.toml");
const DECISION_GOVERNANCE: &str = include_str!("../../../policies/decision_governance.toml");

pub fn load_charter() -> InstitutionalResult<InstitutionalCharter> {
    toml::from_str(CHARTER).map_err(|error| {
        InstitutionalError::parse(
            "enterprise/policies/institutional_charter.toml",
            error.to_string(),
        )
    })
}

pub fn load_approval_thresholds() -> InstitutionalResult<ApprovalThresholdCatalog> {
    toml::from_str(APPROVAL_THRESHOLDS).map_err(|error| {
        InstitutionalError::parse(
            "enterprise/policies/approval_thresholds.toml",
            error.to_string(),
        )
    })
}

pub fn load_invariants() -> InstitutionalResult<InstitutionalInvariantCatalog> {
    toml::from_str(INVARIANTS).map_err(|error| {
        InstitutionalError::parse("enterprise/policies/invariants.toml", error.to_string())
    })
}

/// Loads the decision-governance vocabulary catalog.
pub fn load_decision_governance() -> InstitutionalResult<DecisionGovernanceCatalog> {
    toml::from_str(DECISION_GOVERNANCE).map_err(|error| {
        InstitutionalError::parse(
            "enterprise/policies/decision_governance.toml",
            error.to_string(),
        )
    })
}
