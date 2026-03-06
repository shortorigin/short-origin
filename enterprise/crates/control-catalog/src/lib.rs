use error_model::{InstitutionalError, InstitutionalResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ControlDefinition {
    pub control_id: String,
    pub objective: String,
    pub owner_role: String,
    pub checkpoint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ControlCatalogV1 {
    pub version: String,
    pub controls: Vec<ControlDefinition>,
}

const CONTROL_CATALOG: &str = include_str!("../../../policies/control_catalog.toml");

pub fn load_control_catalog() -> InstitutionalResult<ControlCatalogV1> {
    toml::from_str(CONTROL_CATALOG).map_err(|error| {
        InstitutionalError::parse(
            "enterprise/policies/control_catalog.toml",
            error.to_string(),
        )
    })
}
