use chrono::{TimeZone, Utc};
use contracts::{
    ApprovalRequirementV1, ConstraintKindV1, DecisionAuditRecordV1, DecisionClassV1,
    DecisionConstraintV1, DecisionContextV1, DecisionOptionV1, DecisionRecommendationV1,
    DecisionStateV1, LearnedAdapterStatusV1, NormalFormGameV1, OutcomeDistributionV1,
    OutcomeEstimateV1, OutcomeScenarioV1, PayoffCellV1, PolicyGateResultV1, PolicyGateVerdictV1,
    ProvenanceV1, RankedDecisionOptionV1, RecommendationStatusV1, ReversibilityClassV1,
    RiskAssessmentV1, RiskTierV1, UtilityBreakdownV1,
};
use identity::{ActorRef, DecisionId};

fn sample_provenance() -> ProvenanceV1 {
    ProvenanceV1 {
        source_system: "testing".to_owned(),
        source_refs: vec!["fixture://decision".to_owned()],
        generated_by: "decision-tests".to_owned(),
        assumptions: vec!["bounded scenario set".to_owned()],
    }
}

fn sample_context() -> DecisionContextV1 {
    DecisionContextV1 {
        decision_id: DecisionId::from("decision::sample"),
        created_at: Utc
            .with_ymd_and_hms(2026, 3, 10, 12, 0, 0)
            .single()
            .expect("valid timestamp"),
        decision_class: DecisionClassV1::ReleaseRiskAssessment,
        state: DecisionStateV1::Pending,
        actor_ref: ActorRef("agent.strategist".to_owned()),
        subject: "release-candidate".to_owned(),
        objective: "Choose the safest rollout option".to_owned(),
        evaluation_seed: 42,
        risk_tier: RiskTierV1::Tier2,
        approval_requirement: ApprovalRequirementV1::DualApproval,
        policy_refs: vec!["policy.release.risk.v1".to_owned()],
        reversibility: ReversibilityClassV1::GuardedRollback,
        requested_learned_support: true,
        options: vec![DecisionOptionV1 {
            option_id: "staged".to_owned(),
            title: "Staged rollout".to_owned(),
            description: "Roll out gradually".to_owned(),
            expected_outcomes: vec![OutcomeEstimateV1 {
                estimate_id: "estimate-1".to_owned(),
                description: "Lower blast radius".to_owned(),
                probability: 0.7,
                expected_utility: 0.8,
                risk_adjustment: -0.1,
                confidence: 0.8,
                rationale: "Prior release history".to_owned(),
            }],
            outcome_distribution: OutcomeDistributionV1 {
                distribution_id: "distribution-1".to_owned(),
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
                        utility: 0.1,
                        risk: 0.6,
                    },
                ],
                expected_value: 0.74,
                variance: 0.08,
                downside_probability: 0.2,
                rationale: "Derived from bounded scenarios".to_owned(),
            },
            risk_assessment: RiskAssessmentV1 {
                risk_score: 0.3,
                downside_probability: 0.2,
                tail_risk_score: 0.4,
                confidence: 0.75,
                rationale: "Staged deployment is easier to contain".to_owned(),
                mitigation: "Canary and rollback guardrails".to_owned(),
            },
            utility_breakdown: UtilityBreakdownV1 {
                value_score: 0.7,
                resilience_score: 0.9,
                compliance_score: 0.8,
                cost_score: 0.6,
                reversibility_score: 0.9,
                strategic_fit_score: 0.8,
                rationale: "Balanced operational trade-off".to_owned(),
            },
            rollback_plan: "Pause rollout and revert manifest".to_owned(),
            reversibility: ReversibilityClassV1::GuardedRollback,
            historical_successes: 8,
            historical_failures: 2,
        }],
        constraints: vec![DecisionConstraintV1 {
            constraint_id: "constraint-risk".to_owned(),
            kind: ConstraintKindV1::MaxRiskScore,
            description: "Risk score must stay below 0.6".to_owned(),
            threshold: 0.6,
            hard: true,
            rationale: "Release risk guardrail".to_owned(),
        }],
        normal_form_game: Some(NormalFormGameV1 {
            focal_actor: "origin".to_owned(),
            counterpart_actor: "operators".to_owned(),
            focal_strategies: vec!["staged".to_owned()],
            counterpart_strategies: vec!["watch".to_owned()],
            payoff_cells: vec![PayoffCellV1 {
                focal_strategy: "staged".to_owned(),
                counterpart_strategy: "watch".to_owned(),
                focal_payoff: 0.8,
                counterpart_payoff: 0.7,
            }],
        }),
        provenance: sample_provenance(),
    }
}

