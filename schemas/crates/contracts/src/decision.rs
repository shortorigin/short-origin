use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use identity::{ActorRef, DecisionId};
use serde::{Deserialize, Serialize};

/// Records the provenance and assumption set behind a decision artifact.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProvenanceV1 {
    /// Identifies the system or component that produced the artifact.
    pub source_system: String,
    /// Captures traceable references to the source inputs used during evaluation.
    pub source_refs: Vec<String>,
    /// Names the engine or workflow step that produced the artifact.
    pub generated_by: String,
    /// Lists explicit assumptions preserved for audit reconstruction.
    pub assumptions: Vec<String>,
}

/// Classifies the type of platform decision being evaluated.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum DecisionClassV1 {
    /// Strategic prioritization across competing initiatives or tracks.
    StrategicPrioritization,
    /// Release or rollout risk evaluation before a governed change.
    ReleaseRiskAssessment,
    /// Resource allocation across finite teams, budgets, or capacity pools.
    ResourceAllocation,
    /// Anomaly routing and escalation selection under uncertainty.
    AnomalyEscalationRouting,
    /// Policy-constrained action selection where governance remains authoritative.
    PolicyConstrainedActionSelection,
}

/// Tracks the lifecycle state of a decision context or artifact.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum DecisionStateV1 {
    /// The decision request has been created but not yet evaluated.
    Pending,
    /// The decision request has been evaluated and carries a recommendation.
    Evaluated,
    /// The decision request is retained for history only.
    Archived,
}

/// Grades the risk tier attached to a decision request.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum RiskTierV1 {
    /// Lowest-risk, read-only or easily reversible decision surface.
    Tier0,
    /// Low-risk decision with narrow blast radius.
    Tier1,
    /// Elevated-risk decision that crosses boundaries or requires stronger review.
    Tier2,
    /// Highest-risk decision that requires the strongest governance scrutiny.
    Tier3,
}

/// States the minimum approval pattern expected before execution can proceed.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalRequirementV1 {
    /// No additional human approval is required after the policy gate passes.
    None,
    /// A designated domain owner must approve execution.
    DomainOwner,
    /// Two recorded human approvals are required.
    DualApproval,
    /// Institutional council review is required before execution.
    InstitutionalCouncil,
}

/// Describes how reversible an option remains after execution.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ReversibilityClassV1 {
    /// The action can be rolled back directly and safely.
    Reversible,
    /// The action can be rolled back only through a governed rollback path.
    GuardedRollback,
    /// The action is materially irreversible.
    Irreversible,
}

/// Identifies the type of constraint that applies during decision evaluation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ConstraintKindV1 {
    /// Reject options whose risk score exceeds the threshold.
    MaxRiskScore,
    /// Reject options whose aggregate confidence falls below the threshold.
    MinConfidenceScore,
    /// Reject options whose expected utility falls below the threshold.
    MinExpectedUtility,
    /// Reject options that are not reversible enough for the decision surface.
    RequiresReversible,
    /// Reject options that do not carry a rollback plan.
    RequiresRollbackPlan,
}

/// Captures a single expected outcome contribution for an option.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OutcomeEstimateV1 {
    /// Stable identifier for the estimate entry.
    pub estimate_id: String,
    /// Human-readable explanation of the expected outcome.
    pub description: String,
    /// Probability assigned to the estimate.
    pub probability: f64,
    /// Utility contribution expected from the outcome.
    pub expected_utility: f64,
    /// Risk penalty or bonus associated with the outcome.
    pub risk_adjustment: f64,
    /// Confidence in the estimate.
    pub confidence: f64,
    /// Audit-oriented rationale for the estimate.
    pub rationale: String,
}

/// Represents one discrete scenario within an outcome distribution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OutcomeScenarioV1 {
    /// Scenario label used in audit output.
    pub label: String,
    /// Probability assigned to the scenario.
    pub probability: f64,
    /// Utility realized if the scenario occurs.
    pub utility: f64,
    /// Risk realized if the scenario occurs.
    pub risk: f64,
}

/// Captures the distribution used for probabilistic sampling.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OutcomeDistributionV1 {
    /// Stable identifier for the distribution.
    pub distribution_id: String,
    /// Discrete scenarios used by the Monte Carlo sampler.
    pub scenarios: Vec<OutcomeScenarioV1>,
    /// Expected value of the distribution.
    pub expected_value: f64,
    /// Variance of the distribution.
    pub variance: f64,
    /// Probability mass assigned to downside scenarios.
    pub downside_probability: f64,
    /// Explanation of the distribution construction.
    pub rationale: String,
}

/// Summarizes the risk posture associated with an option.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RiskAssessmentV1 {
    /// Aggregate risk score on a normalized 0..1 scale.
    pub risk_score: f64,
    /// Probability of a downside or adverse outcome.
    pub downside_probability: f64,
    /// Tail-risk score on a normalized 0..1 scale.
    pub tail_risk_score: f64,
    /// Confidence in the risk assessment.
    pub confidence: f64,
    /// Rationale for the risk posture.
    pub rationale: String,
    /// Mitigation notes preserved with the assessment.
    pub mitigation: String,
}

