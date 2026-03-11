use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;

use contracts::{
    BestExecutionRecordV1, ComplianceReportV1, DailyControlAttestationV1, LimitBreachRecordV1,
    OrderAuditRecordV1, OrderRequestV1, ServiceBoundaryV1,
};
use error_model::{InstitutionalError, InstitutionalResult};
use trading_core::{Clock, IdGenerator, SystemClock, SystemIdGenerator};

#[derive(Debug, Clone)]
pub struct CapitalLimits {
    pub global_notional_limit: f64,
    pub strategy_limits: BTreeMap<String, f64>,
}

impl CapitalLimits {
    pub fn check(
        &self,
        strategy_id: &str,
        order_notional: f64,
        ids: &dyn IdGenerator,
        clock: &dyn Clock,
    ) -> Option<LimitBreachRecordV1> {
        if order_notional > self.global_notional_limit {
            return Some(LimitBreachRecordV1 {
                breach_id: ids.next_id(),
                detected_at: clock.now(),
                control: "global_notional_limit".to_string(),
                severity: "high".to_string(),
                details: format!(
                    "order {:.2} > global {:.2}",
                    order_notional, self.global_notional_limit
                ),
            });
        }

        if let Some(limit) = self.strategy_limits.get(strategy_id)
            && order_notional > *limit
        {
            return Some(LimitBreachRecordV1 {
                breach_id: ids.next_id(),
                detected_at: clock.now(),
                control: "strategy_notional_limit".to_string(),
                severity: "medium".to_string(),
                details: format!("order {:.2} > strategy {:.2}", order_notional, limit),
            });
        }

        None
    }
}

#[derive(Debug, Clone)]
pub struct TradeSurveillance {
    pub max_single_order_quantity: f64,
}

impl TradeSurveillance {
    pub fn inspect(
        &self,
        order: &OrderRequestV1,
        ids: &dyn IdGenerator,
        clock: &dyn Clock,
    ) -> Option<LimitBreachRecordV1> {
        if order.quantity > self.max_single_order_quantity {
            return Some(LimitBreachRecordV1 {
                breach_id: ids.next_id(),
                detected_at: clock.now(),
                control: "max_single_order_quantity".to_string(),
                severity: "medium".to_string(),
                details: format!("quantity {:.4} exceeds limit", order.quantity),
            });
        }
        None
    }
}

pub struct CompliancePackBuilder {
    order_audits: Vec<OrderAuditRecordV1>,
    breaches: Vec<LimitBreachRecordV1>,
    best_execution: Vec<BestExecutionRecordV1>,
    controls_checked: HashSet<String>,
    exceptions: Vec<String>,
    ids: Arc<dyn IdGenerator>,
    clock: Arc<dyn Clock>,
}

impl std::fmt::Debug for CompliancePackBuilder {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CompliancePackBuilder")
            .field("order_audits", &self.order_audits.len())
            .field("breaches", &self.breaches.len())
            .field("best_execution", &self.best_execution.len())
            .finish_non_exhaustive()
    }
}

impl CompliancePackBuilder {
    pub fn capture_order_audit(
        &mut self,
        order: OrderRequestV1,
        decision_trace: impl Into<String>,
    ) {
        self.order_audits.push(OrderAuditRecordV1 {
            audit_id: self.ids.next_id(),
            recorded_at: self.clock.now(),
            order,
            decision_trace: decision_trace.into(),
        });
    }

    pub fn capture_best_execution(&mut self, record: BestExecutionRecordV1) {
        self.best_execution.push(record);
    }

    pub fn capture_breach(&mut self, breach: LimitBreachRecordV1) {
        self.exceptions
            .push(format!("{}: {}", breach.control, breach.details));
        self.breaches.push(breach);
    }

    pub fn check_control(&mut self, control: &str) {
        self.controls_checked.insert(control.to_string());
    }

    #[must_use]
    pub fn build(self, date: &str, approved_models: Vec<String>) -> ComplianceReportV1 {
        let mut controls_checked = self.controls_checked.into_iter().collect::<Vec<_>>();
        controls_checked.sort();
        ComplianceReportV1 {
            order_audit_records: self.order_audits,
            limit_breach_records: self.breaches,
            best_execution_records: self.best_execution,
            daily_control_attestation: DailyControlAttestationV1 {
                attestation_id: self.ids.next_id(),
                business_date: date.to_string(),
                generated_at: self.clock.now(),
                approved_models,
                controls_checked,
                exceptions: self.exceptions,
            },
        }
    }
}

pub struct ComplianceService {
    reports: Vec<ComplianceReportV1>,
    ids: Arc<dyn IdGenerator>,
    clock: Arc<dyn Clock>,
}

impl std::fmt::Debug for ComplianceService {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ComplianceService")
            .field("reports", &self.reports.len())
            .finish_non_exhaustive()
    }
}

impl Default for ComplianceService {
    fn default() -> Self {
        Self::new(Arc::new(SystemClock), Arc::new(SystemIdGenerator))
    }
}

impl ComplianceService {
    #[must_use]
    pub fn new(clock: Arc<dyn Clock>, ids: Arc<dyn IdGenerator>) -> Self {
        Self {
            reports: Vec::new(),
            ids,
            clock,
        }
    }

    #[must_use]
    pub fn new_builder(&self) -> CompliancePackBuilder {
        CompliancePackBuilder {
            order_audits: Vec::new(),
            breaches: Vec::new(),
            best_execution: Vec::new(),
            controls_checked: HashSet::new(),
            exceptions: Vec::new(),
            ids: Arc::clone(&self.ids),
            clock: Arc::clone(&self.clock),
        }
    }

    pub fn record_report(&mut self, report: ComplianceReportV1) -> InstitutionalResult<()> {
        validate_compliance_pack(&report)?;
        self.reports.push(report);
        Ok(())
    }

    #[must_use]
    pub fn reports(&self) -> &[ComplianceReportV1] {
        &self.reports
    }
}

pub fn validate_compliance_pack(report: &ComplianceReportV1) -> InstitutionalResult<()> {
    if report.daily_control_attestation.controls_checked.is_empty() {
        return Err(InstitutionalError::invariant(
            error_model::OperationContext::new(
                "services/compliance-service",
                "validate_compliance_pack",
            ),
            "controls_checked cannot be empty",
        ));
    }
    if report.daily_control_attestation.business_date.is_empty() {
        return Err(InstitutionalError::invariant(
            error_model::OperationContext::new(
                "services/compliance-service",
                "validate_compliance_pack",
            ),
            "business_date required",
        ));
    }
    Ok(())
}

#[must_use]
pub fn service_boundary() -> ServiceBoundaryV1 {
    ServiceBoundaryV1 {
        service_name: "compliance-service".into(),
        domain: "compliance".to_owned(),
        approved_workflows: vec![
            "compliance_attestation".into(),
            "policy_exception".into(),
            "quant_strategy_promotion".into(),
        ],
        owned_aggregates: vec![
            "control_attestation".into(),
            "compliance_checkpoint".into(),
            "compliance_pack".into(),
            "best_execution_report".into(),
        ],
    }
}
