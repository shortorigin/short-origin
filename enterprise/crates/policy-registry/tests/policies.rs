use policy_registry::{
    load_approval_thresholds, load_charter, load_decision_governance, load_invariants,
};

#[test]
fn policy_registry_loads_charter_thresholds_and_invariants() {
    let charter = load_charter().unwrap();
    let thresholds = load_approval_thresholds().unwrap();
    let invariants = load_invariants().unwrap();
    let decision_governance = load_decision_governance().unwrap();

    assert_eq!(charter.version, "v1");
    assert_eq!(thresholds.thresholds.len(), 4);
    assert!(
        invariants
            .invariants
            .iter()
            .any(|invariant| invariant == "workflows_is_the_only_cross_domain_mutation_path")
    );
    assert!(
        decision_governance
            .decision_classes
            .iter()
            .any(|value| value == "policy_constrained_action_selection")
    );
}
