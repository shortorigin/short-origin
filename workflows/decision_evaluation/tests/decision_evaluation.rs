use chrono::{TimeZone, Utc};
use contracts::{
    ApprovalRequirementV1, DecisionClassV1, DecisionConstraintV1, DecisionContextV1,
    DecisionOptionV1, DecisionStateV1, OutcomeDistributionV1, OutcomeEstimateV1, OutcomeScenarioV1,
    ProvenanceV1, ReversibilityClassV1, RiskAssessmentV1, RiskTierV1, UtilityBreakdownV1,
};
use governance_service::GovernanceService;
use identity::{ActorRef, DecisionId};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

fn expected_json(path: &str) -> Value {
    let fixture = fs::read_to_string(fixture_path(path)).expect("read recommendation fixture");
    serde_json::from_str(&fixture).expect("deserialize fixture")
}

fn fixture_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn normalize_json(value: Value) -> Value {
    match value {
        Value::Array(values) => Value::Array(values.into_iter().map(normalize_json).collect()),
        Value::Object(entries) => Value::Object(
            entries
                .into_iter()
                .map(|(key, value)| (key, normalize_json(value)))
                .collect(),
        ),
        Value::Number(number) if !(number.is_i64() || number.is_u64()) => {
            let value = number.as_f64().expect("finite json number");
            let rounded = ((value * 1_000_000_000_000.0).round()) / 1_000_000_000_000.0;
            Value::Number(
                serde_json::Number::from_f64(if rounded == -0.0 { 0.0 } else { rounded })
                    .expect("rounded json number"),
            )
        }
        other => other,
    }
}

fn base_provenance() -> ProvenanceV1 {
    ProvenanceV1 {
        source_system: "tests".to_owned(),
        source_refs: vec!["fixture://decisioning".to_owned()],
        generated_by: "decision-evaluation-tests".to_owned(),
        assumptions: vec!["bounded scenarios".to_owned()],
    }
}

