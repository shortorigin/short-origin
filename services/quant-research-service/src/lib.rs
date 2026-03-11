use std::cmp::Ordering;
use std::collections::VecDeque;

use contracts::{ExperimentMetadataV1, ExperimentResultV1, ResearchTaskV1, ServiceBoundaryV1};
use error_model::{InstitutionalError, InstitutionalResult};
use trading_errors::TradingError;
use trading_sim::{DeterministicBacktestEngine, SweepJob, run_trend_sweep};

mod weather;

pub use weather::*;

const SERVICE_NAME: &str = "quant-research-service";
const DOMAIN_NAME: &str = "capital_markets_research";
const APPROVED_WORKFLOWS: &[&str] = &["quant_strategy_promotion"];
const OWNED_AGGREGATES: &[&str] = &["experiment_result", "research_task"];

fn map_trading_error(error: TradingError) -> InstitutionalError {
    InstitutionalError::external(
        "trading-sim",
        Some("quant-research".to_string()),
        error.to_string(),
    )
}

#[derive(Debug, Default, Clone)]
struct InMemoryResearchCatalog {
    results: Vec<ExperimentResultV1>,
    tasks: VecDeque<ResearchTaskV1>,
}

impl InMemoryResearchCatalog {
    fn register(&mut self, result: ExperimentResultV1) {
        self.results.push(result);
    }

    fn ranked(&self) -> Vec<ExperimentResultV1> {
        let mut ranked = self.results.clone();
        ranked.sort_by(|left, right| {
            right
                .summary
                .sharpe
                .partial_cmp(&left.summary.sharpe)
                .unwrap_or(Ordering::Equal)
                .then_with(|| left.config_hash.cmp(&right.config_hash))
        });
        ranked
    }

    fn top_result(&self) -> Option<&ExperimentResultV1> {
        self.results.iter().max_by(|left, right| {
            left.summary
                .sharpe
                .partial_cmp(&right.summary.sharpe)
                .unwrap_or(Ordering::Equal)
                .then_with(|| right.config_hash.cmp(&left.config_hash))
        })
    }

    fn result_count(&self) -> usize {
        self.results.len()
    }

    fn is_empty(&self) -> bool {
        self.results.is_empty()
    }

    fn enqueue(&mut self, task: ResearchTaskV1) {
        self.tasks.push_back(task);
    }

    fn dequeue(&mut self) -> Option<ResearchTaskV1> {
        self.tasks.pop_front()
    }

    fn task_count(&self) -> usize {
        self.tasks.len()
    }

    fn tasks_empty(&self) -> bool {
        self.tasks.is_empty()
    }
}

#[derive(Debug, Default, Clone)]
pub struct QuantResearchService {
    catalog: InMemoryResearchCatalog,
}

impl QuantResearchService {
    pub fn register(&mut self, result: ExperimentResultV1) {
        self.catalog.register(result);
    }

    #[must_use]
    pub fn ranked(&self) -> Vec<ExperimentResultV1> {
        self.catalog.ranked()
    }

    #[must_use]
    pub fn top_result(&self) -> Option<&ExperimentResultV1> {
        self.catalog.top_result()
    }

    #[must_use]
    pub fn result_count(&self) -> usize {
        self.catalog.result_count()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.catalog.is_empty()
    }

    pub fn enqueue(&mut self, task: ResearchTaskV1) {
        self.catalog.enqueue(task);
    }

    pub fn dequeue(&mut self) -> Option<ResearchTaskV1> {
        self.catalog.dequeue()
    }

    #[must_use]
    pub fn task_count(&self) -> usize {
        self.catalog.task_count()
    }

    #[must_use]
    pub fn tasks_empty(&self) -> bool {
        self.catalog.tasks_empty()
    }

    pub fn evaluate_trend_sweep(
        &mut self,
        engine: &DeterministicBacktestEngine,
        events: &[contracts::MarketEventV1],
        jobs: &[SweepJob],
        simulation: &contracts::SimulationConfigV1,
    ) -> InstitutionalResult<Vec<ExperimentResultV1>> {
        let mut results = Vec::new();
        for result in
            run_trend_sweep(engine, events, jobs, simulation).map_err(map_trading_error)?
        {
            let experiment = ExperimentResultV1 {
                config_hash: result.config_hash.clone(),
                summary: result.summary.clone(),
                metadata: ExperimentMetadataV1 {
                    trade_count: Some(result.trade_count),
                },
            };
            self.register(experiment.clone());
            results.push(experiment);
        }
        Ok(results)
    }
}

#[must_use]
pub fn ai_assisted_summary(results: &[ExperimentResultV1]) -> String {
    if results.is_empty() {
        return "No experiments were available for summarization.".to_string();
    }

    let best = results
        .iter()
        .max_by(|left, right| {
            left.summary
                .sharpe
                .partial_cmp(&right.summary.sharpe)
                .unwrap_or(Ordering::Equal)
        })
        .expect("non-empty");

    format!(
        "Top experiment {} achieved Sharpe {:.3}, return {:.2}%, drawdown {:.2}% over {} runs.",
        best.config_hash,
        best.summary.sharpe,
        best.summary.total_return * 100.0,
        best.summary.max_drawdown * 100.0,
        results.len()
    )
}

