use std::collections::BTreeMap;

use contracts::{
    ConstraintKindV1, DecisionAuditRecordV1, DecisionContextV1, DecisionRecommendationV1,
    LearnedAdapterStatusV1, PolicyGateVerdictV1, RankedDecisionOptionV1, RecommendationStatusV1,
    ReversibilityClassV1,
};
use error_model::{InstitutionalError, InstitutionalResult, OperationContext};

use crate::{
    aggregate_confidence, risk_penalized_utility, BaselineForecastModel, BaselineGameModel,
    BaselinePolicyGate, ConfidenceScore, ConstraintViolation, ForecastAdapter, ForecastModel,
    GameAnalysis, GameModel, LearnedAdapterError, NotConfiguredLearnedAdapter, PolicyGate,
    PolicyModelAdapter, ProbabilityScore, RepresentationModel, RiskScore, ThompsonSamplingResult,
    UtilityScore,
};

/// Inputs combined by the utility function after probabilistic and strategic scoring.
#[derive(Debug, Clone, PartialEq)]
pub struct OptionScoreInputs {
    /// Weighted expected-value score.
    pub expected_value: UtilityScore,
    /// Monte Carlo scenario mean.
    pub sampled_value: UtilityScore,
    /// Thompson-sampled success probability.
    pub thompson_score: ProbabilityScore,
    /// Aggregated confidence score.
    pub aggregated_confidence: ConfidenceScore,
    /// Aggregated risk score.
    pub aggregated_risk: RiskScore,
}

/// Utility-combination surface consumed by the decision engine.
pub trait UtilityFunction {
    /// Combines probabilistic and strategic inputs into a final utility score.
    fn combine(
        &self,
        option: &contracts::DecisionOptionV1,
        inputs: &OptionScoreInputs,
        strategic_bonus: f64,
    ) -> UtilityScore;
}

/// Linear risk-adjusted utility combiner.
#[derive(Debug, Clone, Copy)]
pub struct WeightedUtilityFunction {
    penalty_factor: f64,
}

impl Default for WeightedUtilityFunction {
    fn default() -> Self {
        Self {
            penalty_factor: 0.6,
        }
    }
}

impl WeightedUtilityFunction {
    /// Creates a combiner with a specific risk penalty factor.
    #[must_use]
    pub fn new(penalty_factor: f64) -> Self {
        Self { penalty_factor }
    }
}

impl UtilityFunction for WeightedUtilityFunction {
    fn combine(
        &self,
        _option: &contracts::DecisionOptionV1,
        inputs: &OptionScoreInputs,
        strategic_bonus: f64,
    ) -> UtilityScore {
        let base = (inputs.expected_value.value()
            + inputs.sampled_value.value()
            + inputs.thompson_score.value())
            / 3.0
            + strategic_bonus
            + inputs.aggregated_confidence.value() * 0.1;
        risk_penalized_utility(
            UtilityScore::new(base),
            inputs.aggregated_risk,
            self.penalty_factor,
        )
    }
}

/// Complete structured evaluation output produced by the decision engine.
#[derive(Debug, Clone, PartialEq)]
pub struct DecisionEvaluation {
    /// Ranked recommendation output.
    pub recommendation: DecisionRecommendationV1,
    /// Explicit policy-gate result.
    pub gate_result: contracts::PolicyGateResultV1,
    /// Structured audit record.
    pub audit_record: DecisionAuditRecordV1,
}

/// Decision-engine surface used by services and workflows.
pub trait DecisionEngine {
    /// Evaluates a decision context and returns the structured result.
    fn evaluate(&self, context: &DecisionContextV1) -> InstitutionalResult<DecisionEvaluation>;
}

/// Sink surface for persisting or forwarding decision audit records.
pub trait DecisionAuditSink {
    /// Records one decision audit record.
    fn record(&mut self, record: DecisionAuditRecordV1) -> InstitutionalResult<()>;
}

/// In-memory audit sink used for tests and service-local evaluation.
#[derive(Debug, Clone, Default)]
pub struct MemoryDecisionAuditSink {
    records: Vec<DecisionAuditRecordV1>,
}

impl MemoryDecisionAuditSink {
    /// Returns the recorded audit records.
    #[must_use]
    pub fn records(&self) -> &[DecisionAuditRecordV1] {
        &self.records
    }
}

