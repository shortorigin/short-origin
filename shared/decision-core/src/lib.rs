//! Governed quantitative decision primitives and deterministic baseline evaluators.

mod engine;
mod game;
mod learned;
mod policy;
mod probabilistic;
mod scores;

pub use engine::{
    BaselineDecisionEngine, DecisionAuditSink, DecisionEngine, DecisionEvaluation,
    MemoryDecisionAuditSink, OptionScoreInputs, UtilityFunction, WeightedUtilityFunction,
};
pub use game::{
    BaselineGameModel, GameAnalysis, GameModel, PureStrategyEquilibrium, StrategicBestResponses,
};
pub use learned::{
    ForecastAdapter, ForecastAdapterOutput, LearnedAdapterError, NotConfiguredLearnedAdapter,
    PolicyModelAdapter, PolicyModelOutput, RepresentationModel, RepresentationOutput,
};
pub use policy::{BaselinePolicyGate, PolicyGate};
pub use probabilistic::{
    aggregate_confidence, risk_penalized_utility, BaselineForecastModel, ForecastModel,
    MonteCarloScenarioSampler, MonteCarloSummary, ProbabilisticEvaluation, ThompsonSamplingBandit,
    ThompsonSamplingResult, WeightedExpectedValueScorer,
};
pub use scores::{
    ConfidenceScore, ConstraintViolation, DecisionClass, DecisionId, DecisionState,
    ProbabilityScore, RecommendationStatus, RiskScore, UtilityScore,
};
