use contracts::ServiceBoundaryV1;

#[must_use]
pub fn service_boundary() -> ServiceBoundaryV1 {
    ServiceBoundaryV1 {
        service_name: "operations-service".to_owned(),
        domain: "operations".to_owned(),
        approved_workflows: vec![
            "incident_response".to_owned(),
            "change_management".to_owned(),
        ],
        owned_aggregates: vec!["incident_record".to_owned(), "runbook".to_owned()],
    }
}
