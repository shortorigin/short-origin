use contracts::ServiceBoundaryV1;

#[must_use]
pub fn service_boundary() -> ServiceBoundaryV1 {
    ServiceBoundaryV1 {
        service_name: "procurement-service".to_owned(),
        domain: "procurement_vendor".to_owned(),
        approved_workflows: vec!["procurement".to_owned(), "vendor_onboarding".to_owned()],
        owned_aggregates: vec!["vendor_record".to_owned(), "spend_approval".to_owned()],
    }
}
