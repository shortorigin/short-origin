use std::collections::{BTreeMap, BTreeSet};

use contracts::DecisionContextV1;

/// Enumerates the focal and counterpart best responses in a bounded game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrategicBestResponses {
    /// Best focal response for each counterpart strategy.
    pub focal: BTreeMap<String, String>,
    /// Best counterpart response for each focal strategy.
    pub counterpart: BTreeMap<String, String>,
}

/// Represents one pure-strategy equilibrium detected in a bounded normal-form game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PureStrategyEquilibrium {
    /// Focal strategy participating in the equilibrium.
    pub focal_strategy: String,
    /// Counterpart strategy participating in the equilibrium.
    pub counterpart_strategy: String,
}

/// Summary of the strategic analysis used by the decision engine.
#[derive(Debug, Clone, PartialEq)]
pub struct GameAnalysis {
    /// Dominated focal strategies removed by elimination.
    pub dominated_focal_strategies: Vec<String>,
    /// Best responses for the focal and counterpart actors.
    pub best_responses: StrategicBestResponses,
    /// Pure-strategy equilibria discovered in the game.
    pub pure_strategy_equilibria: Vec<PureStrategyEquilibrium>,
    /// Option-level strategic bonuses used during utility aggregation.
    pub strategic_bonus_by_option: BTreeMap<String, f64>,
    /// Summary text suitable for recommendation rationale.
    pub summary: String,
}

/// Strategy-analysis surface consumed by the decision engine.
pub trait GameModel {
    /// Analyzes the optional normal-form game embedded in the decision context.
    fn analyze(&self, context: &DecisionContextV1) -> Option<GameAnalysis>;
}

/// Baseline finite-game analyzer with dominated-strategy elimination and equilibrium detection.
#[derive(Debug, Clone, Copy, Default)]
pub struct BaselineGameModel;

impl BaselineGameModel {
    fn focal_payoff(
        matrix: &BTreeMap<(String, String), (f64, f64)>,
        focal_strategy: &str,
        counterpart_strategy: &str,
    ) -> Option<f64> {
        matrix
            .get(&(focal_strategy.to_owned(), counterpart_strategy.to_owned()))
            .map(|payoffs| payoffs.0)
    }

    fn counterpart_payoff(
        matrix: &BTreeMap<(String, String), (f64, f64)>,
        focal_strategy: &str,
        counterpart_strategy: &str,
    ) -> Option<f64> {
        matrix
            .get(&(focal_strategy.to_owned(), counterpart_strategy.to_owned()))
            .map(|payoffs| payoffs.1)
    }

    fn payoff_matrix(
        context: &DecisionContextV1,
    ) -> Option<BTreeMap<(String, String), (f64, f64)>> {
        context.normal_form_game.as_ref().map(|game| {
            game.payoff_cells
                .iter()
                .map(|cell| {
                    (
                        (
                            cell.focal_strategy.clone(),
                            cell.counterpart_strategy.clone(),
                        ),
                        (cell.focal_payoff, cell.counterpart_payoff),
                    )
                })
                .collect()
        })
    }

    fn dominated_strategies(
        focal_strategies: &[String],
        counterpart_strategies: &[String],
        matrix: &BTreeMap<(String, String), (f64, f64)>,
    ) -> Vec<String> {
        let mut dominated = BTreeSet::new();
        for candidate in focal_strategies {
            for challenger in focal_strategies {
                if candidate == challenger {
                    continue;
                }

                let mut challenger_always_better = true;
                let mut challenger_strictly_better_once = false;
                for counterpart in counterpart_strategies {
                    let candidate_payoff = Self::focal_payoff(matrix, candidate, counterpart)
                        .unwrap_or(f64::NEG_INFINITY);
                    let challenger_payoff = Self::focal_payoff(matrix, challenger, counterpart)
                        .unwrap_or(f64::NEG_INFINITY);
                    if challenger_payoff < candidate_payoff {
                        challenger_always_better = false;
                        break;
                    }
                    if challenger_payoff > candidate_payoff {
                        challenger_strictly_better_once = true;
                    }
                }

                if challenger_always_better && challenger_strictly_better_once {
                    dominated.insert(candidate.clone());
                }
            }
        }

        dominated.into_iter().collect()
    }

