use std::collections::BTreeMap;

use contracts::{AgentActionRequestV1, Classification, ImpactTier};
use error_model::{InstitutionalError, InstitutionalResult};
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentManifest {
    pub role: String,
    pub objective: String,
    pub classification_ceiling: String,
    pub owner_role: String,
    pub allowed_inputs: Vec<String>,
    pub allowed_workflows: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentAuthorization {
    pub role: String,
    pub requested_workflow: String,
    pub requires_human_approval: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentRegistry {
    manifests: BTreeMap<String, AgentManifest>,
}

#[derive(Debug, Deserialize)]
struct AgentMetadataToml {
    role: String,
    objective: String,
    classification_ceiling: String,
    owner_role: String,
}

#[derive(Debug, Deserialize)]
struct AllowedInputsToml {
    allowed_inputs: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct AllowedActionsToml {
    allowed_workflows: Vec<String>,
}

const ARCHITECT_METADATA: &str = include_str!("../../architect_operator/agent.toml");
const ARCHITECT_INPUTS: &str = include_str!("../../architect_operator/allowed_inputs.toml");
const ARCHITECT_ACTIONS: &str = include_str!("../../architect_operator/allowed_actions.toml");
const COMPLIANCE_METADATA: &str = include_str!("../../compliance_officer/agent.toml");
const COMPLIANCE_INPUTS: &str = include_str!("../../compliance_officer/allowed_inputs.toml");
const COMPLIANCE_ACTIONS: &str = include_str!("../../compliance_officer/allowed_actions.toml");
const FINANCE_METADATA: &str = include_str!("../../finance_analyst/agent.toml");
const FINANCE_INPUTS: &str = include_str!("../../finance_analyst/allowed_inputs.toml");
const FINANCE_ACTIONS: &str = include_str!("../../finance_analyst/allowed_actions.toml");
const LEGAL_METADATA: &str = include_str!("../../legal_advisor/agent.toml");
const LEGAL_INPUTS: &str = include_str!("../../legal_advisor/allowed_inputs.toml");
const LEGAL_ACTIONS: &str = include_str!("../../legal_advisor/allowed_actions.toml");
const STRATEGIST_METADATA: &str = include_str!("../../strategist/agent.toml");
const STRATEGIST_INPUTS: &str = include_str!("../../strategist/allowed_inputs.toml");
const STRATEGIST_ACTIONS: &str = include_str!("../../strategist/allowed_actions.toml");

impl AgentRegistry {
    pub fn load_default() -> InstitutionalResult<Self> {
        let manifests = [
            build_manifest(
                ARCHITECT_METADATA,
                ARCHITECT_INPUTS,
                ARCHITECT_ACTIONS,
                "agents/architect_operator",
            )?,
            build_manifest(
                COMPLIANCE_METADATA,
                COMPLIANCE_INPUTS,
                COMPLIANCE_ACTIONS,
                "agents/compliance_officer",
            )?,
            build_manifest(
                FINANCE_METADATA,
                FINANCE_INPUTS,
                FINANCE_ACTIONS,
                "agents/finance_analyst",
            )?,
            build_manifest(
                LEGAL_METADATA,
                LEGAL_INPUTS,
                LEGAL_ACTIONS,
                "agents/legal_advisor",
            )?,
            build_manifest(
                STRATEGIST_METADATA,
                STRATEGIST_INPUTS,
                STRATEGIST_ACTIONS,
                "agents/strategist",
            )?,
        ]
        .into_iter()
        .map(|manifest| (manifest.role.clone(), manifest))
        .collect();

        Ok(Self { manifests })
    }

    pub fn authorize_action(
        &self,
        role: &str,
        action: &AgentActionRequestV1,
    ) -> InstitutionalResult<AgentAuthorization> {
        let manifest = self
            .manifests
            .get(role)
            .ok_or_else(|| InstitutionalError::NotFound {
                resource: role.to_owned(),
            })?;

        if !manifest
            .allowed_workflows
            .iter()
            .any(|workflow| workflow == &action.requested_workflow)
        {
            return Err(InstitutionalError::PolicyDenied {
                reason: format!(
                    "agent `{role}` is not allowed to request workflow `{}`",
                    action.requested_workflow
                ),
            });
        }

        if classification_rank(action.classification)
            > classification_ceiling_rank(&manifest.classification_ceiling)
        {
            return Err(InstitutionalError::PolicyDenied {
                reason: format!(
                    "agent `{role}` exceeds classification ceiling `{}`",
                    manifest.classification_ceiling
                ),
            });
        }

        Ok(AgentAuthorization {
            role: role.to_owned(),
            requested_workflow: action.requested_workflow.clone(),
            requires_human_approval: action.impact_tier >= ImpactTier::Tier2,
        })
    }
}

fn build_manifest(
    metadata_source: &str,
    inputs_source: &str,
    actions_source: &str,
    source_name: &str,
) -> InstitutionalResult<AgentManifest> {
    let metadata = toml::from_str::<AgentMetadataToml>(metadata_source)
        .map_err(|error| InstitutionalError::parse(source_name, error.to_string()))?;
    let inputs = toml::from_str::<AllowedInputsToml>(inputs_source)
        .map_err(|error| InstitutionalError::parse(source_name, error.to_string()))?;
    let actions = toml::from_str::<AllowedActionsToml>(actions_source)
        .map_err(|error| InstitutionalError::parse(source_name, error.to_string()))?;

    Ok(AgentManifest {
        role: metadata.role,
        objective: metadata.objective,
        classification_ceiling: metadata.classification_ceiling,
        owner_role: metadata.owner_role,
        allowed_inputs: inputs.allowed_inputs,
        allowed_workflows: actions.allowed_workflows,
    })
}

fn classification_rank(classification: Classification) -> u8 {
    match classification {
        Classification::Public => 0,
        Classification::Internal => 1,
        Classification::Confidential => 2,
        Classification::Restricted => 3,
    }
}

fn classification_ceiling_rank(classification: &str) -> u8 {
    match classification {
        "internal" => 1,
        "confidential" => 2,
        "restricted" => 3,
        _ => 0,
    }
}