#[must_use]
pub fn service_boundary() -> ServiceBoundaryV1 {
    ServiceBoundaryV1 {
        service_name: SERVICE_NAME.into(),
        domain: DOMAIN_NAME.to_owned(),
        approved_workflows: APPROVED_WORKFLOWS.iter().copied().map(Into::into).collect(),
        owned_aggregates: OWNED_AGGREGATES.iter().copied().map(Into::into).collect(),
    }
}

#[cfg(test)]
mod tests {
    mod contract_parity {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../testing/contract_parity.rs"
        ));
    }

    use std::collections::BTreeMap;

    use chrono::{Duration, TimeZone, Utc};
    use contract_parity::assert_service_boundary_matches_catalog;
    use contracts::{
        AssetClassV1, ExperimentMetadataV1, ExperimentResultV1, MarketEventV1, OhlcvBarV1,
        PerformanceSummaryV1, ResearchTaskV1, SymbolV1, VenueV1,
    };
    use trading_core::{
        BasicLinearModel, build_feature_rows, experiment_config_hash, walk_forward,
    };

    use super::{DOMAIN_NAME, QuantResearchService, ai_assisted_summary, service_boundary};

    #[test]
    fn service_boundary_matches_enterprise_catalog() {
        let source = include_str!(
            "../../../enterprise/domains/capital_markets_research/service_boundaries.toml"
        );
        let boundary = service_boundary();

        assert_service_boundary_matches_catalog(&boundary, DOMAIN_NAME, source);
    }

    #[test]
    fn registry_ranks_deterministically() {
        let mut service = QuantResearchService::default();
        service.register(ExperimentResultV1 {
            config_hash: "b".to_string(),
            summary: PerformanceSummaryV1 {
                total_return: 0.2,
                sharpe: 1.0,
                max_drawdown: 0.1,
                turnover: 1.0,
            },
            metadata: ExperimentMetadataV1 { trade_count: None },
        });
        service.register(ExperimentResultV1 {
            config_hash: "a".to_string(),
            summary: PerformanceSummaryV1 {
                total_return: 0.2,
                sharpe: 1.0,
                max_drawdown: 0.1,
                turnover: 1.0,
            },
            metadata: ExperimentMetadataV1 { trade_count: None },
        });
        let ranked = service.ranked();
        assert_eq!(ranked[0].config_hash, "a");
    }

    #[test]
    fn walk_forward_splits_data() {
        let symbol = SymbolV1::new(VenueV1::Coinbase, AssetClassV1::Crypto, "BTC", "USD");
        let start = Utc
            .with_ymd_and_hms(2026, 3, 1, 0, 0, 0)
            .single()
            .expect("start");
        let mut events = Vec::new();
        for index in 0..30 {
            events.push(MarketEventV1::Bar(OhlcvBarV1 {
                symbol: symbol.clone(),
                open_time: start + Duration::minutes(index),
                close_time: start + Duration::minutes(index + 1),
                open: 100.0,
                high: 101.0,
                low: 99.0,
                close: 100.0 + f64::from(i32::try_from(index).unwrap_or(i32::MAX)),
                volume: 1000.0,
            }));
        }
        let rows = build_feature_rows(&events);
        let splits = walk_forward(&rows, 10, 5).expect("split");
        assert_eq!(splits.len(), 4);
        let model = BasicLinearModel::fit(&splits[0].train);
        assert!(model.score(&splits[0].test[0]).is_finite());
    }

    #[test]
    fn config_hash_stable() {
        let config = contracts::ExperimentConfigV1 {
            strategy_name: "trend".to_string(),
            parameter_grid: BTreeMap::from([("lookback".to_string(), 20.0)]),
            training_window: 200,
            test_window: 50,
        };
        let hash1 = experiment_config_hash(&config).expect("hash");
        let hash2 = experiment_config_hash(&config).expect("hash");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn queue_round_trip() {
        let mut service = QuantResearchService::default();
        service.enqueue(ResearchTaskV1 {
            task_id: "1".to_string(),
            description: "sweep".to_string(),
        });
        assert_eq!(service.task_count(), 1);
        let task = service.dequeue().expect("task");
        assert_eq!(task.task_id, "1");
    }

    #[test]
    fn summary_mentions_top_experiment() {
        let text = ai_assisted_summary(&[ExperimentResultV1 {
            config_hash: "cfg".to_string(),
            summary: PerformanceSummaryV1 {
                total_return: 0.1,
                sharpe: 1.2,
                max_drawdown: 0.05,
                turnover: 0.8,
            },
            metadata: ExperimentMetadataV1 { trade_count: None },
        }]);
        assert!(text.contains("Top experiment"));
    }
}
