use contracts::ServiceBoundaryV1;

#[must_use]
pub fn service_boundary() -> ServiceBoundaryV1 {
    ServiceBoundaryV1 {
        service_name: "knowledge-service".to_owned(),
        domain: "data_knowledge".to_owned(),
        approved_workflows: vec![
            "knowledge_publication".to_owned(),
            "record_retention".to_owned(),
        ],
        owned_aggregates: vec!["knowledge_record".to_owned(), "retention_policy".to_owned()],
    }
}
