use contracts::{RiskRecordV1, ServiceBoundaryV1};

const SERVICE_NAME: &str = "risk-service";
const DOMAIN_NAME: &str = "compliance";
const APPROVED_WORKFLOWS: &[&str] = &["policy_exception", "continuity_activation"];
const OWNED_AGGREGATES: &[&str] = &["risk_record", "treatment_plan"];

#[derive(Debug, Default, Clone)]
struct InMemoryRiskRegister {
    records: Vec<RiskRecordV1>,
}

impl InMemoryRiskRegister {
    fn register(&mut self, record: RiskRecordV1) {
        self.records.push(record);
    }

    fn active_records(&self) -> &[RiskRecordV1] {
        &self.records
    }
}

#[derive(Debug, Default, Clone)]
pub struct RiskService {
    register: InMemoryRiskRegister,
}

impl RiskService {
    pub fn register(&mut self, record: RiskRecordV1) {
        self.register.register(record);
    }

    #[must_use]
    pub fn active_records(&self) -> &[RiskRecordV1] {
        self.register.active_records()
    }
}

#[must_use]
pub fn service_boundary() -> ServiceBoundaryV1 {
    ServiceBoundaryV1 {
        service_name: SERVICE_NAME.to_owned(),
        domain: DOMAIN_NAME.to_owned(),
        approved_workflows: APPROVED_WORKFLOWS
            .iter()
            .map(|value| (*value).to_owned())
            .collect(),
        owned_aggregates: OWNED_AGGREGATES
            .iter()
            .map(|value| (*value).to_owned())
            .collect(),
    }
}
