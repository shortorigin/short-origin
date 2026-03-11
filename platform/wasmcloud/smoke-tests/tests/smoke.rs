use lattice_config::{
    LatticeConfigV1, RolloutTargetV1, finance_service_component_binding,
    treasury_disbursement_component_binding,
};
use sdk_rs::{
    InstitutionalPlatformClientV1, InstitutionalPlatformRuntimeClient,
    LocalHarnessPlatformTransport, ReleasedUiAppV1,
};

#[tokio::test]
async fn lattice_config_and_sdk_smoke_round_trip_component_descriptors() {
    let lattice = LatticeConfigV1 {
        lattice_name: "institutional-lattice".to_string(),
        rollout: RolloutTargetV1 {
            environment: "prod".to_string(),
            namespace: "runtime".to_string(),
            policy_group: "institutional-default".to_string(),
        },
        components: vec![
            finance_service_component_binding(),
            treasury_disbursement_component_binding(),
        ],
    };

    let manifest = InstitutionalPlatformClientV1 {
        client_name: "control-center".to_string(),
        supported_services: vec![finance_service::service_boundary()],
        supported_workflows: vec![treasury_disbursement::workflow_boundary()],
        lattice_config: Some(lattice.clone()),
    };
    let dashboard = manifest.dashboard_snapshot(
        vec![
            ReleasedUiAppV1 {
                app_id: "system.control-center".to_string(),
                display_name: "Control Center".to_string(),
                desktop_enabled: true,
            },
            ReleasedUiAppV1 {
                app_id: "system.terminal".to_string(),
                display_name: "Terminal".to_string(),
                desktop_enabled: true,
            },
            ReleasedUiAppV1 {
                app_id: "system.settings".to_string(),
                display_name: "System Settings".to_string(),
                desktop_enabled: true,
            },
        ],
        true,
    );
    let transport = LocalHarnessPlatformTransport::new(dashboard.clone(), Vec::new());
    let client = InstitutionalPlatformRuntimeClient::new(manifest, transport);

    let queried = client.query_dashboard().await.expect("dashboard");
    assert_eq!(queried, dashboard);
    assert_eq!(queried.lattice.expect("lattice"), lattice);
}
