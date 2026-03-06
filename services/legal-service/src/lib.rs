use contracts::ServiceBoundaryV1;

#[must_use]
pub fn service_boundary() -> ServiceBoundaryV1 {
    ServiceBoundaryV1 {
        service_name: "legal-service".to_owned(),
        domain: "legal".to_owned(),
        approved_workflows: vec!["contract_lifecycle".to_owned()],
        owned_aggregates: vec!["legal_agreement".to_owned()],
    }
}
