use error_model::{InstitutionalError, InstitutionalResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InstitutionalCharter {
    pub version: String,
    pub mission: String,
    pub sovereign_objective: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApprovalThreshold {
    pub tier: String,
    pub minimum_human_approvals: usize,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApprovalThresholdCatalog {
    pub version: String,
    pub thresholds: Vec<ApprovalThreshold>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InstitutionalInvariantCatalog {
    pub version: String,
    pub invariants: Vec<String>,
}

const CHARTER: &str = include_str!("../../../policies/institutional_charter.toml");
const APPROVAL_THRESHOLDS: &str = include_str!("../../../policies/approval_thresholds.toml");
const INVARIANTS: &str = include_str!("../../../policies/invariants.toml");

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