impl DecisionAuditSink for MemoryDecisionAuditSink {
    fn record(&mut self, record: DecisionAuditRecordV1) -> InstitutionalResult<()> {
        self.records.push(record);
        Ok(())
    }
}

/// Baseline governed decision engine composed from deterministic analytics.
#[derive(Debug, Clone)]
pub struct BaselineDecisionEngine<L = NotConfiguredLearnedAdapter> {
    forecast_model: BaselineForecastModel,
    game_model: BaselineGameModel,
    utility_function: WeightedUtilityFunction,
    policy_gate: BaselinePolicyGate,
    learned_adapter: L,
    monte_carlo_iterations: usize,
}

impl Default for BaselineDecisionEngine<NotConfiguredLearnedAdapter> {
    fn default() -> Self {
        Self {
            forecast_model: BaselineForecastModel,
            game_model: BaselineGameModel,
            utility_function: WeightedUtilityFunction::default(),
            policy_gate: BaselinePolicyGate,
            learned_adapter: NotConfiguredLearnedAdapter,
            monte_carlo_iterations: 64,
        }
    }
}

impl<L> BaselineDecisionEngine<L> {
    /// Creates a baseline engine with a custom learned adapter.
    #[must_use]
    pub fn with_learned_adapter(learned_adapter: L) -> Self {
        Self {
            forecast_model: BaselineForecastModel,
            game_model: BaselineGameModel,
            utility_function: WeightedUtilityFunction::default(),
            policy_gate: BaselinePolicyGate,
            learned_adapter,
            monte_carlo_iterations: 64,
        }
    }
}

impl<L> BaselineDecisionEngine<L>
where
    L: ForecastAdapter + PolicyModelAdapter + RepresentationModel,
{
    fn operation_context(operation: &str) -> OperationContext {
        OperationContext::new("shared/decision-core", operation)
    }

    fn normalize_context(context: &DecisionContextV1) -> DecisionContextV1 {
        let mut normalized = context.clone();
        normalized
            .options
            .sort_by(|left, right| left.option_id.cmp(&right.option_id));
        for option in &mut normalized.options {
            option
                .expected_outcomes
                .sort_by(|left, right| left.estimate_id.cmp(&right.estimate_id));
            option
                .outcome_distribution
                .scenarios
                .sort_by(|left, right| left.label.cmp(&right.label));

            let total_probability = option
                .outcome_distribution
                .scenarios
                .iter()
                .map(|scenario| scenario.probability.max(0.0))
                .sum::<f64>();
            if total_probability > 0.0 {
                for scenario in &mut option.outcome_distribution.scenarios {
                    scenario.probability = (scenario.probability.max(0.0)) / total_probability;
                }
            }
        }
        normalized
    }

    fn check_constraints(
        context: &DecisionContextV1,
    ) -> BTreeMap<String, Vec<ConstraintViolation>> {
        let scorer = crate::WeightedExpectedValueScorer;
        context
            .options
            .iter()
            .map(|option| {
                let expected_value = scorer.score(option).value();
                let aggregated_confidence = aggregate_confidence(&[
                    ConfidenceScore::new(option.risk_assessment.confidence),
                    ConfidenceScore::new(
                        option
                            .expected_outcomes
                            .iter()
                            .map(|estimate| estimate.confidence)
                            .sum::<f64>()
                            / usize_to_f64(option.expected_outcomes.len().max(1)),
                    ),
                ])
                .value();
                let violations = context
                    .constraints
                    .iter()
                    .filter_map(|constraint| {
                        let violated = match constraint.kind {
                            ConstraintKindV1::MaxRiskScore => {
                                option.risk_assessment.risk_score > constraint.threshold
                            }
                            ConstraintKindV1::MinConfidenceScore => {
                                aggregated_confidence < constraint.threshold
                            }
                            ConstraintKindV1::MinExpectedUtility => {
                                expected_value < constraint.threshold
                            }
                            ConstraintKindV1::RequiresReversible => {
                                option.reversibility == ReversibilityClassV1::Irreversible
                            }
                            ConstraintKindV1::RequiresRollbackPlan => {
                                option.rollback_plan.trim().is_empty()
                            }
                        };
                        violated.then(|| ConstraintViolation {
                            constraint_id: constraint.constraint_id.clone(),
                            message: constraint.description.clone(),
                        })
                    })
                    .collect();
                (option.option_id.clone(), violations)
            })
            .collect()
    }
}

