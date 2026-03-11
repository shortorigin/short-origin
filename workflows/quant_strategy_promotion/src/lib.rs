use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use chrono::{TimeZone, Timelike, Utc};
use compliance_service::{
    CapitalLimits, ComplianceService, TradeSurveillance, validate_compliance_pack,
};
use contracts::{
    AgentActionRequestV1, AssetClassV1, BestExecutionRecordV1, ExperimentConfigV1,
    ExperimentResultV1, HistoricalDataRequestV1, OrderRequestV1, OrderTypeV1, PromotionGateV1,
    PromotionRecommendationV1, QuantStrategyPromotionRequestV1, ResearchTaskV1, SignalSideV1,
    SignalV1, SimulationConfigV1, SymbolV1, TimeInForceV1, VenueV1, WorkflowBoundaryV1,
};
use error_model::{InstitutionalError, InstitutionalResult, OperationContext};
use evidence_service::EvidenceService;
use execution_service::{EventDrivenStrategyEngine, ExecutionRouter};
use governance_service::GovernanceService;
use identity::{AggregateId, EnvironmentId, ServiceId, WorkflowId};
use market_data_service::{CoinbaseAdapter, MarketDataService, OandaAdapter};
use orchestrator::WorkflowEngine;
use policy_sdk::{ApprovalVerificationPort, PolicyDecisionPort};
use portfolio_service::PortfolioService;
use quant_research_service::{QuantResearchService, ai_assisted_summary};
use strategy_sandbox::{StrategySandbox, WasmRuntimePolicy};
use trading_core::{
    BasicLinearModel, Clock, FixedClock, IdGenerator, LimitConfig, SequenceIdGenerator,
    StrategyAllocationRule, build_feature_rows, experiment_config_hash, walk_forward,
};
use trading_risk_service::TradingRiskService;
use trading_sim::{
    AgentBehavior, DeterministicBacktestEngine, MeanReversion, PaperVenueAdapter, PaperVenueConfig,
    SweepJob, TrendFollower, run_agent_market_simulation,
};

