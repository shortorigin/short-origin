use rand::{rngs::StdRng, Rng, SeedableRng};

use contracts::{DecisionOptionV1, OutcomeDistributionV1};

use crate::{ConfidenceScore, ProbabilityScore, RiskScore, UtilityScore};

/// Weighted expected-value scorer over explicit utility dimensions and outcome estimates.
#[derive(Debug, Clone, Copy, Default)]
pub struct WeightedExpectedValueScorer;

impl WeightedExpectedValueScorer {
    /// Scores one option by combining its utility breakdown and expected outcomes.
    #[must_use]
    pub fn score(&self, option: &DecisionOptionV1) -> UtilityScore {
        let utility = &option.utility_breakdown;
        let base = utility.value_score * 0.25
            + utility.resilience_score * 0.20
            + utility.compliance_score * 0.15
            + utility.cost_score * 0.10
            + utility.reversibility_score * 0.15
            + utility.strategic_fit_score * 0.15;
        let outcomes = option
            .expected_outcomes
            .iter()
            .map(|estimate| estimate.probability * estimate.expected_utility)
            .sum::<f64>();
        UtilityScore::new(base + outcomes)
    }
}

/// Summary statistics produced by seeded Monte Carlo scenario sampling.
#[derive(Debug, Clone, PartialEq)]
pub struct MonteCarloSummary {
    /// Mean utility observed across sampled scenarios.
    pub mean_utility: UtilityScore,
    /// Mean risk observed across sampled scenarios.
    pub mean_risk: RiskScore,
    /// Confidence proxy derived from repeated sampling stability.
    pub sampled_confidence: ConfidenceScore,
}

/// Seeded Monte Carlo sampler over bounded discrete scenario sets.
#[derive(Debug, Clone, Copy)]
pub struct MonteCarloScenarioSampler {
    iterations: usize,
}

impl MonteCarloScenarioSampler {
    /// Creates a sampler with a fixed number of iterations.
    #[must_use]
    pub fn new(iterations: usize) -> Self {
        Self {
            iterations: iterations.max(1),
        }
    }

    /// Samples a discrete distribution deterministically under the provided seed.
    #[must_use]
    pub fn sample(&self, distribution: &OutcomeDistributionV1, seed: u64) -> MonteCarloSummary {
        let mut rng = StdRng::seed_from_u64(seed);
        let iteration_count = usize_to_f64(self.iterations);
        let scenarios = if distribution.scenarios.is_empty() {
            vec![(
                distribution.expected_value,
                distribution.downside_probability,
            )]
        } else {
            distribution
                .scenarios
                .iter()
                .map(|scenario| (scenario.utility, scenario.risk))
                .collect()
        };
        let weights = if distribution.scenarios.is_empty() {
            vec![1.0]
        } else {
            distribution
                .scenarios
                .iter()
                .map(|scenario| scenario.probability)
                .collect()
        };

        let mut utility_sum = 0.0;
        let mut risk_sum = 0.0;
        let mut utility_values = Vec::with_capacity(self.iterations);

        for _ in 0..self.iterations {
            let index = sample_index(&weights, &mut rng);
            let (utility, risk) = scenarios[index];
            utility_sum += utility;
            risk_sum += risk;
            utility_values.push(utility);
        }

        let mean_utility = utility_sum / iteration_count;
        let mean_risk = risk_sum / iteration_count;
        let variance = utility_values
            .iter()
            .map(|value| {
                let delta = *value - mean_utility;
                delta * delta
            })
            .sum::<f64>()
            / iteration_count;
        let sampled_confidence = ConfidenceScore::new(1.0 / (1.0 + variance.sqrt()));

        MonteCarloSummary {
            mean_utility: UtilityScore::new(mean_utility),
            mean_risk: RiskScore::new(mean_risk),
            sampled_confidence,
        }
    }
}

/// Result of a Thompson-sampling pass over one option.
#[derive(Debug, Clone, PartialEq)]
pub struct ThompsonSamplingResult {
    /// Identifier of the sampled option.
    pub option_id: String,
    /// Sampled success probability.
    pub sampled_probability: ProbabilityScore,
}

/// Deterministic Beta-Bernoulli Thompson sampler using integer posterior counts.
#[derive(Debug, Clone, Copy, Default)]
pub struct ThompsonSamplingBandit;

impl ThompsonSamplingBandit {
    /// Returns posterior counts after applying the default `Beta(1, 1)` prior.
    #[must_use]
    pub fn posterior_counts(&self, option: &DecisionOptionV1) -> (u32, u32) {
        (
            option.historical_successes.saturating_add(1),
            option.historical_failures.saturating_add(1),
        )
    }