/// Breaks utility into explicit governed dimensions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UtilityBreakdownV1 {
    /// Long-horizon value creation contribution.
    pub value_score: f64,
    /// Resilience contribution.
    pub resilience_score: f64,
    /// Compliance contribution.
    pub compliance_score: f64,
    /// Cost efficiency contribution.
    pub cost_score: f64,
    /// Reversibility contribution.
    pub reversibility_score: f64,
    /// Strategic fit contribution.
    pub strategic_fit_score: f64,
    /// Explanation of the utility decomposition.
    pub rationale: String,
}

/// Defines a single decision option evaluated by the engine.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DecisionOptionV1 {
    /// Stable identifier for the option.
    pub option_id: String,
    /// Human-readable option title.
    pub title: String,
    /// Description of the option and intended action.
    pub description: String,
    /// Expected outcome estimates attached to the option.
    pub expected_outcomes: Vec<OutcomeEstimateV1>,
    /// Distribution used for seeded scenario sampling.
    pub outcome_distribution: OutcomeDistributionV1,
    /// Risk posture for the option.
    pub risk_assessment: RiskAssessmentV1,
    /// Utility decomposition for the option.
    pub utility_breakdown: UtilityBreakdownV1,
    /// Rollback notes for the option.
    pub rollback_plan: String,
    /// Reversibility classification for the option.
    pub reversibility: ReversibilityClassV1,
    /// Historical positive binary outcomes for Thompson sampling.
    pub historical_successes: u32,
    /// Historical negative binary outcomes for Thompson sampling.
    pub historical_failures: u32,
}

/// Defines a single hard or soft constraint applied during evaluation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DecisionConstraintV1 {
    /// Stable identifier for the constraint.
    pub constraint_id: String,
    /// Constraint type evaluated by the engine.
    pub kind: ConstraintKindV1,
    /// Human-readable description of the constraint.
    pub description: String,
    /// Numeric threshold used during evaluation.
    pub threshold: f64,
    /// Indicates whether violation forces non-executable status.
    pub hard: bool,
    /// Preserves the rationale behind the threshold.
    pub rationale: String,
}

/// Enumerates the bounded normal-form game used for strategic analysis.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NormalFormGameV1 {
    /// Name of the focal actor whose options are being recommended.
    pub focal_actor: String,
    /// Name of the counterpart actor considered in the game.
    pub counterpart_actor: String,
    /// Strategies available to the focal actor.
    pub focal_strategies: Vec<String>,
    /// Strategies available to the counterpart actor.
    pub counterpart_strategies: Vec<String>,
    /// Payoff cells for the finite normal-form representation.
    pub payoff_cells: Vec<PayoffCellV1>,
}

/// Captures one payoff entry in the normal-form game.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PayoffCellV1 {
    /// Focal strategy name.
    pub focal_strategy: String,
    /// Counterpart strategy name.
    pub counterpart_strategy: String,
    /// Utility-style payoff for the focal actor.
    pub focal_payoff: f64,
    /// Utility-style payoff for the counterpart actor.
    pub counterpart_payoff: f64,
}

/// Defines the authoritative decision input passed to the service and workflow layers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DecisionContextV1 {
    /// Stable identifier for the decision request.
    pub decision_id: DecisionId,
    /// Timestamp at which the decision request was created.
    pub created_at: DateTime<Utc>,
    /// Taxonomy class for the decision.
    pub decision_class: DecisionClassV1,
    /// Lifecycle state for the request.
    pub state: DecisionStateV1,
    /// Actor originating the request.
    pub actor_ref: ActorRef,
    /// Subject or surface being evaluated.
    pub subject: String,
    /// Objective against which alternatives are scored.
    pub objective: String,
    /// Seed controlling all stochastic behavior during evaluation.
    pub evaluation_seed: u64,
    /// Risk tier assigned to the decision.
    pub risk_tier: RiskTierV1,
    /// Approval requirement expected before execution.
    pub approval_requirement: ApprovalRequirementV1,
    /// Policy references associated with the decision.
    pub policy_refs: Vec<String>,
    /// Reversibility expectation at the decision level.
    pub reversibility: ReversibilityClassV1,
    /// Requests learned-model support when available.
    pub requested_learned_support: bool,
    /// Candidate options evaluated by the engine.
    pub options: Vec<DecisionOptionV1>,
    /// Constraints applied before policy gating.
    pub constraints: Vec<DecisionConstraintV1>,
    /// Optional normal-form game used for strategic analysis.
    pub normal_form_game: Option<NormalFormGameV1>,
    /// Provenance metadata for the request.
    pub provenance: ProvenanceV1,
}