const SEED: u64 = 20_260_305;
const INITIAL_CASH: f64 = 100_000.0;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct PipelineSummary {
    pub seed: u64,
    pub top_config_hash: String,
    pub experiments: usize,
    pub paper_sessions_ok: bool,
    pub compliance_ok: bool,
    pub promotion_ready: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct QuantStrategyPromotionReport {
    pub summary: PipelineSummary,
    pub promotion_gate: PromotionGateV1,
    pub recommendation: PromotionRecommendationV1,
    pub compliance_report: contracts::ComplianceReportV1,
    pub ranked_experiments: Vec<ExperimentResultV1>,
}

#[must_use]
pub fn workflow_boundary() -> WorkflowBoundaryV1 {
    WorkflowBoundaryV1 {
        workflow_name: "quant_strategy_promotion".to_owned(),
        touched_domains: vec![
            "capital_markets_data".to_owned(),
            "capital_markets_research".to_owned(),
            "capital_markets_execution".to_owned(),
            "capital_markets_portfolio".to_owned(),
            "capital_markets_risk".to_owned(),
            "compliance".to_owned(),
            "strategy_governance".to_owned(),
            "audit_assurance".to_owned(),
        ],
        target_services: vec![
            "market-data-service".to_owned(),
            "quant-research-service".to_owned(),
            "execution-service".to_owned(),
            "portfolio-service".to_owned(),
            "trading-risk-service".to_owned(),
            "compliance-service".to_owned(),
            "governance-service".to_owned(),
            "evidence-service".to_owned(),
        ],
        emits_evidence: true,
        mutation_path_only: true,
    }
}

pub async fn execute<P, A, E>(
    engine: &mut WorkflowEngine<P, A, E>,
    governance_service: &mut GovernanceService,
    compliance_service: &mut ComplianceService,
    audit_service: &mut EvidenceService,
    market_data_service: &mut MarketDataService,
    research_service: &mut QuantResearchService,
    action: &AgentActionRequestV1,
    request: QuantStrategyPromotionRequestV1,
) -> InstitutionalResult<QuantStrategyPromotionReport>
where
    P: PolicyDecisionPort,
    A: ApprovalVerificationPort,
    E: evidence_sdk::EvidenceSink,
{
    let clock: Arc<dyn Clock> = Arc::new(FixedClock::new(
        Utc.with_ymd_and_hms(2026, 3, 5, 17, 44, 41)
            .single()
            .expect("clock")
            .with_nanosecond(669_722_000)
            .expect("nanoseconds"),
    ));
    let ids: Arc<dyn IdGenerator> = Arc::new(SequenceIdGenerator::new("capital"));

    audit_service.append_audit_event(serde_json::json!({
        "event": "pipeline_start",
        "promotion_id": request.promotion_id,
        "seed": request.seed,
    }))?;

    let start = Utc
        .with_ymd_and_hms(2026, 3, 1, 0, 0, 0)
        .single()
        .expect("start");
    let end = Utc
        .with_ymd_and_hms(2026, 3, 1, 4, 0, 0)
        .single()
        .expect("end");
    let coinbase_symbol = SymbolV1::new(VenueV1::Coinbase, AssetClassV1::Crypto, "BTC", "USD");
    let oanda_symbol = SymbolV1::new(VenueV1::Oanda, AssetClassV1::Forex, "EUR", "USD");

    let coinbase = CoinbaseAdapter::new(SEED, Arc::clone(&clock));
    let oanda = OandaAdapter::new(SEED + 10, Arc::clone(&clock));
    let btc_dataset = market_data_service.ingest_historical(
        ids.as_ref(),
        &coinbase,
        "coinbase_btcusd_1m",
        HistoricalDataRequestV1 {
            symbol: coinbase_symbol.clone(),
            start,
            end,
            interval_seconds: 60,
        },
    )?;
    let _eur_dataset = market_data_service.ingest_historical(
        ids.as_ref(),
        &oanda,
        "oanda_eurusd_1m",
        HistoricalDataRequestV1 {
            symbol: oanda_symbol.clone(),
            start,
            end,
            interval_seconds: 60,
        },
    )?;

    let features = build_feature_rows(&btc_dataset.events);
    let splits = walk_forward(&features, 120, 30).map_err(invariant_error)?;
    let simulation = SimulationConfigV1 {
        seed: SEED,
        fee_bps: 1.0,
        slippage_bps: 1.5,
        latency_ms: 25,
        initial_cash: INITIAL_CASH,
    };
    let engine_backtest = DeterministicBacktestEngine::new(Arc::clone(&clock), Arc::clone(&ids));
    let mut jobs = Vec::new();
    for lookback in 5..15 {
        let config = ExperimentConfigV1 {
            strategy_name: request.strategy_id.clone(),
            parameter_grid: BTreeMap::from([(
                "lookback".to_string(),
                f64::from(u32::try_from(lookback).unwrap_or(u32::MAX)),
            )]),
            training_window: 120,
            test_window: 30,
        };
        jobs.push(SweepJob {
            strategy_id: format!("trend_lb_{lookback}"),
            lookback,
            trade_size: 0.10,
            config_hash: experiment_config_hash(&config).map_err(invariant_error)?,
        });
    }

    research_service.evaluate_trend_sweep(
        &engine_backtest,
        &btc_dataset.events,
        &jobs,
        &simulation,
    )?;
    let ranked = research_service.ranked();
    let Some(top) = ranked.first() else {
        return Err(InstitutionalError::invariant(
            OperationContext::new("workflows/quant_strategy_promotion", "execute"),
            "quant strategy promotion requires ranked experiments",
        ));
    };
    let ai_summary = ai_assisted_summary(&ranked);

    let mut mean_reversion = MeanReversion::new("mean_reversion", 20, 1.5, 0.10);
    let mean_reversion_result = trading_core::BacktestEngine::run(
        &engine_backtest,
        &mut mean_reversion,
        &btc_dataset.events,
        "mean_reversion_cfg",
        &simulation,
    )
    .map_err(invariant_error)?;

    let allocation_rules = vec![
        StrategyAllocationRule {
            strategy_id: "trend_lb_5".to_string(),
            max_notional: 25_000.0,
        },
        StrategyAllocationRule {
            strategy_id: "mean_reversion".to_string(),
            max_notional: 20_000.0,
        },
    ];
    let portfolio_service =
        PortfolioService::new(INITIAL_CASH, Arc::clone(&clock), allocation_rules.clone());
    let allocation_preview = portfolio_service.allocate(
        &[
            SignalV1 {
                strategy_id: "trend_lb_5".to_string(),
                symbol: coinbase_symbol.clone(),
                side: SignalSideV1::Buy,
                quantity: 1.5,
                confidence: 0.6,
                reason: "preview".to_string(),
                determinism: contracts::DeterminismKeyV1::new("alloc-1", "v1", "alloc"),
            },
            SignalV1 {
                strategy_id: "mean_reversion".to_string(),
                symbol: coinbase_symbol.clone(),
                side: SignalSideV1::Sell,
                quantity: 2.0,
                confidence: 0.4,
                reason: "preview".to_string(),
                determinism: contracts::DeterminismKeyV1::new("alloc-2", "v1", "alloc"),
            },
        ],
        &HashMap::from([(coinbase_symbol.ticker(), 100.0)]),
    );

    research_service.enqueue(ResearchTaskV1 {
        task_id: "rq-1".to_string(),
        description: "run walk-forward sweep".to_string(),
    });
    research_service.enqueue(ResearchTaskV1 {
        task_id: "rq-2".to_string(),
        description: "evaluate mean reversion".to_string(),
    });
    let _ = research_service.dequeue();

    if let Some(split) = splits.first() {
        let model = BasicLinearModel::fit(&split.train);
        let _ = split
            .test
            .first()
            .map(|row| model.score(row))
            .unwrap_or_default();
    }

    let agent_simulation = run_agent_market_simulation(
        start,
        100.0,
        120,
        &[
            AgentBehavior {
                name: "maker".to_string(),
                inventory_bias: 0.2,
                aggressiveness: 0.8,
            },
            AgentBehavior {
                name: "momentum_taker".to_string(),
                inventory_bias: 0.4,
                aggressiveness: 1.0,
            },
        ],
    );

    let paper_sessions_ok = run_paper_sessions(
        &btc_dataset.events,
        top.config_hash.clone(),
        Arc::clone(&clock),
        Arc::clone(&ids),
    );

    governance_service.submit_model("trend_follower", "v1", "baseline daily model");
    governance_service.approve_model("trend_follower", "v1", "risk_officer");

    let capital_limits = CapitalLimits {
        global_notional_limit: 1_000_000.0,
        strategy_limits: BTreeMap::from([
            ("trend_follower".to_string(), 100_000.0),
            ("mean_reversion".to_string(), 50_000.0),
        ]),
    };
    let surveillance = TradeSurveillance {
        max_single_order_quantity: 1_000.0,
    };
    let sample_order = OrderRequestV1 {
        order_id: ids.next_id(),
        strategy_id: "trend_follower".to_string(),
        symbol: coinbase_symbol.clone(),
        venue: VenueV1::Coinbase,
        side: contracts::Side::Buy,
        quantity: 0.5,
        limit_price: Some(100.0),
        order_type: OrderTypeV1::Limit,
        tif: TimeInForceV1::Gtc,
        submitted_at: clock.now(),
    };

    let mut builder = compliance_service.new_builder();
    builder.check_control("risk_limits");
    builder.check_control("model_approval");
    builder.check_control("best_execution");
    builder.capture_order_audit(sample_order.clone(), "risk approved + execution accepted");
    builder.capture_best_execution(BestExecutionRecordV1 {
        record_id: ids.next_id(),
        venue: VenueV1::Coinbase,
        captured_at: clock.now(),
        slippage_bps: 1.5,
        expected_price: 100.0,
        executed_price: 100.015,
    });
    if let Some(breach) = capital_limits.check(
        &sample_order.strategy_id,
        100_000_000.0,
        ids.as_ref(),
        clock.as_ref(),
    ) {
        builder.capture_breach(breach);
    }
    if let Some(breach) = surveillance.inspect(&sample_order, ids.as_ref(), clock.as_ref()) {
        builder.capture_breach(breach);
    }
    let compliance_report =
        builder.build(&request.business_date, governance_service.approved_models());
    validate_compliance_pack(&compliance_report)?;
    compliance_service.record_report(compliance_report.clone())?;

    audit_service.append_audit_event(serde_json::json!({
        "event": "compliance_pack_generated",
        "promotion_id": request.promotion_id,
        "controls_checked": compliance_report.daily_control_attestation.controls_checked,
    }))?;

    let mut sandbox = StrategySandbox::new(WasmRuntimePolicy::default(), Arc::clone(&clock))
        .map_err(invariant_error)?;
    sandbox
        .load_wat(
            "trend_wasm_v1",
            &strategy_sandbox::demo_strategy_wat("trend_wasm"),
            contracts::StrategyConfigV1 {
                strategy_id: "trend_wasm".to_string(),
                model_version: "v1".to_string(),
                config_hash: top.config_hash.clone(),
                parameters: serde_json::json!({ "lookback": 8 }),
            },
        )
        .map_err(invariant_error)?;
    sandbox
        .on_market_event(
            "trend_wasm_v1",
            btc_dataset.events.first().expect("btc events available"),
            contracts::DeterminismKeyV1::new(ids.next_id(), "v1", top.config_hash.clone()),
        )
        .map_err(invariant_error)?;

    let promotion_gate = PromotionGateV1 {
        backtest_evidence: !ranked.is_empty() && mean_reversion_result.trade_count > 0,
        paper_trade_evidence: paper_sessions_ok,
        risk_signoff: true,
        compliance_attested: compliance_report.daily_control_attestation.exceptions.len() <= 10,
    };
    let summary = PipelineSummary {
        seed: request.seed,
        top_config_hash: top.config_hash.clone(),
        experiments: research_service.result_count(),
        paper_sessions_ok,
        compliance_ok: validate_compliance_pack(&compliance_report).is_ok(),
        promotion_ready: promotion_gate.ready(),
        notes: vec![
            ai_summary.clone(),
            format!("walk_forward_splits={}", splits.len()),
            format!("agent_sim_ticks={}", agent_simulation.ticks.len()),
            format!("allocation_preview={}", allocation_preview.len()),
        ],
    };

    let recommendation = PromotionRecommendationV1 {
        recommendation_id: ids.next_id(),
        strategy_id: request.strategy_id.clone(),
        config_hash: top.config_hash.clone(),
        recommended: promotion_gate.ready(),
        summary: ai_summary,
        required_workflows: vec![
            "strategy_review".to_string(),
            "compliance_attestation".to_string(),
        ],
        gate: promotion_gate.clone(),
    };

    let guarded_request = enforcement::GuardedMutationRequest {
        action_id: action.action_id.clone(),
        workflow_name: WorkflowId::from("quant_strategy_promotion"),
        target_service: ServiceId::from("governance-service"),
        target_aggregate: AggregateId::from("promotion_recommendation"),
        actor_ref: action.actor_ref.clone(),
        impact_tier: action.impact_tier,
        classification: action.classification,
        policy_refs: action.policy_refs.clone(),
        required_approver_roles: action.required_approver_roles.clone(),
        environment: EnvironmentId::from("prod"),
        cross_domain: true,
    };

    let recommendation = engine
        .execute_mutation(guarded_request, |context| {
            governance_service.record_recommendation(context, recommendation.clone())
        })
        .await?;

    Ok(QuantStrategyPromotionReport {
        summary,
        promotion_gate,
        recommendation,
        compliance_report,
        ranked_experiments: ranked,
    })
}

fn run_paper_sessions(
    events: &[contracts::MarketEventV1],
    config_hash: String,
    clock: Arc<dyn Clock>,
    ids: Arc<dyn IdGenerator>,
) -> bool {
    let mut all_good = true;

    for session_index in 0..5 {
        let strategy = Box::new(TrendFollower::new(
            format!("paper_trend_{session_index}"),
            8,
            0.1,
        ));
        let risk = Box::new(TradingRiskService::new(
            LimitConfig {
                max_order_notional: 100_000.0,
                max_gross_exposure: 500_000.0,
                max_open_orders: 100,
            },
            Arc::clone(&clock),
        ));
        let portfolio = Box::new(PortfolioService::new(
            INITIAL_CASH,
            Arc::clone(&clock),
            Vec::new(),
        ));

        let mut router = ExecutionRouter::new();
        router.register(
            VenueV1::Coinbase,
            Box::new(PaperVenueAdapter::new(
                VenueV1::Coinbase,
                PaperVenueConfig {
                    base_price: 100.0,
                    partial_fill_ratio: 1.0,
                    ..PaperVenueConfig::default()
                },
                Arc::clone(&clock),
                Arc::clone(&ids),
            )),
        );
        router.register(
            VenueV1::Oanda,
            Box::new(PaperVenueAdapter::new(
                VenueV1::Oanda,
                PaperVenueConfig {
                    base_price: 1.08,
                    partial_fill_ratio: 1.0,
                    ..PaperVenueConfig::default()
                },
                Arc::clone(&clock),
                Arc::clone(&ids),
            )),
        );

        let mut engine = EventDrivenStrategyEngine::new(
            strategy,
            risk,
            portfolio,
            router,
            config_hash.clone(),
            "v1",
            Arc::clone(&ids),
            Arc::clone(&clock),
        );

        for event in events.iter().take(90) {
            if engine.on_market_event(event).is_err() {
                all_good = false;
            }
        }

        if engine.recorded_events().is_empty() {
            all_good = false;
        }
    }

    all_good
}

fn invariant_error(error: impl ToString) -> InstitutionalError {
    InstitutionalError::invariant(
        OperationContext::new("workflows/quant_strategy_promotion", "invariant_error"),
        error.to_string(),
    )
}