    /// Samples all provided options deterministically under the provided seed.
    #[must_use]
    pub fn sample_options(
        &self,
        options: &[DecisionOptionV1],
        seed: u64,
    ) -> Vec<ThompsonSamplingResult> {
        let mut rng = StdRng::seed_from_u64(seed);
        options
            .iter()
            .map(|option| {
                let (alpha, beta) = self.posterior_counts(option);
                ThompsonSamplingResult {
                    option_id: option.option_id.clone(),
                    sampled_probability: ProbabilityScore::new(sample_beta(alpha, beta, &mut rng)),
                }
            })
            .collect()
    }
}

/// Aggregated probabilistic evaluation for one option.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbabilisticEvaluation {
    /// Weighted expected-value score.
    pub expected_value: UtilityScore,
    /// Monte Carlo summary.
    pub monte_carlo: MonteCarloSummary,
    /// Thompson-sampled binary success probability.
    pub thompson_score: ProbabilityScore,
    /// Aggregated confidence score.
    pub aggregated_confidence: ConfidenceScore,
    /// Aggregated risk score.
    pub aggregated_risk: RiskScore,
}

/// Forecast model surface used by the decision engine.
pub trait ForecastModel {
    /// Evaluates one option deterministically under the provided seed.
    fn evaluate(
        &self,
        option: &DecisionOptionV1,
        seed: u64,
        iterations: usize,
    ) -> ProbabilisticEvaluation;
}

/// Baseline deterministic forecast model built from explicit bounded algorithms.
#[derive(Debug, Clone, Copy, Default)]
pub struct BaselineForecastModel;

impl ForecastModel for BaselineForecastModel {
    fn evaluate(
        &self,
        option: &DecisionOptionV1,
        seed: u64,
        iterations: usize,
    ) -> ProbabilisticEvaluation {
        let scorer = WeightedExpectedValueScorer;
        let sampler = MonteCarloScenarioSampler::new(iterations);
        let bandit = ThompsonSamplingBandit;
        let expected_value = scorer.score(option);
        let monte_carlo = sampler.sample(&option.outcome_distribution, seed);
        let sampled_probability = bandit
            .sample_options(std::slice::from_ref(option), seed.saturating_add(1))
            .into_iter()
            .next()
            .map_or_else(
                || ProbabilityScore::new(0.5),
                |result| result.sampled_probability,
            );
        let aggregated_confidence = aggregate_confidence(&[
            ConfidenceScore::new(option.risk_assessment.confidence),
            monte_carlo.sampled_confidence,
            ConfidenceScore::new(
                option
                    .expected_outcomes
                    .iter()
                    .map(|estimate| estimate.confidence)
                    .sum::<f64>()
                    / usize_to_f64(option.expected_outcomes.len().max(1)),
            ),
        ]);
        let aggregated_risk = RiskScore::new(
            (option.risk_assessment.risk_score
                + option.risk_assessment.tail_risk_score
                + option.risk_assessment.downside_probability
                + monte_carlo.mean_risk.value())
                / 4.0,
        );

        ProbabilisticEvaluation {
            expected_value,
            monte_carlo,
            thompson_score: sampled_probability,
            aggregated_confidence,
            aggregated_risk,
        }
    }
}

/// Aggregates multiple confidence values into a bounded score.
#[must_use]
pub fn aggregate_confidence(values: &[ConfidenceScore]) -> ConfidenceScore {
    if values.is_empty() {
        return ConfidenceScore::new(0.0);
    }

    let mean = values.iter().map(|value| value.value()).sum::<f64>() / usize_to_f64(values.len());
    ConfidenceScore::new(mean)
}

/// Applies a linear risk penalty to a utility score.
#[must_use]
pub fn risk_penalized_utility(
    utility: UtilityScore,
    risk: RiskScore,
    penalty_factor: f64,
) -> UtilityScore {
    UtilityScore::new(utility.value() - risk.value() * penalty_factor)
}

fn sample_index(weights: &[f64], rng: &mut StdRng) -> usize {
    let total = weights
        .iter()
        .copied()
        .filter(|weight| weight.is_finite() && *weight > 0.0)
        .sum::<f64>();
    if total <= 0.0 {
        return 0;
    }

    let target = rng.random::<f64>() * total;
    let mut running = 0.0;
    for (index, weight) in weights.iter().copied().enumerate() {
        if !weight.is_finite() || weight <= 0.0 {
            continue;
        }
        running += weight;
        if target <= running {
            return index;
        }
    }
    weights.len().saturating_sub(1)
}

fn usize_to_f64(value: usize) -> f64 {
    f64::from(u32::try_from(value).unwrap_or(u32::MAX))
}

