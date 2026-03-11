use control_catalog::load_control_catalog;

#[test]
fn control_catalog_contains_baseline_governance_controls() {
    let catalog = load_control_catalog().unwrap();

    assert_eq!(catalog.version, "v1");
    assert!(
        catalog
            .controls
            .iter()
            .any(|control| control.control_id == "CTRL-GOV-001")
    );
    assert!(
        catalog
            .controls
            .iter()
            .any(|control| control.checkpoint == "approval_verification")
    );
}
