use contracts::{RiskRecordV1, ServiceBoundaryV1};

#[derive(Debug, Default, Clone)]
pub struct RiskService {
    records: Vec<RiskRecordV1>,
}

impl RiskService {
    pub fn register(&mut self, record: RiskRecordV1) {
        self.records.push(record);
    }

    #[must_use]
    pub fn active_records(&self) -> &[RiskRecordV1] {
        &self.records
    }
}

#[must_use]
pub fn service_boundary() -> ServiceBoundaryV1 {
    ServiceBoundaryV1 {
        service_name: "risk-service".to_owned(),
        domain: "compliance".to_owned(),
        approved_workflows: vec![
            "policy_exception".to_owned(),
            "continuity_activation".to_owned(),
        ],
        owned_aggregates: vec!["risk_record".to_owned(), "treatment_plan".to_owned()],
    }
}
