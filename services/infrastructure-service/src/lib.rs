use contracts::ServiceBoundaryV1;

#[must_use]
pub fn service_boundary() -> ServiceBoundaryV1 {
    ServiceBoundaryV1 {
        service_name: "infrastructure-service".to_owned(),
        domain: "infrastructure_it".to_owned(),
        approved_workflows: vec!["environment_change".to_owned(), "access_review".to_owned()],
        owned_aggregates: vec!["environment_change".to_owned(), "access_grant".to_owned()],
    }
}
