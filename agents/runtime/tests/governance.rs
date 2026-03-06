use agent_runtime::AgentRegistry;
use contracts::{AgentActionRequestV1, Classification, ImpactTier};
use identity::{ActorRef, InstitutionalRole};

fn architect_action(workflow: &str) -> AgentActionRequestV1 {
    AgentActionRequestV1 {
        action_id: format!("action::{workflow}"),
        actor_ref: ActorRef("agent.architect_operator".to_owned()),
        objective: "Propose or execute governed change".to_owned(),
        requested_workflow: workflow.to_owned(),
        impact_tier: ImpactTier::Tier2,
        classification: Classification::Restricted,
        required_approver_roles: vec![
            InstitutionalRole::Cto,
            InstitutionalRole::ChiefComplianceOfficer,
        ],
        policy_refs: vec!["governance.architecture.change.v1".to_owned()],
    }
}

#[test]
fn agent_runtime_requires_human_approval_for_high_impact_actions() {
    let registry = AgentRegistry::load_default().unwrap();
    let authorization = registry
        .authorize_action("architect_operator", &architect_action("strategy_review"))
        .unwrap();

    assert!(authorization.requires_human_approval);
}

#[test]
fn agent_runtime_denies_workflows_outside_allowlist() {
    let registry = AgentRegistry::load_default().unwrap();
    let result =
        registry.authorize_action("legal_advisor", &architect_action("environment_change"));

    assert!(matches!(
        result,
        Err(error_model::InstitutionalError::PolicyDenied { .. })
    ));
}
