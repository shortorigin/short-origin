use contracts::ServiceBoundaryV1;

#[must_use]
pub fn service_boundary() -> ServiceBoundaryV1 {
    ServiceBoundaryV1 {
        service_name: "engineering-service".to_owned(),
        domain: "engineering_platform".to_owned(),
        approved_workflows: vec!["release_approval".to_owned()],
        owned_aggregates: vec!["release_window".to_owned(), "build_provenance".to_owned()],
    }
}