fn sample_beta(alpha: u32, beta: u32, rng: &mut StdRng) -> f64 {
    let left = sample_erlang(alpha.max(1), rng);
    let right = sample_erlang(beta.max(1), rng);
    let total = left + right;
    if total > 0.0 {
        left / total
    } else {
        0.5
    }
}

fn sample_erlang(shape: u32, rng: &mut StdRng) -> f64 {
    (0..shape)
        .map(|_| {
            let draw = rng.random::<f64>().clamp(f64::MIN_POSITIVE, 1.0);
            -draw.ln()
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use contracts::{
        OutcomeDistributionV1, OutcomeEstimateV1, OutcomeScenarioV1, RiskAssessmentV1,
        UtilityBreakdownV1,
    };

    use super::{
        aggregate_confidence, risk_penalized_utility, MonteCarloScenarioSampler,
        ThompsonSamplingBandit, WeightedExpectedValueScorer,
    };
    use crate::{ConfidenceScore, RiskScore, UtilityScore};
    use contracts::{DecisionOptionV1, ReversibilityClassV1};

    fn option() -> DecisionOptionV1 {
        DecisionOptionV1 {
            option_id: "safe".to_owned(),
            title: "Safe".to_owned(),
            description: "Safe option".to_owned(),
            expected_outcomes: vec![OutcomeEstimateV1 {
                estimate_id: "estimate-1".to_owned(),
                description: "steady".to_owned(),
                probability: 0.6,
                expected_utility: 0.7,
                risk_adjustment: -0.1,
                confidence: 0.8,
                rationale: "historical".to_owned(),
            }],
            outcome_distribution: OutcomeDistributionV1 {
                distribution_id: "distribution-1".to_owned(),
                scenarios: vec![
                    OutcomeScenarioV1 {
                        label: "upside".to_owned(),
                        probability: 0.7,
                        utility: 0.8,
                        risk: 0.2,
                    },
                    OutcomeScenarioV1 {
                        label: "downside".to_owned(),
                        probability: 0.3,
                        utility: 0.2,
                        risk: 0.6,
                    },
                ],
                expected_value: 0.62,
                variance: 0.04,
                downside_probability: 0.3,
                rationale: "bounded".to_owned(),
            },
            risk_assessment: RiskAssessmentV1 {
                risk_score: 0.25,
                downside_probability: 0.3,
                tail_risk_score: 0.4,
                confidence: 0.75,
                rationale: "controlled".to_owned(),
                mitigation: "rollback".to_owned(),
            },
            utility_breakdown: UtilityBreakdownV1 {
                value_score: 0.7,
                resilience_score: 0.8,
                compliance_score: 0.9,
                cost_score: 0.5,
                reversibility_score: 0.9,
                strategic_fit_score: 0.8,
                rationale: "balanced".to_owned(),
            },
            rollback_plan: "rollback".to_owned(),
            reversibility: ReversibilityClassV1::GuardedRollback,
            historical_successes: 6,
            historical_failures: 2,
        }
    }

    #[test]
    fn expected_value_scoring_is_stable() {
        let score = WeightedExpectedValueScorer.score(&option()).value();
        assert!((score - 1.195).abs() < 1e-9);
    }

    #[test]
    fn monte_carlo_sampling_is_seed_deterministic() {
        let sampler = MonteCarloScenarioSampler::new(64);
        let first = sampler.sample(&option().outcome_distribution, 7);
        let second = sampler.sample(&option().outcome_distribution, 7);
        assert_eq!(first, second);
    }

    #[test]
    fn thompson_sampling_uses_beta_posterior_counts() {
        let bandit = ThompsonSamplingBandit;
        assert_eq!(bandit.posterior_counts(&option()), (7, 3));
        let samples = bandit.sample_options(&[option()], 11);
        assert_eq!(samples.len(), 1);
        assert!(samples[0].sampled_probability.value() >= 0.0);
        assert!(samples[0].sampled_probability.value() <= 1.0);
    }

    #[test]
    fn confidence_aggregation_averages_inputs() {
        let aggregated = aggregate_confidence(&[
            ConfidenceScore::new(0.4),
            ConfidenceScore::new(0.6),
            ConfidenceScore::new(0.8),
        ]);
        assert!((aggregated.value() - 0.6).abs() < 1e-9);
    }

    #[test]
    fn risk_penalized_utility_combiner_applies_linear_penalty() {
        let combined = risk_penalized_utility(UtilityScore::new(0.9), RiskScore::new(0.2), 0.5);
        assert!((combined.value() - 0.8).abs() < 1e-9);
    }
}