impl<L> DecisionEngine for BaselineDecisionEngine<L>
where
    L: ForecastAdapter + PolicyModelAdapter + RepresentationModel,
{
    fn evaluate(&self, context: &DecisionContextV1) -> InstitutionalResult<DecisionEvaluation> {
        let mut engine_trace = vec!["context_ingestion".to_owned()];
        if context.options.is_empty() {
            return Err(InstitutionalError::validation(
                Self::operation_context("evaluate"),
                "decision context must include at least one option",
            ));
        }

        let normalized = Self::normalize_context(context);
        engine_trace.push("option_normalization".to_owned());
        let constraint_violations = Self::check_constraints(&normalized);
        engine_trace.push("constraint_checking".to_owned());

        let learned_status = if normalized.requested_learned_support {
            match self.learned_adapter.forecast(&normalized) {
                Ok(_) => LearnedAdapterStatusV1::Applied,
                Err(LearnedAdapterError::NotConfigured) => LearnedAdapterStatusV1::NotConfigured,
            }
        } else {
            LearnedAdapterStatusV1::NotRequested
        };

        let thompson_results = crate::ThompsonSamplingBandit
            .sample_options(
                &normalized.options,
                normalized.evaluation_seed.saturating_add(1),
            )
            .into_iter()
            .map(|result: ThompsonSamplingResult| (result.option_id, result.sampled_probability))
            .collect::<BTreeMap<_, _>>();

        let probabilistic_scores = normalized
            .options
            .iter()
            .enumerate()
            .map(|(index, option)| {
                let seed = normalized
                    .evaluation_seed
                    .saturating_add(u64::try_from(index).unwrap_or(0));
                let evaluation =
                    self.forecast_model
                        .evaluate(option, seed, self.monte_carlo_iterations);
                (option.option_id.clone(), evaluation)
            })
            .collect::<BTreeMap<_, _>>();
        engine_trace.push("probabilistic_scoring".to_owned());

        let game_analysis = self.game_model.analyze(&normalized);
        engine_trace.push("game_analysis".to_owned());

        let ranked_options = normalized
            .options
            .iter()
            .map(|option| {
                let probabilistic = probabilistic_scores
                    .get(&option.option_id)
                    .expect("probabilistic evaluation must exist");
                let strategic_bonus = game_analysis
                    .as_ref()
                    .and_then(|analysis: &GameAnalysis| {
                        analysis
                            .strategic_bonus_by_option
                            .get(&option.option_id)
                            .copied()
                    })
                    .unwrap_or(0.0);
                let strategic_summary = game_analysis.as_ref().map_or_else(
                    || "No strategic interaction model supplied.".to_owned(),
                    |analysis| analysis.summary.clone(),
                );
                let final_utility = self.utility_function.combine(
                    option,
                    &OptionScoreInputs {
                        expected_value: probabilistic.expected_value,
                        sampled_value: probabilistic.monte_carlo.mean_utility,
                        thompson_score: *thompson_results
                            .get(&option.option_id)
                            .unwrap_or(&probabilistic.thompson_score),
                        aggregated_confidence: probabilistic.aggregated_confidence,
                        aggregated_risk: probabilistic.aggregated_risk,
                    },
                    strategic_bonus,
                );

                RankedDecisionOptionV1 {
                    option_id: option.option_id.clone(),
                    rank: 0,
                    expected_value: probabilistic.expected_value.value(),
                    sampled_value: probabilistic.monte_carlo.mean_utility.value(),
                    thompson_score: probabilistic.thompson_score.value(),
                    aggregated_confidence: probabilistic.aggregated_confidence.value(),
                    aggregated_risk: probabilistic.aggregated_risk.value(),
                    final_utility: final_utility.value(),
                    strategic_summary,
                    constraint_violations: constraint_violations
                        .get(&option.option_id)
                        .map(|violations| {
                            violations
                                .iter()
                                .map(|violation| violation.message.clone())
                                .collect()
                        })
                        .unwrap_or_default(),
                }
            })
            .collect::<Vec<_>>();
        engine_trace.push("utility_aggregation".to_owned());

        let mut ranked_options = ranked_options;
        ranked_options.sort_by(|left, right| {
            right
                .final_utility
                .total_cmp(&left.final_utility)
                .then_with(|| left.option_id.cmp(&right.option_id))
        });
        for (index, option) in ranked_options.iter_mut().enumerate() {
            option.rank = index + 1;
        }
        engine_trace.push("recommendation_selection".to_owned());

        let provisional_status = if ranked_options
            .first()
            .is_none_or(|option| !option.constraint_violations.is_empty())
        {
            RecommendationStatusV1::NonExecutable
        } else {
            RecommendationStatusV1::Executable
        };
        let mut gate_result =
            self.policy_gate
                .evaluate(&normalized, &ranked_options, provisional_status);
        engine_trace.push("policy_gating".to_owned());

        let mut final_status = match provisional_status {
            RecommendationStatusV1::NonExecutable => RecommendationStatusV1::NonExecutable,
            _ => match gate_result.verdict {
                PolicyGateVerdictV1::Passed => RecommendationStatusV1::Executable,
                PolicyGateVerdictV1::Advisory => RecommendationStatusV1::Advisory,
                PolicyGateVerdictV1::Rejected => RecommendationStatusV1::Rejected,
            },
        };

        if learned_status == LearnedAdapterStatusV1::NotConfigured
            && final_status == RecommendationStatusV1::Executable
        {
            gate_result.verdict = PolicyGateVerdictV1::Advisory;
            gate_result.executable = false;
            "Recommendation downgraded to advisory because learned adapters are not configured."
                .clone_into(&mut gate_result.rationale);
            final_status = RecommendationStatusV1::Advisory;
        }

        let selected_option_id = ranked_options
            .first()
            .map(|option| option.option_id.clone());
        let (risk, confidence, utility, rollback_plan, reversibility) = selected_option_id
            .as_ref()
            .and_then(|selected| {
                let ranked = ranked_options
                    .iter()
                    .find(|option| &option.option_id == selected)?;
                let source = normalized
                    .options
                    .iter()
                    .find(|option| &option.option_id == selected)?;
                Some((
                    ranked.aggregated_risk,
                    ranked.aggregated_confidence,
                    ranked.final_utility,
                    source.rollback_plan.clone(),
                    source.reversibility,
                ))
            })
            .unwrap_or((1.0, 0.0, 0.0, String::new(), normalized.reversibility));
        let uncertainty = 1.0 - confidence;
        let rationale = ranked_options.first().map_or_else(
            || "No viable option satisfied the decision inputs.".to_owned(),
            |option| {
                format!(
                    "Selected `{}` with utility {:.3}, risk {:.3}, and confidence {:.3}. {}",
                    option.option_id,
                    option.final_utility,
                    option.aggregated_risk,
                    option.aggregated_confidence,
                    option.strategic_summary
                )
            },
        );

        let recommendation = DecisionRecommendationV1 {
            recommendation_id: format!("recommendation::{}", normalized.decision_id),
            decision_id: normalized.decision_id.clone(),
            generated_at: normalized.created_at,
            status: final_status,
            selected_option_id: selected_option_id.clone(),
            ranked_options: ranked_options.clone(),
            rationale,
            confidence,
            uncertainty,
            risk,
            utility,
            reversibility,
            rollback_plan,
            policy_status: gate_result.verdict,
            provenance: normalized.provenance.clone(),
        };

        engine_trace.push("audit_record_creation".to_owned());
        let audit_record = DecisionAuditRecordV1 {
            audit_id: format!("audit::{}", normalized.decision_id),
            decision_id: normalized.decision_id.clone(),
            recorded_at: normalized.created_at,
            context: normalized.clone(),
            evaluated_options: ranked_options,
            selected_option_id,
            gate_result: gate_result.clone(),
            assumptions: normalized.provenance.assumptions.clone(),
            rejection_rationale: gate_result.rejection_reasons.clone(),
            learned_status,
            engine_trace,
            provenance: normalized.provenance.clone(),
        };

        Ok(DecisionEvaluation {
            recommendation,
            gate_result,
            audit_record,
        })
    }
}