fn rollout_context(requested_learned_support: bool) -> DecisionContextV1 {
    DecisionContextV1 {
        decision_id: DecisionId::from(if requested_learned_support {
            "decision::rollout::advisory"
        } else {
            "decision::rollout"
        }),
        created_at: Utc
            .with_ymd_and_hms(2026, 3, 10, 16, 0, 0)
            .single()
            .expect("timestamp"),
        decision_class: DecisionClassV1::ReleaseRiskAssessment,
        state: DecisionStateV1::Pending,
        actor_ref: ActorRef("agent.strategist".to_owned()),
        subject: "release-candidate".to_owned(),
        objective: "Select a release rollout option".to_owned(),
        evaluation_seed: 101,
        risk_tier: RiskTierV1::Tier1,
        approval_requirement: ApprovalRequirementV1::None,
        policy_refs: vec!["policy.release.risk.v1".to_owned()],
        reversibility: ReversibilityClassV1::GuardedRollback,
        requested_learned_support,
        options: vec![
            DecisionOptionV1 {
                option_id: "staged".to_owned(),
                title: "Staged rollout".to_owned(),
                description: "Promote traffic gradually with rollback guardrails.".to_owned(),
                expected_outcomes: vec![OutcomeEstimateV1 {
                    estimate_id: "estimate-staged".to_owned(),
                    description: "Lower blast radius".to_owned(),
                    probability: 0.8,
                    expected_utility: 0.8,
                    risk_adjustment: -0.1,
                    confidence: 0.82,
                    rationale: "Prior controlled releases performed well.".to_owned(),
                }],
                outcome_distribution: OutcomeDistributionV1 {
                    distribution_id: "distribution-staged".to_owned(),
                    scenarios: vec![
                        OutcomeScenarioV1 {
                            label: "smooth".to_owned(),
                            probability: 0.8,
                            utility: 0.9,
                            risk: 0.2,
                        },
                        OutcomeScenarioV1 {
                            label: "rollback".to_owned(),
                            probability: 0.2,
                            utility: 0.2,
                            risk: 0.5,
                        },
                    ],
                    expected_value: 0.76,
                    variance: 0.05,
                    downside_probability: 0.2,
                    rationale: "Bounded rollout scenarios".to_owned(),
                },
                risk_assessment: RiskAssessmentV1 {
                    risk_score: 0.30,
                    downside_probability: 0.20,
                    tail_risk_score: 0.30,
                    confidence: 0.80,
                    rationale: "Traffic can be halted quickly.".to_owned(),
                    mitigation: "Canary checks and automated rollback".to_owned(),
                },
                utility_breakdown: UtilityBreakdownV1 {
                    value_score: 0.80,
                    resilience_score: 0.90,
                    compliance_score: 0.80,
                    cost_score: 0.60,
                    reversibility_score: 0.90,
                    strategic_fit_score: 0.80,
                    rationale: "Strong balance of speed and control.".to_owned(),
                },
                rollback_plan: "Pause promotion and redeploy prior manifest.".to_owned(),
                reversibility: ReversibilityClassV1::GuardedRollback,
                historical_successes: 8,
                historical_failures: 2,
            },
            DecisionOptionV1 {
                option_id: "immediate".to_owned(),
                title: "Immediate rollout".to_owned(),
                description: "Promote to full traffic immediately.".to_owned(),
                expected_outcomes: vec![OutcomeEstimateV1 {
                    estimate_id: "estimate-immediate".to_owned(),
                    description: "Faster delivery".to_owned(),
                    probability: 0.9,
                    expected_utility: 0.9,
                    risk_adjustment: -0.3,
                    confidence: 0.70,
                    rationale: "Delivery is faster but less bounded.".to_owned(),
                }],
                outcome_distribution: OutcomeDistributionV1 {
                    distribution_id: "distribution-immediate".to_owned(),
                    scenarios: vec![
                        OutcomeScenarioV1 {
                            label: "fast".to_owned(),
                            probability: 0.7,
                            utility: 0.95,
                            risk: 0.5,
                        },
                        OutcomeScenarioV1 {
                            label: "incident".to_owned(),
                            probability: 0.3,
                            utility: 0.1,
                            risk: 0.9,
                        },
                    ],
                    expected_value: 0.695,
                    variance: 0.20,
                    downside_probability: 0.3,
                    rationale: "Faster but more variable".to_owned(),
                },
                risk_assessment: RiskAssessmentV1 {
                    risk_score: 0.70,
                    downside_probability: 0.30,
                    tail_risk_score: 0.80,
                    confidence: 0.70,
                    rationale: "Higher blast radius if the release regresses.".to_owned(),
                    mitigation: "Manual rollback only".to_owned(),
                },
                utility_breakdown: UtilityBreakdownV1 {
                    value_score: 0.90,
                    resilience_score: 0.50,
                    compliance_score: 0.70,
                    cost_score: 0.70,
                    reversibility_score: 0.50,
                    strategic_fit_score: 0.80,
                    rationale: "Maximizes delivery speed".to_owned(),
                },
                rollback_plan: "Redeploy prior manifest after incident response review.".to_owned(),
                reversibility: ReversibilityClassV1::GuardedRollback,
                historical_successes: 9,
                historical_failures: 3,
            },
        ],
        constraints: vec![DecisionConstraintV1 {
            constraint_id: "risk-threshold".to_owned(),
            kind: contracts::ConstraintKindV1::MaxRiskScore,
            description: "Risk must stay below 0.8".to_owned(),
            threshold: 0.8,
            hard: true,
            rationale: "Release guardrail".to_owned(),
        }],
        normal_form_game: None,
        provenance: base_provenance(),
    }
}