    fn best_responses(
        focal_strategies: &[String],
        counterpart_strategies: &[String],
        matrix: &BTreeMap<(String, String), (f64, f64)>,
    ) -> StrategicBestResponses {
        let focal = counterpart_strategies
            .iter()
            .filter_map(|counterpart| {
                focal_strategies
                    .iter()
                    .max_by(|left, right| {
                        Self::focal_payoff(matrix, left, counterpart)
                            .unwrap_or(f64::NEG_INFINITY)
                            .total_cmp(
                                &Self::focal_payoff(matrix, right, counterpart)
                                    .unwrap_or(f64::NEG_INFINITY),
                            )
                    })
                    .cloned()
                    .map(|best| (counterpart.clone(), best))
            })
            .collect();

        let counterpart = focal_strategies
            .iter()
            .filter_map(|focal_strategy| {
                counterpart_strategies
                    .iter()
                    .max_by(|left, right| {
                        Self::counterpart_payoff(matrix, focal_strategy, left)
                            .unwrap_or(f64::NEG_INFINITY)
                            .total_cmp(
                                &Self::counterpart_payoff(matrix, focal_strategy, right)
                                    .unwrap_or(f64::NEG_INFINITY),
                            )
                    })
                    .cloned()
                    .map(|best| (focal_strategy.clone(), best))
            })
            .collect();

        StrategicBestResponses { focal, counterpart }
    }

    fn pure_equilibria(
        focal_strategies: &[String],
        counterpart_strategies: &[String],
        best_responses: &StrategicBestResponses,
    ) -> Vec<PureStrategyEquilibrium> {
        let mut equilibria = Vec::new();
        for focal_strategy in focal_strategies {
            for counterpart_strategy in counterpart_strategies {
                let focal_best = best_responses.focal.get(counterpart_strategy);
                let counterpart_best = best_responses.counterpart.get(focal_strategy);
                if focal_best == Some(focal_strategy)
                    && counterpart_best == Some(counterpart_strategy)
                {
                    equilibria.push(PureStrategyEquilibrium {
                        focal_strategy: focal_strategy.clone(),
                        counterpart_strategy: counterpart_strategy.clone(),
                    });
                }
            }
        }
        equilibria
    }
}

