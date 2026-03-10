use contracts::{
    ApprovalRequirementV1, DecisionContextV1, PolicyGateResultV1, PolicyGateVerdictV1,
    RankedDecisionOptionV1, RecommendationStatusV1, ReversibilityClassV1, RiskTierV1,
};

/// Policy-gate surface used by the decision engine after ranking alternatives.
pub trait PolicyGate {
    /// Evaluates the ranked recommendation and returns the authoritative gate result.
    fn evaluate(
        &self,
        context: &DecisionContextV1,
        ranked_options: &[RankedDecisionOptionV1],
        provisional_status: RecommendationStatusV1,
    ) -> PolicyGateResultV1;
}

/// Baseline policy gate enforcing explicit approval and risk-tier rules.
#[derive(Debug, Clone, Copy, Default)]
pub struct BaselinePolicyGate;

impl BaselinePolicyGate {
    fn required_approval_actions(requirement: ApprovalRequirementV1) -> Vec<String> {
        match requirement {
            ApprovalRequirementV1::None => Vec::new(),
            ApprovalRequirementV1::DomainOwner => vec!["domain_owner_approval".to_owned()],
            ApprovalRequirementV1::DualApproval => vec!["dual_approval".to_owned()],
            ApprovalRequirementV1::InstitutionalCouncil => {
                vec!["institutional_council_approval".to_owned()]
            }
        }
    }

    fn max_executable_risk(tier: RiskTierV1) -> f64 {
        match tier {
            RiskTierV1::Tier0 => 0.85,
            RiskTierV1::Tier1 => 0.70,
            RiskTierV1::Tier2 => 0.55,
            RiskTierV1::Tier3 => 0.40,
        }
    }
}