/// Represents one ranked option in the recommendation output.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RankedDecisionOptionV1 {
    /// Identifier of the option.
    pub option_id: String,
    /// Ranking position, where 1 is best.
    pub rank: usize,
    /// Weighted expected-value score for the option.
    pub expected_value: f64,
    /// Monte Carlo scenario mean for the option.
    pub sampled_value: f64,
    /// Thompson sample used for repeated binary-outcome choice.
    pub thompson_score: f64,
    /// Aggregated confidence score.
    pub aggregated_confidence: f64,
    /// Aggregated risk score.
    pub aggregated_risk: f64,
    /// Final utility score after aggregation and penalties.
    pub final_utility: f64,
    /// Summary produced by the bounded game analysis.
    pub strategic_summary: String,
    /// Constraint violations observed for the option.
    pub constraint_violations: Vec<String>,
}

/// Captures the recommendation state returned by the evaluation engine.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum RecommendationStatusV1 {
    /// The recommendation is informative only.
    Advisory,
    /// The recommendation passed the policy gate and is executable if invoked by a workflow.
    Executable,
    /// The recommendation was rejected by policy.
    Rejected,
    /// The recommendation cannot execute because constraints failed.
    NonExecutable,
}

/// States the result of the explicit policy gate.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PolicyGateVerdictV1 {
    /// The recommendation passed the gate without additional obligations.
    Passed,
    /// The recommendation remains advisory because additional governance is required.
    Advisory,
    /// The recommendation failed the gate.
    Rejected,
}

/// Tracks whether learned adapters were applied, omitted, or unconfigured.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum LearnedAdapterStatusV1 {
    /// The request did not ask for learned-model assistance.
    NotRequested,
    /// Learned-model assistance was requested but no configured adapter was available.
    NotConfigured,
    /// A deterministic learned-model adapter contributed to the result.
    Applied,
}

/// Records the explicit policy-gate decision over the ranked recommendation output.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicyGateResultV1 {
    /// Stable identifier for the gate result.
    pub gate_id: String,
    /// Decision request identifier.
    pub decision_id: DecisionId,
    /// Timestamp at which the gate result was produced.
    pub evaluated_at: DateTime<Utc>,
    /// Gate verdict.
    pub verdict: PolicyGateVerdictV1,
    /// Indicates whether the recommendation is executable.
    pub executable: bool,
    /// Required approval actions preserved for workflow gating.
    pub required_approval_actions: Vec<String>,
    /// Human-readable rationale for the verdict.
    pub rationale: String,
    /// Explicit rejection reasons when the gate blocks execution.
    pub rejection_reasons: Vec<String>,
    /// Policy references evaluated by the gate.
    pub policy_refs: Vec<String>,
}

/// Carries the selected recommendation and the ranked alternatives.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DecisionRecommendationV1 {
    /// Stable identifier for the recommendation.
    pub recommendation_id: String,
    /// Decision request identifier.
    pub decision_id: DecisionId,
    /// Timestamp at which the recommendation was produced.
    pub generated_at: DateTime<Utc>,
    /// Recommendation status after policy gating.
    pub status: RecommendationStatusV1,
    /// Selected option identifier, if any.
    pub selected_option_id: Option<String>,
    /// Ranked alternatives evaluated by the engine.
    pub ranked_options: Vec<RankedDecisionOptionV1>,
    /// Human-readable rationale for the selected outcome.
    pub rationale: String,
    /// Aggregate confidence score for the recommendation.
    pub confidence: f64,
    /// Aggregate uncertainty score for the recommendation.
    pub uncertainty: f64,
    /// Aggregate risk score for the recommendation.
    pub risk: f64,
    /// Aggregate utility score for the recommendation.
    pub utility: f64,
    /// Reversibility classification for the selected option.
    pub reversibility: ReversibilityClassV1,
    /// Rollback notes preserved with the recommendation.
    pub rollback_plan: String,
    /// Policy status attached to the recommendation.
    pub policy_status: PolicyGateVerdictV1,
    /// Provenance metadata for the recommendation.
    pub provenance: ProvenanceV1,
}

/// Preserves the full structured audit trail for a decision evaluation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DecisionAuditRecordV1 {
    /// Stable identifier for the audit record.
    pub audit_id: String,
    /// Decision request identifier.
    pub decision_id: DecisionId,
    /// Timestamp at which the record was created.
    pub recorded_at: DateTime<Utc>,
    /// Full input context used by the engine.
    pub context: DecisionContextV1,
    /// Ranked options and scoring outputs captured for reconstruction.
    pub evaluated_options: Vec<RankedDecisionOptionV1>,
    /// Selected option identifier, if any.
    pub selected_option_id: Option<String>,
    /// Explicit policy-gate output.
    pub gate_result: PolicyGateResultV1,
    /// Explicit assumptions applied during evaluation.
    pub assumptions: Vec<String>,
    /// Structured rejection rationale when a recommendation is blocked.
    pub rejection_rationale: Vec<String>,
    /// Learned adapter status captured for explainability.
    pub learned_status: LearnedAdapterStatusV1,
    /// Ordered trace of the engine pipeline.
    pub engine_trace: Vec<String>,
    /// Provenance metadata for the audit record.
    pub provenance: ProvenanceV1,
}

/// Provides a deterministic metadata map used by stubs or extensions.
pub type DecisionMetadataV1 = BTreeMap<String, String>;