impl GameModel for BaselineGameModel {
    fn analyze(&self, context: &DecisionContextV1) -> Option<GameAnalysis> {
        let game = context.normal_form_game.as_ref()?;
        let matrix = Self::payoff_matrix(context)?;
        let dominated = Self::dominated_strategies(
            &game.focal_strategies,
            &game.counterpart_strategies,
            &matrix,
        );
        let best_responses = Self::best_responses(
            &game.focal_strategies,
            &game.counterpart_strategies,
            &matrix,
        );
        let equilibria = Self::pure_equilibria(
            &game.focal_strategies,
            &game.counterpart_strategies,
            &best_responses,
        );
        let equilibrium_strategies = equilibria
            .iter()
            .map(|entry| entry.focal_strategy.clone())
            .collect::<BTreeSet<_>>();

        let strategic_bonus_by_option = game
            .focal_strategies
            .iter()
            .map(|strategy| {
                let mut bonus = 0.0;
                if dominated.contains(strategy) {
                    bonus -= 0.20;
                }
                if equilibrium_strategies.contains(strategy) {
                    bonus += 0.10;
                }
                if best_responses.focal.values().any(|best| best == strategy) {
                    bonus += 0.05;
                }
                (strategy.clone(), bonus)
            })
            .collect();

        let summary = format!(
            "{} dominated focal strategies; {} pure equilibria identified.",
            dominated.len(),
            equilibria.len()
        );

        Some(GameAnalysis {
            dominated_focal_strategies: dominated,
            best_responses,
            pure_strategy_equilibria: equilibria,
            strategic_bonus_by_option,
            summary,
        })
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use contracts::{
        ApprovalRequirementV1, DecisionClassV1, DecisionContextV1, DecisionOptionV1,
        DecisionStateV1, NormalFormGameV1, OutcomeDistributionV1, PayoffCellV1, ProvenanceV1,
        ReversibilityClassV1, RiskAssessmentV1, RiskTierV1, UtilityBreakdownV1,
    };
    use identity::{ActorRef, DecisionId};

    use super::{BaselineGameModel, GameModel};

    fn context() -> DecisionContextV1 {
        DecisionContextV1 {
            decision_id: DecisionId::from("decision::game"),
            created_at: Utc
                .with_ymd_and_hms(2026, 3, 10, 12, 0, 0)
                .single()
                .expect("timestamp"),
            decision_class: DecisionClassV1::StrategicPrioritization,
            state: DecisionStateV1::Pending,
            actor_ref: ActorRef("agent.strategist".to_owned()),
            subject: "priorities".to_owned(),
            objective: "Pick the strongest strategy".to_owned(),
            evaluation_seed: 3,
            risk_tier: RiskTierV1::Tier1,
            approval_requirement: ApprovalRequirementV1::None,
            policy_refs: vec!["policy.strategy.v1".to_owned()],
            reversibility: ReversibilityClassV1::Reversible,
            requested_learned_support: false,
            options: vec![
                DecisionOptionV1 {
                    option_id: "a".to_owned(),
                    title: "A".to_owned(),
                    description: "A".to_owned(),
                    expected_outcomes: Vec::new(),
                    outcome_distribution: OutcomeDistributionV1 {
                        distribution_id: "d".to_owned(),
                        scenarios: Vec::new(),
                        expected_value: 0.0,
                        variance: 0.0,
                        downside_probability: 0.0,
                        rationale: "n/a".to_owned(),
                    },
                    risk_assessment: RiskAssessmentV1 {
                        risk_score: 0.2,
                        downside_probability: 0.1,
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
                },
                DecisionOptionV1 {
                    option_id: "b".to_owned(),
                    title: "B".to_owned(),
                    description: "B".to_owned(),
                    expected_outcomes: Vec::new(),
                    outcome_distribution: OutcomeDistributionV1 {
                        distribution_id: "d2".to_owned(),
                        scenarios: Vec::new(),
                        expected_value: 0.0,
                        variance: 0.0,
                        downside_probability: 0.0,
                        rationale: "n/a".to_owned(),
                    },
                    risk_assessment: RiskAssessmentV1 {
                        risk_score: 0.2,
                        downside_probability: 0.1,
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
                },
            ],
            constraints: Vec::new(),
            normal_form_game: Some(NormalFormGameV1 {
                focal_actor: "origin".to_owned(),
                counterpart_actor: "market".to_owned(),
                focal_strategies: vec!["a".to_owned(), "b".to_owned()],
                counterpart_strategies: vec!["x".to_owned(), "y".to_owned()],
                payoff_cells: vec![
                    PayoffCellV1 {
                        focal_strategy: "a".to_owned(),
                        counterpart_strategy: "x".to_owned(),
                        focal_payoff: 3.0,
                        counterpart_payoff: 2.0,
                    },
                    PayoffCellV1 {
                        focal_strategy: "a".to_owned(),
                        counterpart_strategy: "y".to_owned(),
                        focal_payoff: 1.0,
                        counterpart_payoff: 4.0,
                    },
                    PayoffCellV1 {
                        focal_strategy: "b".to_owned(),
                        counterpart_strategy: "x".to_owned(),
                        focal_payoff: 2.0,
                        counterpart_payoff: 1.0,
                    },
                    PayoffCellV1 {
                        focal_strategy: "b".to_owned(),
                        counterpart_strategy: "y".to_owned(),
                        focal_payoff: 0.0,
                        counterpart_payoff: 3.0,
                    },
                ],
            }),
            provenance: ProvenanceV1 {
                source_system: "tests".to_owned(),
                source_refs: Vec::new(),
                generated_by: "tests".to_owned(),
                assumptions: Vec::new(),
            },
        }
    }

    #[test]
    fn dominated_strategy_elimination_detects_weaker_strategy() {
        let analysis = BaselineGameModel.analyze(&context()).expect("analysis");
        assert_eq!(analysis.dominated_focal_strategies, vec!["b".to_owned()]);
    }

    #[test]
    fn pure_strategy_equilibrium_detection_finds_expected_cell() {
        let analysis = BaselineGameModel.analyze(&context()).expect("analysis");
        assert_eq!(analysis.pure_strategy_equilibria.len(), 1);
        assert_eq!(analysis.pure_strategy_equilibria[0].focal_strategy, "a");
        assert_eq!(
            analysis.pure_strategy_equilibria[0].counterpart_strategy,
            "y"
        );
    }
}