impl PolicyGate for BaselinePolicyGate {
    fn evaluate(
        &self,
        context: &DecisionContextV1,
        ranked_options: &[RankedDecisionOptionV1],
        provisional_status: RecommendationStatusV1,
    ) -> PolicyGateResultV1 {
        let required_approval_actions =
            Self::required_approval_actions(context.approval_requirement);
        let mut rejection_reasons = Vec::new();
        let mut rationale = "Policy gate passed.".to_owned();
        let mut verdict = PolicyGateVerdictV1::Passed;

        let selected = ranked_options.first();
        if provisional_status == RecommendationStatusV1::NonExecutable {
            verdict = PolicyGateVerdictV1::Rejected;
            "Constraint failures prevent execution.".clone_into(&mut rationale);
            rejection_reasons.push("constraint violations detected".to_owned());
        } else if let Some(selected_option) = selected {
            if selected_option.aggregated_risk > Self::max_executable_risk(context.risk_tier) {
                verdict = PolicyGateVerdictV1::Rejected;
                "Risk tier threshold exceeded.".clone_into(&mut rationale);
                rejection_reasons.push(format!(
                    "aggregated risk {:.3} exceeds tier threshold",
                    selected_option.aggregated_risk
                ));
            } else if context.risk_tier >= RiskTierV1::Tier2 {
                let selected_reversibility = context
                    .options
                    .iter()
                    .find(|option| option.option_id == selected_option.option_id)
                    .map_or(context.reversibility, |option| option.reversibility);
                if selected_reversibility == ReversibilityClassV1::Irreversible {
                    verdict = PolicyGateVerdictV1::Rejected;
                    "High-tier decisions must remain reversible.".clone_into(&mut rationale);
                    rejection_reasons
                        .push("irreversible option not allowed at this risk tier".to_owned());
                }
            }
        } else {
            verdict = PolicyGateVerdictV1::Rejected;
            "No ranked option available for gating.".clone_into(&mut rationale);
            rejection_reasons.push("no ranked option produced".to_owned());
        }

        if verdict == PolicyGateVerdictV1::Passed && !required_approval_actions.is_empty() {
            verdict = PolicyGateVerdictV1::Advisory;
            "Recommendation remains advisory until required approvals are recorded."
                .clone_into(&mut rationale);
        }

        PolicyGateResultV1 {
            gate_id: format!("gate::{}", context.decision_id),
            decision_id: context.decision_id.clone(),
            evaluated_at: context.created_at,
            verdict,
            executable: verdict == PolicyGateVerdictV1::Passed,
            required_approval_actions,
            rationale,
            rejection_reasons,
            policy_refs: context.policy_refs.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use contracts::{
        ApprovalRequirementV1, DecisionClassV1, DecisionContextV1, DecisionOptionV1,
        DecisionStateV1, OutcomeDistributionV1, ProvenanceV1, RankedDecisionOptionV1,
        RecommendationStatusV1, ReversibilityClassV1, RiskAssessmentV1, RiskTierV1,
        UtilityBreakdownV1,
    };
    use identity::{ActorRef, DecisionId};

    use super::{BaselinePolicyGate, PolicyGate};

    fn context() -> DecisionContextV1 {
        DecisionContextV1 {
            decision_id: DecisionId::from("decision::policy"),
            created_at: Utc
                .with_ymd_and_hms(2026, 3, 10, 12, 0, 0)
                .single()
                .expect("timestamp"),
            decision_class: DecisionClassV1::PolicyConstrainedActionSelection,
            state: DecisionStateV1::Pending,
            actor_ref: ActorRef("agent.strategist".to_owned()),
            subject: "change".to_owned(),
            objective: "Choose governed action".to_owned(),
            evaluation_seed: 5,
            risk_tier: RiskTierV1::Tier1,
            approval_requirement: ApprovalRequirementV1::None,
            policy_refs: vec!["policy.governance.v1".to_owned()],
            reversibility: ReversibilityClassV1::Reversible,
            requested_learned_support: false,
            options: vec![DecisionOptionV1 {
                option_id: "safe".to_owned(),
                title: "safe".to_owned(),
                description: "safe".to_owned(),
                expected_outcomes: Vec::new(),
                outcome_distribution: OutcomeDistributionV1 {
                    distribution_id: "distribution".to_owned(),
                    scenarios: Vec::new(),
                    expected_value: 0.0,
                    variance: 0.0,
                    downside_probability: 0.0,
                    rationale: "n/a".to_owned(),
                },
                risk_assessment: RiskAssessmentV1 {
                    risk_score: 0.2,
                    downside_probability: 0.2,
                    tail_risk_score: 0.2,
                    confidence: 0.8,
                    rationale: "n/a".to_owned(),
                    mitigation: "n/a".to_owned(),
                },
                utility_breakdown: UtilityBreakdownV1 {
                    value_score: 0.8,
                    resilience_score: 0.8,
                    compliance_score: 0.8,
                    cost_score: 0.8,
                    reversibility_score: 0.8,
                    strategic_fit_score: 0.8,
                    rationale: "n/a".to_owned(),
                },
                rollback_plan: "rollback".to_owned(),
                reversibility: ReversibilityClassV1::Reversible,
                historical_successes: 1,
                historical_failures: 1,
            }],
            constraints: Vec::new(),
            normal_form_game: None,
            provenance: ProvenanceV1 {
                source_system: "tests".to_owned(),
                source_refs: Vec::new(),
                generated_by: "tests".to_owned(),
                assumptions: Vec::new(),
            },
        }
    }

    #[test]
    fn policy_gate_passes_low_risk_without_additional_approval() {
        let gate = BaselinePolicyGate.evaluate(
            &context(),
            &[RankedDecisionOptionV1 {
                option_id: "safe".to_owned(),
                rank: 1,
                expected_value: 0.7,
                sampled_value: 0.7,
                thompson_score: 0.7,
                aggregated_confidence: 0.8,
                aggregated_risk: 0.3,
                final_utility: 0.7,
                strategic_summary: "n/a".to_owned(),
                constraint_violations: Vec::new(),
            }],
            RecommendationStatusV1::Executable,
        );

        assert!(gate.executable);
    }

    #[test]
    fn policy_gate_rejects_high_risk_option() {
        let gate = BaselinePolicyGate.evaluate(
            &context(),
            &[RankedDecisionOptionV1 {
                option_id: "safe".to_owned(),
                rank: 1,
                expected_value: 0.9,
                sampled_value: 0.9,
                thompson_score: 0.9,
                aggregated_confidence: 0.8,
                aggregated_risk: 0.9,
                final_utility: 0.9,
                strategic_summary: "n/a".to_owned(),
                constraint_violations: Vec::new(),
            }],
            RecommendationStatusV1::Executable,
        );

        assert!(!gate.executable);
        assert_eq!(gate.verdict, contracts::PolicyGateVerdictV1::Rejected);
    }
}