fn policy_rejection_context() -> DecisionContextV1 {
    DecisionContextV1 {
        decision_id: DecisionId::from("decision::policy-rejection"),
        created_at: Utc
            .with_ymd_and_hms(2026, 3, 10, 17, 0, 0)
            .single()
            .expect("timestamp"),
        decision_class: DecisionClassV1::PolicyConstrainedActionSelection,
        state: DecisionStateV1::Pending,
        actor_ref: ActorRef("agent.strategist".to_owned()),
        subject: "capacity-shift".to_owned(),
        objective: "Select an emergency capacity response".to_owned(),
        evaluation_seed: 303,
        risk_tier: RiskTierV1::Tier1,
        approval_requirement: ApprovalRequirementV1::None,
        policy_refs: vec!["policy.capacity.risk.v1".to_owned()],
        reversibility: ReversibilityClassV1::GuardedRollback,
        requested_learned_support: false,
        options: vec![DecisionOptionV1 {
            option_id: "aggressive".to_owned(),
            title: "Aggressive reallocation".to_owned(),
            description: "Shift most capacity immediately to the new path.".to_owned(),
            expected_outcomes: vec![OutcomeEstimateV1 {
                estimate_id: "estimate-aggressive".to_owned(),
                description: "Max utility".to_owned(),
                probability: 0.95,
                expected_utility: 1.1,
                risk_adjustment: -0.4,
                confidence: 0.75,
                rationale: "High upside if it works".to_owned(),
            }],
            outcome_distribution: OutcomeDistributionV1 {
                distribution_id: "distribution-aggressive".to_owned(),
                scenarios: vec![
                    OutcomeScenarioV1 {
                        label: "win".to_owned(),
                        probability: 0.8,
                        utility: 1.0,
                        risk: 0.7,
                    },
                    OutcomeScenarioV1 {
                        label: "major-incident".to_owned(),
                        probability: 0.2,
                        utility: 0.3,
                        risk: 0.95,
                    },
                ],
                expected_value: 0.86,
                variance: 0.18,
                downside_probability: 0.2,
                rationale: "High-risk high-upside".to_owned(),
            },
            risk_assessment: RiskAssessmentV1 {
                risk_score: 0.92,
                downside_probability: 0.35,
                tail_risk_score: 0.95,
                confidence: 0.72,
                rationale: "Crosses the policy threshold".to_owned(),
                mitigation: "Limited mitigation available".to_owned(),
            },
            utility_breakdown: UtilityBreakdownV1 {
                value_score: 0.98,
                resilience_score: 0.40,
                compliance_score: 0.60,
                cost_score: 0.85,
                reversibility_score: 0.40,
                strategic_fit_score: 0.95,
                rationale: "High speed, low resilience".to_owned(),
            },
            rollback_plan: "Manual rollback after incident review.".to_owned(),
            reversibility: ReversibilityClassV1::GuardedRollback,
            historical_successes: 14,
            historical_failures: 4,
        }],
        constraints: vec![DecisionConstraintV1 {
            constraint_id: "risk-threshold".to_owned(),
            kind: contracts::ConstraintKindV1::MaxRiskScore,
            description: "Risk must stay below 0.95".to_owned(),
            threshold: 0.95,
            hard: true,
            rationale: "Only policy gate should reject here".to_owned(),
        }],
        normal_form_game: None,
        provenance: base_provenance(),
    }
}

#[test]
fn rollout_context_selects_staged_option_under_risk_constraints() {
    let mut service = GovernanceService::default();
    let report =
        decision_evaluation::execute(&mut service, rollout_context(false)).expect("workflow");
    let expected = expected_json(
        "../../testing/fixtures/decisioning/run-2026-03-10/rollout_recommendation.json",
    );

    assert_eq!(
        normalize_json(
            serde_json::to_value(&report.evaluation.recommendation)
                .expect("serialize recommendation"),
        ),
        normalize_json(expected)
    );
}

#[test]
fn policy_gate_rejects_high_utility_option_that_violates_policy() {
    let mut service = GovernanceService::default();
    let report =
        decision_evaluation::execute(&mut service, policy_rejection_context()).expect("workflow");
    let expected = expected_json(
        "../../testing/fixtures/decisioning/run-2026-03-10/policy_rejection_recommendation.json",
    );

    assert_eq!(
        normalize_json(
            serde_json::to_value(&report.evaluation.recommendation)
                .expect("serialize recommendation"),
        ),
        normalize_json(expected)
    );
}

#[test]
fn workflow_produces_ranked_recommendations_with_audit_output() {
    let mut service = GovernanceService::default();
    let report =
        decision_evaluation::execute(&mut service, rollout_context(false)).expect("workflow");
    let expected =
        expected_json("../../testing/fixtures/decisioning/run-2026-03-10/ranked_audit.json");

    assert_eq!(
        normalize_json(
            serde_json::to_value(&report.evaluation.audit_record).expect("serialize audit")
        ),
        normalize_json(expected)
    );
}

#[test]
fn workflow_returns_advisory_when_learned_adapters_are_unconfigured() {
    let mut service = GovernanceService::default();
    let report =
        decision_evaluation::execute(&mut service, rollout_context(true)).expect("workflow");
    let expected = expected_json(
        "../../testing/fixtures/decisioning/run-2026-03-10/advisory_recommendation.json",
    );

    assert_eq!(
        normalize_json(
            serde_json::to_value(&report.evaluation.recommendation)
                .expect("serialize recommendation"),
        ),
        normalize_json(expected)
    );
}