#[test]
fn decision_contracts_round_trip_through_json() {
    let context = sample_context();
    let gate = PolicyGateResultV1 {
        gate_id: "gate::sample".to_owned(),
        decision_id: context.decision_id.clone(),
        evaluated_at: context.created_at,
        verdict: PolicyGateVerdictV1::Advisory,
        executable: false,
        required_approval_actions: vec!["dual_approval".to_owned()],
        rationale: "Human approval remains required".to_owned(),
        rejection_reasons: Vec::new(),
        policy_refs: context.policy_refs.clone(),
    };
    let ranked_options = vec![RankedDecisionOptionV1 {
        option_id: "staged".to_owned(),
        rank: 1,
        expected_value: 0.74,
        sampled_value: 0.71,
        thompson_score: 0.81,
        aggregated_confidence: 0.77,
        aggregated_risk: 0.31,
        final_utility: 0.68,
        strategic_summary: "No dominated strategy detected.".to_owned(),
        constraint_violations: Vec::new(),
    }];
    let recommendation = DecisionRecommendationV1 {
        recommendation_id: "recommendation::sample".to_owned(),
        decision_id: context.decision_id.clone(),
        generated_at: context.created_at,
        status: RecommendationStatusV1::Advisory,
        selected_option_id: Some("staged".to_owned()),
        ranked_options: ranked_options.clone(),
        rationale: "Staged rollout best balances utility and risk.".to_owned(),
        confidence: 0.77,
        uncertainty: 0.23,
        risk: 0.31,
        utility: 0.68,
        reversibility: ReversibilityClassV1::GuardedRollback,
        rollback_plan: "Pause rollout and revert manifest".to_owned(),
        policy_status: PolicyGateVerdictV1::Advisory,
        provenance: sample_provenance(),
    };
    let audit = DecisionAuditRecordV1 {
        audit_id: "audit::sample".to_owned(),
        decision_id: context.decision_id.clone(),
        recorded_at: context.created_at,
        context: context.clone(),
        evaluated_options: ranked_options,
        selected_option_id: Some("staged".to_owned()),
        gate_result: gate,
        assumptions: context.provenance.assumptions.clone(),
        rejection_rationale: Vec::new(),
        learned_status: LearnedAdapterStatusV1::NotConfigured,
        engine_trace: vec![
            "context_ingestion".to_owned(),
            "recommendation_selection".to_owned(),
        ],
        provenance: sample_provenance(),
    };

    let encoded_context = serde_json::to_string(&context).expect("serialize context");
    let decoded_context: DecisionContextV1 =
        serde_json::from_str(&encoded_context).expect("deserialize context");
    assert_eq!(decoded_context, context);

    let encoded_recommendation =
        serde_json::to_string(&recommendation).expect("serialize recommendation");
    let decoded_recommendation: DecisionRecommendationV1 =
        serde_json::from_str(&encoded_recommendation).expect("deserialize recommendation");
    assert_eq!(decoded_recommendation, recommendation);

    let encoded_audit = serde_json::to_string(&audit).expect("serialize audit");
    let decoded_audit: DecisionAuditRecordV1 =
        serde_json::from_str(&encoded_audit).expect("deserialize audit");
    assert_eq!(decoded_audit, audit);
}
