use contracts::ServiceBoundaryV1;

#[must_use]
pub fn service_boundary() -> ServiceBoundaryV1 {
    ServiceBoundaryV1 {
        service_name: "security-service".to_owned(),
        domain: "security".to_owned(),
        approved_workflows: vec!["access_review".to_owned(), "incident_response".to_owned()],
        owned_aggregates: vec!["access_decision".to_owned(), "security_incident".to_owned()],
    }
}
