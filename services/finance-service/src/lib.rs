pub mod component;

use contracts::{ServiceBoundaryV1, TreasuryDisbursementRecordedV1, TreasuryDisbursementRequestV1};
use enforcement::ApprovedMutationContext;
use error_model::InstitutionalResult;

#[derive(Debug, Default, Clone)]
pub struct FinanceService {
    disbursements: Vec<TreasuryDisbursementRecordedV1>,
}

impl FinanceService {
    pub fn record_disbursement(
        &mut self,
        context: &ApprovedMutationContext,
        request: TreasuryDisbursementRequestV1,
    ) -> InstitutionalResult<TreasuryDisbursementRecordedV1> {
        context.assert_workflow("treasury_disbursement")?;
        context.assert_target_service("finance-service")?;

        let approved_by_roles = context
            .approvals()
            .iter()
            .map(|decision| decision.approver_role)
            .collect();
        let record = TreasuryDisbursementRecordedV1::new(
            context.trace_context().correlation_id.clone(),
            &request,
            approved_by_roles,
        );
        self.disbursements.push(record.clone());
        Ok(record)
    }

    #[must_use]
    pub fn disbursements(&self) -> &[TreasuryDisbursementRecordedV1] {
        &self.disbursements
    }
}

#[must_use]
pub fn service_boundary() -> ServiceBoundaryV1 {
    ServiceBoundaryV1 {
        service_name: "finance-service".to_owned(),
        domain: "finance_treasury".to_owned(),
        approved_workflows: vec!["payroll".to_owned(), "treasury_disbursement".to_owned()],
        owned_aggregates: vec!["treasury_ledger".to_owned(), "payroll_batch".to_owned()],
    }
}
