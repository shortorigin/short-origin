use contracts::{ServiceBoundaryV1, TreasuryDisbursementRecordedV1, TreasuryDisbursementRequestV1};
use enforcement::ApprovedMutationContext;
use error_model::InstitutionalResult;
use identity::{ServiceId, WorkflowId};

const SERVICE_NAME: &str = "finance-service";
const DOMAIN_NAME: &str = "finance_treasury";
const APPROVED_WORKFLOWS: &[&str] = &["payroll", "treasury_disbursement"];
const OWNED_AGGREGATES: &[&str] = &["treasury_ledger", "payroll_batch"];

fn service_id() -> ServiceId {
    SERVICE_NAME.into()
}

fn treasury_disbursement_workflow_id() -> WorkflowId {
    "treasury_disbursement".into()
}

#[derive(Debug, Default, Clone)]
struct InMemoryDisbursementLedger {
    records: Vec<TreasuryDisbursementRecordedV1>,
}

impl InMemoryDisbursementLedger {
    fn record(&mut self, record: TreasuryDisbursementRecordedV1) {
        self.records.push(record);
    }

    fn records(&self) -> &[TreasuryDisbursementRecordedV1] {
        &self.records
    }
}

#[derive(Debug, Default, Clone)]
pub struct FinanceService {
    disbursements: InMemoryDisbursementLedger,
}

impl FinanceService {
    pub fn record_disbursement(
        &mut self,
        context: &ApprovedMutationContext,
        request: TreasuryDisbursementRequestV1,
    ) -> InstitutionalResult<TreasuryDisbursementRecordedV1> {
        context.assert_workflow(&treasury_disbursement_workflow_id())?;
        context.assert_target_service(&service_id())?;

        let approved_by_roles = context
            .approvals()
            .iter()
            .map(|decision| decision.approver_role)
            .collect();
        let record = TreasuryDisbursementRecordedV1::new(
            context.trace_context().correlation_id.to_string(),
            &request,
            approved_by_roles,
        );
        self.disbursements.record(record.clone());
        Ok(record)
    }

    #[must_use]
    pub fn disbursements(&self) -> &[TreasuryDisbursementRecordedV1] {
        self.disbursements.records()
    }
}

#[must_use]
pub fn service_boundary() -> ServiceBoundaryV1 {
    ServiceBoundaryV1 {
        service_name: SERVICE_NAME.to_owned(),
        domain: DOMAIN_NAME.to_owned(),
        approved_workflows: APPROVED_WORKFLOWS.iter().copied().map(Into::into).collect(),
        owned_aggregates: OWNED_AGGREGATES
            .iter()
            .copied()
            .map(str::to_owned)
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    mod contract_parity {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../testing/contract_parity.rs"
        ));
    }

    use contract_parity::assert_service_boundary_matches_catalog;

    use super::{DOMAIN_NAME, service_boundary};

    #[test]
    fn service_boundary_matches_enterprise_catalog() {
        let source =
            include_str!("../../../enterprise/domains/finance_treasury/service_boundaries.toml");
        let boundary = service_boundary();

        assert_service_boundary_matches_catalog(&boundary, DOMAIN_NAME, source);
    }
}
