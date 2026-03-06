use contracts::ServiceBoundaryV1;

#[must_use]
pub fn service_boundary() -> ServiceBoundaryV1 {
    ServiceBoundaryV1 {
        service_name: "resilience-service".to_owned(),
        domain: "resilience_continuity".to_owned(),
        approved_workflows: vec![
            "continuity_activation".to_owned(),
            "disaster_recovery_test".to_owned(),
        ],
        owned_aggregates: vec!["continuity_plan".to_owned(), "exercise_record".to_owned()],
    }
}