fn usize_to_f64(value: usize) -> f64 {
    f64::from(u32::try_from(value).unwrap_or(u32::MAX))
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use contracts::{
        ApprovalRequirementV1, DecisionClassV1, DecisionConstraintV1, DecisionContextV1,
        DecisionOptionV1, DecisionStateV1, OutcomeDistributionV1, OutcomeEstimateV1,
        OutcomeScenarioV1, ProvenanceV1, ReversibilityClassV1, RiskAssessmentV1, RiskTierV1,
        UtilityBreakdownV1,
    };
    use identity::{ActorRef, DecisionId};

    use super::{BaselineDecisionEngine, DecisionEngine};

    fn context(requested_learned_support: bool) -> DecisionContextV1 {
        DecisionContextV1 {
            decision_id: DecisionId::from("decision::engine"),
            created_at: Utc
                .with_ymd_and_hms(2026, 3, 10, 12, 0, 0)
                .single()
                .expect("timestamp"),
            decision_class: DecisionClassV1::ReleaseRiskAssessment,
            state: DecisionStateV1::Pending,
            actor_ref: ActorRef("agent.strategist".to_owned()),
            subject: "release".to_owned(),
            objective: "Pick the release path".to_owned(),
            evaluation_seed: 19,
            risk_tier: RiskTierV1::Tier1,
            approval_requirement: ApprovalRequirementV1::None,
            policy_refs: vec!["policy.release.v1".to_owned()],
            reversibility: ReversibilityClassV1::GuardedRollback,
            requested_learned_support,
            options: vec![DecisionOptionV1 {
                option_id: "staged".to_owned(),
                title: "staged".to_owned(),
                description: "staged".to_owned(),
                expected_outcomes: vec![OutcomeEstimateV1 {
                    estimate_id: "estimate".to_owned(),
                    description: "steady".to_owned(),
                    probability: 0.8,
                    expected_utility: 0.8,
                    risk_adjustment: -0.1,
                    confidence: 0.8,
                    rationale: "historical".to_owned(),
                }],
                outcome_distribution: OutcomeDistributionV1 {
                    distribution_id: "distribution".to_owned(),
                    scenarios: vec![
                        OutcomeScenarioV1 {
                            label: "good".to_owned(),
                            probability: 0.8,
                            utility: 0.9,
                            risk: 0.2,
                        },
                        OutcomeScenarioV1 {
                            label: "bad".to_owned(),
                            probability: 0.2,
                            utility: 0.2,
                            risk: 0.5,
                        },
                    ],
                    expected_value: 0.76,
                    variance: 0.05,
                    downside_probability: 0.2,
                    rationale: "bounded".to_owned(),
                },
                risk_assessment: RiskAssessmentV1 {
                    risk_score: 0.3,
                    downside_probability: 0.2,
                    tail_risk_score: 0.3,
                    confidence: 0.8,
                    rationale: "controlled".to_owned(),
                    mitigation: "rollback".to_owned(),
                },
                utility_breakdown: UtilityBreakdownV1 {
                    value_score: 0.8,
                    resilience_score: 0.9,
                    compliance_score: 0.8,
                    cost_score: 0.6,
                    reversibility_score: 0.9,
                    strategic_fit_score: 0.8,
                    rationale: "balanced".to_owned(),
                },
                rollback_plan: "rollback".to_owned(),
                reversibility: ReversibilityClassV1::GuardedRollback,
                historical_successes: 7,
                historical_failures: 2,
            }],
            constraints: vec![DecisionConstraintV1 {
                constraint_id: "max-risk".to_owned(),
                kind: contracts::ConstraintKindV1::MaxRiskScore,
                description: "risk must stay bounded".to_owned(),
                threshold: 0.6,
                hard: true,
                rationale: "guardrail".to_owned(),
            }],
            normal_form_game: None,
            provenance: ProvenanceV1 {
                source_system: "tests".to_owned(),
                source_refs: Vec::new(),
                generated_by: "tests".to_owned(),
                assumptions: vec!["bounded scenarios".to_owned()],
            },
        }
    }

    #[test]
    fn learned_adapter_not_configured_downgrades_to_advisory() {
        let engine = BaselineDecisionEngine::default();
        let evaluation = engine.evaluate(&context(true)).expect("evaluation");
        assert_eq!(
            evaluation.recommendation.status,
            contracts::RecommendationStatusV1::Advisory
        );
        assert_eq!(
            evaluation.audit_record.learned_status,
            contracts::LearnedAdapterStatusV1::NotConfigured
        );
    }
}
