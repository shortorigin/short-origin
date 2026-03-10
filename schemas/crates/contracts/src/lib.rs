use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use identity::{
    ActionId, ActorRef, AggregateId, DecisionId, EnvironmentId, EvidenceId, InstitutionalRole,
    ServiceId, WorkflowId,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use telemetry::DecisionRef;
use uuid::Uuid;

mod decision;
mod knowledge;

pub use decision::*;
pub use knowledge::*;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Classification {
    Public,
    Internal,
    Confidential,
    Restricted,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ImpactTier {
    Tier0,
    Tier1,
    Tier2,
    Tier3,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PolicyDecisionOutcome {
    Allow,
    Deny,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PayloadRefV1 {
    pub schema_ref: String,
    pub record_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommandV1 {
    pub command_id: String,
    pub target_service: ServiceId,
    pub target_aggregate: AggregateId,
    pub actor_ref: ActorRef,
    pub authority_ref: String,
    pub classification: Classification,
    pub payload_ref: PayloadRefV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QueryV1 {
    pub query_id: String,
    pub target_service: ServiceId,
    pub actor_ref: ActorRef,
    pub purpose: String,
    pub classification: Classification,
    pub selector: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicyDecisionRequestV1 {
    pub request_id: ActionId,
    pub actor_ref: ActorRef,
    pub action: WorkflowId,
    pub resource: ServiceId,
    pub environment: EnvironmentId,
    pub impact_tier: ImpactTier,
    pub classification: Classification,
    pub cross_domain: bool,
    pub policy_refs: Vec<String>,
    pub exception_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicyDecisionV1 {
    pub decision_id: DecisionId,
    pub request_id: ActionId,
    pub decision: PolicyDecisionOutcome,
    pub obligations: Vec<String>,
    pub denial_reasons: Vec<String>,
    pub evidence_refs: Vec<EvidenceId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApprovalRequestV1 {
    pub action_id: ActionId,
    pub approval_scope: WorkflowId,
    pub required_approver_roles: Vec<InstitutionalRole>,
    pub minimum_approvals: usize,
    pub impact_tier: ImpactTier,
    pub expires_at: DateTime<Utc>,
    pub rationale: String,
}

impl ApprovalRequestV1 {
    #[must_use]
    pub fn required_approval_count(&self) -> usize {
        let tier_minimum = match self.impact_tier {
            ImpactTier::Tier0 => 0,
            ImpactTier::Tier1 => 1,
            ImpactTier::Tier2 | ImpactTier::Tier3 => 2,
        };

        self.minimum_approvals.max(tier_minimum)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApprovalDecisionV1 {
    pub action_id: ActionId,
    pub approver: ActorRef,
    pub approver_role: InstitutionalRole,
    pub approved: bool,
    pub rationale: String,
    pub decided_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExceptionRecordV1 {
    pub exception_id: String,
    pub policy_ref: String,
    pub scope: String,
    pub compensating_controls: Vec<String>,
    pub approver_roles: Vec<InstitutionalRole>,
    pub expires_at: DateTime<Utc>,
    pub review_cadence_days: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RiskRecordV1 {
    pub risk_id: String,
    pub source: String,
    pub likelihood: String,
    pub impact: String,
    pub owner_role: InstitutionalRole,
    pub treatment: String,
    pub accepted_until: Option<DateTime<Utc>>,
    pub review_date: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceManifestV1 {
    pub evidence_id: EvidenceId,
    pub producer: String,
    pub artifact_hash: String,
    pub storage_ref: String,
    pub retention_class: String,
    pub classification: Classification,
    pub related_decision_refs: Vec<DecisionRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentActionRequestV1 {
    pub action_id: ActionId,
    pub actor_ref: ActorRef,
    pub objective: String,
    pub requested_workflow: WorkflowId,
    pub impact_tier: ImpactTier,
    pub classification: Classification,
    pub required_approver_roles: Vec<InstitutionalRole>,
    pub policy_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentHandoffV1 {
    pub handoff_id: String,
    pub objective: String,
    pub inputs_used: Vec<String>,
    pub decisions_made: Vec<String>,
    pub pending_actions: Vec<String>,
    pub verification_status: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VerificationStatusV1 {
    Draft,
    InReview,
    Passed,
    Failed,
    Blocked,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum QuestionSeverityV1 {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OpenQuestionV1 {
    pub question: String,
    pub severity: QuestionSeverityV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProfileEvidenceV1 {
    pub area: String,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProcessTraceabilityV1 {
    pub work_item_id: String,
    pub parent_work_item_id: Option<String>,
    pub iteration: u32,
    pub affected_paths: Vec<String>,
    pub affected_modules: Vec<String>,
    pub policy_refs: Vec<String>,
    pub acceptance_criteria: Vec<String>,
    pub open_questions: Vec<OpenQuestionV1>,
    pub verification_status: VerificationStatusV1,
}

impl ProcessTraceabilityV1 {
    #[must_use]
    pub fn has_high_severity_open_questions(&self) -> bool {
        self.open_questions
            .iter()
            .any(|question| question.severity == QuestionSeverityV1::High)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskContractV1 {
    pub issue_id: u64,
    pub issue_url: String,
    pub branch: String,
    pub primary_architectural_plane: String,
    pub owning_subsystem: String,
    pub architectural_references: Vec<String>,
    pub allowed_touchpoints: Vec<String>,
    pub non_goals: Vec<String>,
    pub scope_in: Vec<String>,
    pub scope_out: Vec<String>,
    pub target_paths: Vec<String>,
    pub acceptance_criteria: Vec<String>,
    pub validation_commands: Vec<String>,
    pub validation_artifacts: Vec<String>,
    pub rollback_path: String,
    pub exec_plan_required: bool,
    pub exec_plan_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkItemV1 {
    #[serde(flatten)]
    pub traceability: ProcessTraceabilityV1,
    pub root_work_item_id: String,
    pub title: String,
    pub objective: String,
    pub status: String,
    pub current_stage: String,
    pub child_work_item_ids: Vec<String>,
    pub decomposition_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResearchSynthesisV1 {
    #[serde(flatten)]
    pub traceability: ProcessTraceabilityV1,
    pub completed_at: DateTime<Utc>,
    pub objective: String,
    pub research_inputs: Vec<String>,
    pub source_refs: Vec<String>,
    pub findings: Vec<String>,
    pub constraints: Vec<String>,
    pub decomposition_signals: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RequirementsSpecV1 {
    #[serde(flatten)]
    pub traceability: ProcessTraceabilityV1,
    pub completed_at: DateTime<Utc>,
    pub objective: String,
    pub functional_requirements: Vec<String>,
    pub non_functional_requirements: Vec<String>,
    pub out_of_scope: Vec<String>,
    pub success_metrics: Vec<String>,
    pub assumptions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArchitectureDesignV1 {
    #[serde(flatten)]
    pub traceability: ProcessTraceabilityV1,
    pub completed_at: DateTime<Utc>,
    pub objective: String,
    pub design_summary: Vec<String>,
    pub public_interface_changes: Vec<String>,
    pub boundary_impacts: Vec<String>,
    pub decomposition_decision: String,
    pub child_work_item_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ImplementationPlanV1 {
    #[serde(flatten)]
    pub traceability: ProcessTraceabilityV1,
    pub completed_at: DateTime<Utc>,
    pub objective: String,
    pub change_slices: Vec<String>,
    pub target_paths: Vec<String>,
    pub test_scenarios: Vec<String>,
    pub profile_evidence: Vec<ProfileEvidenceV1>,
    pub rollout_notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChangeBatchV1 {
    #[serde(flatten)]
    pub traceability: ProcessTraceabilityV1,
    pub completed_at: DateTime<Utc>,
    pub batch_id: String,
    pub summary: Vec<String>,
    pub target_paths: Vec<String>,
    pub change_kinds: Vec<String>,
    pub prerequisite_checks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValidationReportV1 {
    #[serde(flatten)]
    pub traceability: ProcessTraceabilityV1,
    pub completed_at: DateTime<Utc>,
    pub objective: String,
    pub checks_run: Vec<String>,
    pub passed_checks: Vec<String>,
    pub failed_checks: Vec<String>,
    pub findings: Vec<String>,
    pub changed_paths_validated: Vec<String>,
    pub profile_evidence: Vec<ProfileEvidenceV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RefinementRecordV1 {
    #[serde(flatten)]
    pub traceability: ProcessTraceabilityV1,
    pub completed_at: DateTime<Utc>,
    pub decision: String,
    pub improvements: Vec<String>,
    pub residual_risks: Vec<String>,
    pub next_work_item_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServiceBoundaryV1 {
    pub service_name: String,
    pub domain: String,
    pub approved_workflows: Vec<String>,
    pub owned_aggregates: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkflowBoundaryV1 {
    pub workflow_name: String,
    pub touched_domains: Vec<String>,
    pub target_services: Vec<String>,
    pub emits_evidence: bool,
    pub mutation_path_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TreasuryDisbursementRequestV1 {
    pub ledger_ref: String,
    pub amount_minor: u64,
    pub currency: String,
    pub beneficiary: String,
    pub justification: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TreasuryDisbursementRecordedV1 {
    pub disbursement_id: String,
    pub workflow_execution_id: String,
    pub ledger_ref: String,
    pub amount_minor: u64,
    pub currency: String,
    pub beneficiary: String,
    pub approved_by_roles: Vec<InstitutionalRole>,
}

impl TreasuryDisbursementRecordedV1 {
    #[must_use]
    pub fn new(
        workflow_execution_id: impl Into<String>,
        request: &TreasuryDisbursementRequestV1,
        approved_by_roles: Vec<InstitutionalRole>,
    ) -> Self {
        Self {
            disbursement_id: Uuid::new_v4().to_string(),
            workflow_execution_id: workflow_execution_id.into(),
            ledger_ref: request.ledger_ref.clone(),
            amount_minor: request.amount_minor,
            currency: request.currency.clone(),
            beneficiary: request.beneficiary.clone(),
            approved_by_roles,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum VenueV1 {
    #[serde(alias = "Coinbase")]
    Coinbase,
    #[serde(alias = "Oanda")]
    Oanda,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum AssetClassV1 {
    #[serde(alias = "Crypto")]
    Crypto,
    #[serde(alias = "Forex")]
    Forex,
    #[serde(alias = "Equity")]
    Equity,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SymbolV1 {
    pub venue: VenueV1,
    pub asset_class: AssetClassV1,
    pub base: String,
    pub quote: String,
}

impl SymbolV1 {
    #[must_use]
    pub fn new(
        venue: VenueV1,
        asset_class: AssetClassV1,
        base: impl Into<String>,
        quote: impl Into<String>,
    ) -> Self {
        Self {
            venue,
            asset_class,
            base: base.into(),
            quote: quote.into(),
        }
    }

    #[must_use]
    pub fn ticker(&self) -> String {
        format!("{}{}", self.base, self.quote)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OhlcvBarV1 {
    pub symbol: SymbolV1,
    pub open_time: DateTime<Utc>,
    pub close_time: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TradeTickV1 {
    pub symbol: SymbolV1,
    pub trade_time: DateTime<Utc>,
    pub price: f64,
    pub size: f64,
    pub trade_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MarketEventV1 {
    Bar(OhlcvBarV1),
    Trade(TradeTickV1),
}

impl MarketEventV1 {
    #[must_use]
    pub fn event_time(&self) -> DateTime<Utc> {
        match self {
            Self::Bar(bar) => bar.close_time,
            Self::Trade(tick) => tick.trade_time,
        }
    }

    #[must_use]
    pub fn symbol(&self) -> &SymbolV1 {
        match self {
            Self::Bar(bar) => &bar.symbol,
            Self::Trade(tick) => &tick.symbol,
        }
    }

    #[must_use]
    pub fn price(&self) -> f64 {
        match self {
            Self::Bar(bar) => bar.close,
            Self::Trade(tick) => tick.price,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HistoricalDataRequestV1 {
    pub symbol: SymbolV1,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub interval_seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawMarketRecordV1 {
    pub source_symbol: String,
    pub event_time: DateTime<Utc>,
    pub open: Option<f64>,
    pub high: Option<f64>,
    pub low: Option<f64>,
    pub close: f64,
    pub volume: f64,
    pub trade_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthStatusV1 {
    Healthy,
    Degraded(String),
    Unhealthy(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct DeterminismKeyV1 {
    pub event_id: String,
    pub model_version: String,
    pub config_hash: String,
}

impl DeterminismKeyV1 {
    #[must_use]
    pub fn new(
        event_id: impl Into<String>,
        model_version: impl Into<String>,
        config_hash: impl Into<String>,
    ) -> Self {
        Self {
            event_id: event_id.into(),
            model_version: model_version.into(),
            config_hash: config_hash.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StrategyConfigV1 {
    pub strategy_id: String,
    pub model_version: String,
    pub config_hash: String,
    pub parameters: Value,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SignalSideV1 {
    #[serde(alias = "Buy")]
    Buy,
    #[serde(alias = "Sell")]
    Sell,
    #[serde(alias = "Hold")]
    Hold,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SignalV1 {
    pub strategy_id: String,
    pub symbol: SymbolV1,
    pub side: SignalSideV1,
    pub quantity: f64,
    pub confidence: f64,
    pub reason: String,
    pub determinism: DeterminismKeyV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StrategyStateSnapshotV1 {
    pub strategy_id: String,
    pub timestamp: DateTime<Utc>,
    pub state: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SimulationConfigV1 {
    pub seed: u64,
    pub fee_bps: f64,
    pub slippage_bps: f64,
    pub latency_ms: u64,
    pub initial_cash: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EquityPointV1 {
    pub timestamp: DateTime<Utc>,
    pub equity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PerformanceSummaryV1 {
    pub total_return: f64,
    pub sharpe: f64,
    pub max_drawdown: f64,
    pub turnover: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BacktestResultV1 {
    pub config_hash: String,
    pub seed: u64,
    pub equity_curve: Vec<EquityPointV1>,
    pub summary: PerformanceSummaryV1,
    pub trade_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MarketDataBatchV1 {
    pub dataset_id: String,
    pub dataset_name: String,
    pub venue: VenueV1,
    pub event_count: usize,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub events: Vec<MarketEventV1>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OrderSideV1 {
    #[serde(alias = "Buy")]
    Buy,
    #[serde(alias = "Sell")]
    Sell,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OrderTypeV1 {
    #[serde(alias = "Market")]
    Market,
    #[serde(alias = "Limit")]
    Limit,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TimeInForceV1 {
    #[serde(alias = "Gtc")]
    Gtc,
    #[serde(alias = "Ioc")]
    Ioc,
    #[serde(alias = "Fok")]
    Fok,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrderRequestV1 {
    pub order_id: String,
    pub strategy_id: String,
    pub symbol: SymbolV1,
    pub venue: VenueV1,
    pub side: OrderSideV1,
    pub quantity: f64,
    pub limit_price: Option<f64>,
    pub order_type: OrderTypeV1,
    pub tif: TimeInForceV1,
    pub submitted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrderStatusV1 {
    Accepted,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrderAckV1 {
    pub order_id: String,
    pub status: OrderStatusV1,
    pub venue_order_id: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FillV1 {
    pub fill_id: String,
    pub order_id: String,
    pub symbol: SymbolV1,
    pub side: OrderSideV1,
    pub quantity: f64,
    pub price: f64,
    pub fee: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RiskDecisionV1 {
    pub approved: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TradingRiskPolicyV1 {
    pub max_order_notional: f64,
    pub max_gross_exposure: f64,
    pub max_open_orders: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TradingRiskSnapshotV1 {
    pub as_of: DateTime<Utc>,
    pub gross_exposure: f64,
    pub net_exposure: f64,
    pub open_orders: usize,
    pub kill_switch_armed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum LedgerAccountV1 {
    Cash,
    Position,
    Fees,
    Funding,
    Transfers,
    CorporateActions,
    RealizedPnl,
    UnrealizedPnl,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LedgerEntryV1 {
    pub timestamp: DateTime<Utc>,
    pub account: LedgerAccountV1,
    pub symbol: Option<SymbolV1>,
    pub amount: f64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PositionSnapshotV1 {
    pub symbol: SymbolV1,
    pub quantity: f64,
    pub avg_price: f64,
    pub mark_price: f64,
    pub unrealized_pnl: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PortfolioSnapshotV1 {
    pub as_of: DateTime<Utc>,
    pub cash: f64,
    pub realized_pnl: f64,
    pub unrealized_pnl: f64,
    pub positions: Vec<PositionSnapshotV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FeatureRowV1 {
    pub timestamp: DateTime<Utc>,
    pub symbol: String,
    pub close: f64,
    pub return_1: f64,
    pub rolling_vol_20: f64,
    pub microstructure_proxy: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExperimentConfigV1 {
    pub strategy_name: String,
    pub parameter_grid: BTreeMap<String, f64>,
    pub training_window: usize,
    pub test_window: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExperimentMetadataV1 {
    #[serde(default)]
    pub trade_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExperimentResultV1 {
    pub config_hash: String,
    pub summary: PerformanceSummaryV1,
    pub metadata: ExperimentMetadataV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResearchTaskV1 {
    pub task_id: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrderAuditRecordV1 {
    pub audit_id: String,
    pub recorded_at: DateTime<Utc>,
    pub order: OrderRequestV1,
    pub decision_trace: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LimitBreachRecordV1 {
    pub breach_id: String,
    pub detected_at: DateTime<Utc>,
    pub control: String,
    pub severity: String,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BestExecutionRecordV1 {
    pub record_id: String,
    pub venue: VenueV1,
    pub captured_at: DateTime<Utc>,
    pub slippage_bps: f64,
    pub expected_price: f64,
    pub executed_price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DailyControlAttestationV1 {
    pub attestation_id: String,
    pub business_date: String,
    pub generated_at: DateTime<Utc>,
    pub approved_models: Vec<String>,
    pub controls_checked: Vec<String>,
    pub exceptions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComplianceReportV1 {
    pub order_audit_records: Vec<OrderAuditRecordV1>,
    pub limit_breach_records: Vec<LimitBreachRecordV1>,
    pub best_execution_records: Vec<BestExecutionRecordV1>,
    pub daily_control_attestation: DailyControlAttestationV1,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ModelApprovalStatusV1 {
    Pending,
    Approved,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelApprovalV1 {
    pub model_id: String,
    pub version: String,
    pub approved_by: Option<String>,
    pub status: ModelApprovalStatusV1,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PromotionGateV1 {
    pub backtest_evidence: bool,
    pub paper_trade_evidence: bool,
    pub risk_signoff: bool,
    pub compliance_attested: bool,
}

impl PromotionGateV1 {
    #[must_use]
    pub fn ready(&self) -> bool {
        self.backtest_evidence
            && self.paper_trade_evidence
            && self.risk_signoff
            && self.compliance_attested
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PromotionRecommendationV1 {
    pub recommendation_id: String,
    pub strategy_id: String,
    pub config_hash: String,
    pub recommended: bool,
    pub summary: String,
    pub required_workflows: Vec<String>,
    pub gate: PromotionGateV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QuantStrategyPromotionRequestV1 {
    pub promotion_id: String,
    pub strategy_id: String,
    pub business_date: String,
    pub seed: u64,
    pub configuration_ref: String,
}

pub type AssetClass = AssetClassV1;
pub type BacktestResult = BacktestResultV1;
pub type BestExecutionRecord = BestExecutionRecordV1;
pub type ComplianceReport = ComplianceReportV1;
pub type DailyControlAttestation = DailyControlAttestationV1;
pub type DeterminismKey = DeterminismKeyV1;
pub type EquityPoint = EquityPointV1;
pub type ExperimentConfig = ExperimentConfigV1;
pub type ExperimentResult = ExperimentResultV1;
pub type FeatureRow = FeatureRowV1;
pub type Fill = FillV1;
pub type HealthStatus = HealthStatusV1;
pub type HistoricalDataRequest = HistoricalDataRequestV1;
pub type LedgerAccount = LedgerAccountV1;
pub type LedgerEntry = LedgerEntryV1;
pub type LimitBreachRecord = LimitBreachRecordV1;
pub type MarketDataBatch = MarketDataBatchV1;
pub type MarketEvent = MarketEventV1;
pub type ModelApproval = ModelApprovalV1;
pub type ModelApprovalStatus = ModelApprovalStatusV1;
pub type OhlcvBar = OhlcvBarV1;
pub type OrderAck = OrderAckV1;
pub type OrderAuditRecord = OrderAuditRecordV1;
pub type OrderRequest = OrderRequestV1;
pub type OrderStatus = OrderStatusV1;
pub type OrderType = OrderTypeV1;
pub type PerformanceSummary = PerformanceSummaryV1;
pub type PortfolioSnapshot = PortfolioSnapshotV1;
pub type PositionSnapshot = PositionSnapshotV1;
pub type PromotionGate = PromotionGateV1;
pub type PromotionRecommendation = PromotionRecommendationV1;
pub type QuantStrategyPromotionRequest = QuantStrategyPromotionRequestV1;
pub type RawMarketRecord = RawMarketRecordV1;
pub type ResearchTask = ResearchTaskV1;
pub type RiskDecision = RiskDecisionV1;
pub type RiskSnapshot = TradingRiskSnapshotV1;
pub type Side = OrderSideV1;
pub type Signal = SignalV1;
pub type SignalSide = SignalSideV1;
pub type SimulationConfig = SimulationConfigV1;
pub type StrategyConfig = StrategyConfigV1;
pub type StrategyStateSnapshot = StrategyStateSnapshotV1;
pub type Symbol = SymbolV1;
pub type TimeInForce = TimeInForceV1;
pub type TradeTick = TradeTickV1;
pub type TradingRiskPolicy = TradingRiskPolicyV1;
pub type TradingRiskSnapshot = TradingRiskSnapshotV1;
pub type Venue = VenueV1;
