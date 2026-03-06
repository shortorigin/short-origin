use contracts::ServiceBoundaryV1;

#[must_use]
pub fn service_boundary() -> ServiceBoundaryV1 {
    ServiceBoundaryV1 {
        service_name: "revenue-service".to_owned(),
        domain: "sales_revenue".to_owned(),
        approved_workflows: vec![
            "revenue_approval".to_owned(),
            "contract_lifecycle".to_owned(),
        ],
        owned_aggregates: vec!["pricing_decision".to_owned(), "revenue_record".to_owned()],
    }
}
