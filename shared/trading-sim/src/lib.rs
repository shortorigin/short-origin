use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use chrono::{DateTime, Utc};
use contracts::{
    BacktestResultV1, DeterminismKeyV1, EquityPointV1, FillV1, MarketEventV1, OrderStatusV1,
    PerformanceSummaryV1, SignalSideV1, SignalV1, SimulationConfigV1, StrategyConfigV1,
    StrategyStateSnapshotV1, VenueV1,
};
use serde::{Deserialize, Serialize};
use trading_core::{
    BacktestEngine, Clock, DoubleEntryLedger, ExecutionVenueAdapter, IdGenerator, PortfolioLedger,
    StrategyModule, SystemClock, SystemIdGenerator, build_order_request,
};
use trading_errors::TradingResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendFollower {
    pub strategy_id: String,
    pub lookback: usize,
    pub trade_size: f64,
    closes: Vec<f64>,
    cfg: Option<StrategyConfigV1>,
}

impl TrendFollower {
    #[must_use]
    pub fn new(strategy_id: impl Into<String>, lookback: usize, trade_size: f64) -> Self {
        Self {
            strategy_id: strategy_id.into(),
            lookback,
            trade_size,
            closes: Vec::new(),
            cfg: None,
        }
    }
}

impl StrategyModule for TrendFollower {
    fn init(&mut self, config: StrategyConfigV1) -> TradingResult<()> {
        self.cfg = Some(config);
        Ok(())
    }

    fn on_market_event(
        &mut self,
        event: &MarketEventV1,
        determinism: DeterminismKeyV1,
    ) -> TradingResult<Vec<SignalV1>> {
        let price = event.price();
        self.closes.push(price);
        if self.closes.len() < self.lookback {
            return Ok(Vec::new());
        }

        let start = self.closes.len() - self.lookback;
        let moving_average = self.closes[start..].iter().sum::<f64>() / usize_to_f64(self.lookback);
        let side = if price > moving_average {
            SignalSideV1::Buy
        } else if price < moving_average {
            SignalSideV1::Sell
        } else {
            SignalSideV1::Hold
        };

        Ok(vec![SignalV1 {
            strategy_id: self.strategy_id.clone(),
            symbol: event.symbol().clone(),
            side,
            quantity: self.trade_size,
            confidence: (price - moving_average).abs() / price.max(1.0),
            reason: format!("trend(px={price:.4}, ma={moving_average:.4})"),
            determinism,
        }])
    }

    fn on_timer(
        &mut self,
        _now: DateTime<Utc>,
        _determinism: DeterminismKeyV1,
    ) -> TradingResult<Vec<SignalV1>> {
        Ok(Vec::new())
    }

    fn snapshot_state(&self, now: DateTime<Utc>) -> StrategyStateSnapshotV1 {
        StrategyStateSnapshotV1 {
            strategy_id: self.strategy_id.clone(),
            timestamp: now,
            state: serde_json::json!({
                "lookback": self.lookback,
                "close_count": self.closes.len(),
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeanReversion {
    pub strategy_id: String,
    pub lookback: usize,
    pub z_threshold: f64,
    pub trade_size: f64,
    closes: Vec<f64>,
    cfg: Option<StrategyConfigV1>,
}

impl MeanReversion {
    #[must_use]
    pub fn new(
        strategy_id: impl Into<String>,
        lookback: usize,
        z_threshold: f64,
        trade_size: f64,
    ) -> Self {
        Self {
            strategy_id: strategy_id.into(),
            lookback,
            z_threshold,
            trade_size,
            closes: Vec::new(),
            cfg: None,
        }
    }
}

impl StrategyModule for MeanReversion {
    fn init(&mut self, config: StrategyConfigV1) -> TradingResult<()> {
        self.cfg = Some(config);
        Ok(())
    }

    fn on_market_event(
        &mut self,
        event: &MarketEventV1,
        determinism: DeterminismKeyV1,
    ) -> TradingResult<Vec<SignalV1>> {
        let price = event.price();
        self.closes.push(price);
        if self.closes.len() < self.lookback {
            return Ok(Vec::new());
        }

        let start = self.closes.len() - self.lookback;
        let slice = &self.closes[start..];
        let slice_len = usize_to_f64(slice.len());
        let mean = slice.iter().sum::<f64>() / slice_len;
        let variance = slice
            .iter()
            .map(|value| {
                let delta = value - mean;
                delta * delta
            })
            .sum::<f64>()
            / slice_len;
        let stdev = variance.sqrt().max(1e-9);
        let z = (price - mean) / stdev;
        let side = if z > self.z_threshold {
            SignalSideV1::Sell
        } else if z < -self.z_threshold {
            SignalSideV1::Buy
        } else {
            SignalSideV1::Hold
        };

        Ok(vec![SignalV1 {
            strategy_id: self.strategy_id.clone(),
            symbol: event.symbol().clone(),
            side,
            quantity: self.trade_size,
            confidence: z.abs(),
            reason: format!("mean_reversion(z={z:.4})"),
            determinism,
        }])
    }

    fn on_timer(
        &mut self,
        _now: DateTime<Utc>,
        _determinism: DeterminismKeyV1,
    ) -> TradingResult<Vec<SignalV1>> {
        Ok(Vec::new())
    }

    fn snapshot_state(&self, now: DateTime<Utc>) -> StrategyStateSnapshotV1 {
        StrategyStateSnapshotV1 {
            strategy_id: self.strategy_id.clone(),
            timestamp: now,
            state: serde_json::json!({
                "lookback": self.lookback,
                "z_threshold": self.z_threshold,
                "close_count": self.closes.len(),
            }),
        }
    }
}

#[derive(Clone)]
pub struct DeterministicBacktestEngine {
    clock: Arc<dyn Clock>,
    ids: Arc<dyn IdGenerator>,
}

impl std::fmt::Debug for DeterministicBacktestEngine {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("DeterministicBacktestEngine")
            .finish_non_exhaustive()
    }
}

impl Default for DeterministicBacktestEngine {
    fn default() -> Self {
        Self {
            clock: Arc::new(SystemClock),
            ids: Arc::new(SystemIdGenerator),
        }
    }
}

impl DeterministicBacktestEngine {
    #[must_use]
    pub fn new(clock: Arc<dyn Clock>, ids: Arc<dyn IdGenerator>) -> Self {
        Self { clock, ids }
    }

    fn apply_slippage(price: f64, side: SignalSideV1, slippage_bps: f64) -> f64 {
        let slip = slippage_bps / 10_000.0;
        match side {
            SignalSideV1::Buy => price * (1.0 + slip),
            SignalSideV1::Sell => price * (1.0 - slip),
            SignalSideV1::Hold => price,
        }
    }

    fn fee(notional: f64, fee_bps: f64) -> f64 {
        notional.abs() * fee_bps / 10_000.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SweepJob {
    pub strategy_id: String,
    pub lookback: usize,
    pub trade_size: f64,
    pub config_hash: String,
}

pub fn run_trend_sweep(
    engine: &DeterministicBacktestEngine,
    events: &[MarketEventV1],
    jobs: &[SweepJob],
    simulation: &SimulationConfigV1,
) -> TradingResult<Vec<BacktestResultV1>> {
    let mut out = Vec::with_capacity(jobs.len());
    for job in jobs {
        let mut strategy =
            TrendFollower::new(job.strategy_id.clone(), job.lookback, job.trade_size);
        out.push(engine.run(&mut strategy, events, &job.config_hash, simulation)?);
    }
    Ok(out)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentBehavior {
    pub name: String,
    pub inventory_bias: f64,
    pub aggressiveness: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMarketTick {
    pub timestamp: DateTime<Utc>,
    pub mid_price: f64,
    pub volume: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMarketSimulation {
    pub ticks: Vec<AgentMarketTick>,
}

#[must_use]
pub fn run_agent_market_simulation(
    start: DateTime<Utc>,
    initial_mid: f64,
    steps: usize,
    agents: &[AgentBehavior],
) -> AgentMarketSimulation {
    let mut ticks = VecDeque::with_capacity(steps);
    let mut mid = initial_mid;

    for index in 0..steps {
        let net_pressure = agents
            .iter()
            .map(|agent| agent.inventory_bias * agent.aggressiveness)
            .sum::<f64>();
        let impact = net_pressure * 0.0001;
        mid = (mid * (1.0 + impact)).max(0.0001);
        let volume = agents
            .iter()
            .map(|agent| agent.aggressiveness * 100.0)
            .sum::<f64>();
        ticks.push_back(AgentMarketTick {
            timestamp: start + chrono::Duration::seconds(usize_to_i64(index)),
            mid_price: mid,
            volume,
        });
    }

    AgentMarketSimulation {
        ticks: ticks.into_iter().collect(),
    }
}

impl BacktestEngine for DeterministicBacktestEngine {
    fn run(
        &self,
        strategy: &mut dyn StrategyModule,
        events: &[MarketEventV1],
        config_hash: &str,
        simulation: &SimulationConfigV1,
    ) -> TradingResult<BacktestResultV1> {
        let strategy_id = strategy.snapshot_state(self.clock.now()).strategy_id;
        strategy.init(StrategyConfigV1 {
            strategy_id,
            model_version: "v1".to_string(),
            config_hash: config_hash.to_string(),
            parameters: serde_json::json!({}),
        })?;

        let mut ledger = DoubleEntryLedger::new(simulation.initial_cash, Arc::clone(&self.clock));
        let mut trade_count = 0usize;
        let mut turnover = 0.0;
        let mut equity_curve = Vec::new();

        for (idx, event) in events.iter().enumerate() {
            let signals = strategy.on_market_event(
                event,
                DeterminismKeyV1::new(format!("event-{}", idx + 1), "v1", config_hash),
            )?;

            for signal in signals {
                if signal.side == SignalSideV1::Hold || signal.quantity <= 0.0 {
                    continue;
                }

                let Some(order) =
                    build_order_request(&signal, self.ids.as_ref(), self.clock.as_ref())
                else {
                    continue;
                };

                let fill_price =
                    Self::apply_slippage(event.price(), signal.side, simulation.slippage_bps);
                let notional = fill_price * order.quantity;
                turnover += notional;
                let fee = Self::fee(notional, simulation.fee_bps);
                let fill = FillV1 {
                    fill_id: self.ids.next_id(),
                    order_id: order.order_id.clone(),
                    symbol: order.symbol.clone(),
                    side: order.side,
                    quantity: order.quantity,
                    price: fill_price,
                    fee,
                    timestamp: event.event_time(),
                };

                ledger.apply_fill(&fill)?;
                trade_count += 1;
            }

            ledger.mark_to_market(&[(event.symbol().clone(), event.price())])?;
            let snapshot = ledger.snapshot()?;
            equity_curve.push(EquityPointV1 {
                timestamp: event.event_time(),
                equity: snapshot.cash + snapshot.unrealized_pnl,
            });
        }

        Ok(BacktestResultV1 {
            config_hash: config_hash.to_string(),
            seed: simulation.seed,
            summary: summarize(&equity_curve, turnover, simulation.initial_cash),
            equity_curve,
            trade_count,
        })
    }

    fn replay(&self, serialized: &str) -> TradingResult<BacktestResultV1> {
        Ok(serde_json::from_str(serialized)?)
    }

    fn serialize(&self, result: &BacktestResultV1) -> TradingResult<String> {
        Ok(serde_json::to_string_pretty(result)?)
    }
}

fn summarize(curve: &[EquityPointV1], turnover: f64, initial_cash: f64) -> PerformanceSummaryV1 {
    if curve.is_empty() {
        return PerformanceSummaryV1 {
            total_return: 0.0,
            sharpe: 0.0,
            max_drawdown: 0.0,
            turnover: 0.0,
        };
    }

    let first = curve.first().map_or(initial_cash, |point| point.equity);
    let last = curve.last().map_or(initial_cash, |point| point.equity);
    let total_return = if first.abs() > f64::EPSILON {
        (last / first) - 1.0
    } else {
        0.0
    };

    let returns: Vec<f64> = curve
        .windows(2)
        .filter_map(|window| {
            if window[0].equity.abs() > f64::EPSILON {
                Some((window[1].equity / window[0].equity) - 1.0)
            } else {
                None
            }
        })
        .collect();

    let sharpe = if returns.len() > 1 {
        let returns_len = usize_to_f64(returns.len());
        let mean = returns.iter().sum::<f64>() / returns_len;
        let variance = returns
            .iter()
            .map(|ret| {
                let delta = ret - mean;
                delta * delta
            })
            .sum::<f64>()
            / returns_len;
        let stdev = variance.sqrt();
        if stdev > 0.0 {
            mean / stdev * (252.0f64).sqrt()
        } else {
            0.0
        }
    } else {
        0.0
    };

    let mut peak = curve[0].equity;
    let mut max_drawdown = 0.0_f64;
    for point in curve {
        peak = peak.max(point.equity);
        if peak > 0.0 {
            max_drawdown = max_drawdown.max((peak - point.equity) / peak);
        }
    }

    PerformanceSummaryV1 {
        total_return,
        sharpe,
        max_drawdown,
        turnover: if initial_cash > 0.0 {
            turnover / initial_cash
        } else {
            0.0
        },
    }
}

#[derive(Debug, Clone)]
pub struct PaperVenueConfig {
    pub base_price: f64,
    pub fee_bps: f64,
    pub partial_fill_ratio: f64,
    pub reject_every: Option<u64>,
    pub rate_limit_every: Option<u64>,
    pub disconnect_every: Option<u64>,
    pub stale_book_threshold_bps: f64,
}

impl Default for PaperVenueConfig {
    fn default() -> Self {
        Self {
            base_price: 100.0,
            fee_bps: 1.0,
            partial_fill_ratio: 1.0,
            reject_every: None,
            rate_limit_every: None,
            disconnect_every: None,
            stale_book_threshold_bps: 80.0,
        }
    }
}

#[derive(Clone)]
pub struct PaperVenueAdapter {
    venue: VenueV1,
    config: PaperVenueConfig,
    request_count: u64,
    order_status: HashMap<String, OrderStatusV1>,
    pending_fills: Vec<FillV1>,
    clock: Arc<dyn Clock>,
    ids: Arc<dyn IdGenerator>,
}

impl std::fmt::Debug for PaperVenueAdapter {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PaperVenueAdapter")
            .field("venue", &self.venue)
            .field("request_count", &self.request_count)
            .finish_non_exhaustive()
    }
}

fn usize_to_f64(value: usize) -> f64 {
    f64::from(u32::try_from(value).unwrap_or(u32::MAX))
}

fn usize_to_i64(value: usize) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

impl PaperVenueAdapter {
    #[must_use]
    pub fn new(
        venue: VenueV1,
        config: PaperVenueConfig,
        clock: Arc<dyn Clock>,
        ids: Arc<dyn IdGenerator>,
    ) -> Self {
        Self {
            venue,
            config,
            request_count: 0,
            order_status: HashMap::new(),
            pending_fills: Vec::new(),
            clock,
            ids,
        }
    }

    fn mark_price(&self) -> f64 {
        self.config.base_price
    }
}

impl ExecutionVenueAdapter for PaperVenueAdapter {
    fn submit_order(
        &mut self,
        order: &contracts::OrderRequestV1,
    ) -> TradingResult<contracts::OrderAckV1> {
        self.request_count += 1;

        if self
            .config
            .disconnect_every
            .is_some_and(|count| self.request_count.is_multiple_of(count))
        {
            return Err(trading_errors::TradingError::RuntimePolicyViolation {
                details: "simulated disconnect".to_string(),
            });
        }

        if self
            .config
            .rate_limit_every
            .is_some_and(|count| self.request_count.is_multiple_of(count))
        {
            let status = OrderStatusV1::Rejected("rate_limited".to_string());
            self.order_status
                .insert(order.order_id.clone(), status.clone());
            return Ok(contracts::OrderAckV1 {
                order_id: order.order_id.clone(),
                status,
                venue_order_id: None,
                message: "rate limit".to_string(),
            });
        }

        if self
            .config
            .reject_every
            .is_some_and(|count| self.request_count.is_multiple_of(count))
        {
            let status = OrderStatusV1::Rejected("venue_reject".to_string());
            self.order_status
                .insert(order.order_id.clone(), status.clone());
            return Ok(contracts::OrderAckV1 {
                order_id: order.order_id.clone(),
                status,
                venue_order_id: None,
                message: "simulated reject".to_string(),
            });
        }

        let mark = self.mark_price();
        if let Some(limit) = order.limit_price {
            let deviation_bps = ((limit - mark).abs() / mark.max(1e-9)) * 10_000.0;
            if deviation_bps > self.config.stale_book_threshold_bps {
                let status = OrderStatusV1::Rejected("stale_book_protection".to_string());
                self.order_status
                    .insert(order.order_id.clone(), status.clone());
                return Ok(contracts::OrderAckV1 {
                    order_id: order.order_id.clone(),
                    status,
                    venue_order_id: None,
                    message: "stale book protection".to_string(),
                });
            }
        }

        let ratio = self.config.partial_fill_ratio.clamp(0.0, 1.0);
        let fill_qty = (order.quantity * ratio).max(0.0);
        let fee = fill_qty * mark * self.config.fee_bps / 10_000.0;
        let status = if ratio < 1.0 {
            OrderStatusV1::PartiallyFilled
        } else {
            OrderStatusV1::Filled
        };
        self.order_status
            .insert(order.order_id.clone(), status.clone());

        if fill_qty > 0.0 {
            self.pending_fills.push(FillV1 {
                fill_id: self.ids.next_id(),
                order_id: order.order_id.clone(),
                symbol: order.symbol.clone(),
                side: order.side,
                quantity: fill_qty,
                price: mark,
                fee,
                timestamp: self.clock.now(),
            });
        }

        Ok(contracts::OrderAckV1 {
            order_id: order.order_id.clone(),
            status,
            venue_order_id: Some(format!("{:?}-{}", self.venue, order.order_id)),
            message: "accepted".to_string(),
        })
    }

    fn cancel_order(&mut self, order_id: &str) -> TradingResult<contracts::OrderAckV1> {
        self.order_status
            .insert(order_id.to_string(), OrderStatusV1::Cancelled);
        Ok(contracts::OrderAckV1 {
            order_id: order_id.to_string(),
            status: OrderStatusV1::Cancelled,
            venue_order_id: None,
            message: "cancelled".to_string(),
        })
    }

    fn amend_order(
        &mut self,
        order: &contracts::OrderRequestV1,
    ) -> TradingResult<contracts::OrderAckV1> {
        self.submit_order(order)
    }

    fn query_order_state(&self, order_id: &str) -> TradingResult<OrderStatusV1> {
        Ok(self
            .order_status
            .get(order_id)
            .cloned()
            .unwrap_or(OrderStatusV1::Cancelled))
    }

    fn reconcile_fills(&mut self) -> TradingResult<Vec<FillV1>> {
        Ok(std::mem::take(&mut self.pending_fills))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chrono::{Duration, TimeZone, Utc};
    use contracts::{
        AssetClassV1, MarketEventV1, OhlcvBarV1, OrderRequestV1, OrderTypeV1, Side,
        SimulationConfigV1, SymbolV1, TimeInForceV1, VenueV1,
    };
    use trading_core::{BacktestEngine, ExecutionVenueAdapter, FixedClock, SequenceIdGenerator};

    use super::{
        AgentBehavior, DeterministicBacktestEngine, PaperVenueAdapter, PaperVenueConfig, SweepJob,
        TrendFollower, run_agent_market_simulation, run_trend_sweep,
    };

    fn sample_events() -> Vec<MarketEventV1> {
        let symbol = SymbolV1::new(VenueV1::Coinbase, AssetClassV1::Crypto, "BTC", "USD");
        let start = Utc
            .with_ymd_and_hms(2026, 3, 1, 0, 0, 0)
            .single()
            .expect("start");
        let mut events = Vec::new();
        for index in 0..120 {
            let price = 100.0 + (f64::from(i32::try_from(index).unwrap_or(i32::MAX)) * 0.1);
            events.push(MarketEventV1::Bar(OhlcvBarV1 {
                symbol: symbol.clone(),
                open_time: start + Duration::minutes(index),
                close_time: start + Duration::minutes(index + 1),
                open: price - 0.1,
                high: price + 0.2,
                low: price - 0.2,
                close: price,
                volume: 1000.0,
            }));
        }
        events
    }

    fn sample_order(limit_price: Option<f64>) -> OrderRequestV1 {
        OrderRequestV1 {
            order_id: "order-1".to_string(),
            strategy_id: "s1".to_string(),
            symbol: SymbolV1::new(VenueV1::Coinbase, AssetClassV1::Crypto, "BTC", "USD"),
            venue: VenueV1::Coinbase,
            side: Side::Buy,
            quantity: 1.0,
            limit_price,
            order_type: if limit_price.is_some() {
                OrderTypeV1::Limit
            } else {
                OrderTypeV1::Market
            },
            tif: TimeInForceV1::Ioc,
            submitted_at: Utc
                .with_ymd_and_hms(2026, 3, 1, 0, 0, 0)
                .single()
                .expect("submitted"),
        }
    }

    #[test]
    fn replay_is_deterministic() {
        let clock = Arc::new(FixedClock::new(
            Utc.with_ymd_and_hms(2026, 3, 1, 0, 0, 0)
                .single()
                .expect("clock"),
        ));
        let ids = Arc::new(SequenceIdGenerator::new("bt"));
        let engine = DeterministicBacktestEngine::new(clock, ids);
        let mut strategy = TrendFollower::new("trend", 10, 0.1);
        let result = engine
            .run(
                &mut strategy,
                &sample_events(),
                "cfg_hash",
                &SimulationConfigV1 {
                    seed: 7,
                    fee_bps: 1.0,
                    slippage_bps: 2.0,
                    latency_ms: 5,
                    initial_cash: 100_000.0,
                },
            )
            .expect("run");
        let serialized = engine.serialize(&result).expect("serialize");
        let replayed = engine.replay(&serialized).expect("replay");

        assert_eq!(result.trade_count, replayed.trade_count);
        assert_eq!(result.equity_curve.len(), replayed.equity_curve.len());
        assert!((result.summary.total_return - replayed.summary.total_return).abs() < 1e-12);
    }

    #[test]
    fn trend_sweep_runs_multiple_jobs() {
        let engine = DeterministicBacktestEngine::default();
        let jobs = vec![
            SweepJob {
                strategy_id: "t1".to_string(),
                lookback: 5,
                trade_size: 0.1,
                config_hash: "a".to_string(),
            },
            SweepJob {
                strategy_id: "t2".to_string(),
                lookback: 10,
                trade_size: 0.1,
                config_hash: "b".to_string(),
            },
        ];
        let results = run_trend_sweep(
            &engine,
            &sample_events(),
            &jobs,
            &SimulationConfigV1 {
                seed: 11,
                fee_bps: 1.0,
                slippage_bps: 1.0,
                latency_ms: 5,
                initial_cash: 100_000.0,
            },
        )
        .expect("sweep");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn agent_market_simulation_outputs_ticks() {
        let simulation = run_agent_market_simulation(
            Utc.with_ymd_and_hms(2026, 3, 1, 0, 0, 0)
                .single()
                .expect("start"),
            100.0,
            30,
            &[AgentBehavior {
                name: "maker".to_string(),
                inventory_bias: 0.2,
                aggressiveness: 0.8,
            }],
        );
        assert_eq!(simulation.ticks.len(), 30);
    }

    #[test]
    fn partial_fill_is_supported() {
        let clock = Arc::new(FixedClock::new(
            Utc.with_ymd_and_hms(2026, 3, 1, 0, 0, 0)
                .single()
                .expect("clock"),
        ));
        let ids = Arc::new(SequenceIdGenerator::new("fill"));
        let mut adapter = PaperVenueAdapter::new(
            VenueV1::Coinbase,
            PaperVenueConfig {
                partial_fill_ratio: 0.5,
                ..PaperVenueConfig::default()
            },
            clock,
            ids,
        );

        let order = sample_order(None);
        let ack = adapter.submit_order(&order).expect("submit");
        assert!(matches!(
            ack.status,
            contracts::OrderStatusV1::PartiallyFilled
        ));
        let fills = adapter.reconcile_fills().expect("fills");
        assert_eq!(fills.len(), 1);
        assert!(fills[0].quantity < order.quantity);
    }

    #[test]
    fn venue_reject_and_rate_limit_paths() {
        let clock = Arc::new(FixedClock::new(
            Utc.with_ymd_and_hms(2026, 3, 1, 0, 0, 0)
                .single()
                .expect("clock"),
        ));
        let ids = Arc::new(SequenceIdGenerator::new("order"));
        let mut adapter = PaperVenueAdapter::new(
            VenueV1::Coinbase,
            PaperVenueConfig {
                reject_every: Some(2),
                rate_limit_every: Some(3),
                ..PaperVenueConfig::default()
            },
            clock,
            ids,
        );

        let a1 = adapter.submit_order(&sample_order(None)).expect("a1");
        let a2 = adapter.submit_order(&sample_order(None)).expect("a2");
        let a3 = adapter.submit_order(&sample_order(None)).expect("a3");

        assert!(matches!(a1.status, contracts::OrderStatusV1::Filled));
        assert!(matches!(a2.status, contracts::OrderStatusV1::Rejected(_)));
        assert!(matches!(a3.status, contracts::OrderStatusV1::Rejected(_)));
    }

    #[test]
    fn disconnect_path_returns_error() {
        let clock = Arc::new(FixedClock::new(
            Utc.with_ymd_and_hms(2026, 3, 1, 0, 0, 0)
                .single()
                .expect("clock"),
        ));
        let ids = Arc::new(SequenceIdGenerator::new("order"));
        let mut adapter = PaperVenueAdapter::new(
            VenueV1::Coinbase,
            PaperVenueConfig {
                disconnect_every: Some(1),
                ..PaperVenueConfig::default()
            },
            clock,
            ids,
        );

        let error = adapter
            .submit_order(&sample_order(None))
            .expect_err("disconnect");
        assert!(error.to_string().contains("disconnect"));
    }

    #[test]
    fn stale_book_protection_rejects_far_limit() {
        let clock = Arc::new(FixedClock::new(
            Utc.with_ymd_and_hms(2026, 3, 1, 0, 0, 0)
                .single()
                .expect("clock"),
        ));
        let ids = Arc::new(SequenceIdGenerator::new("order"));
        let mut adapter = PaperVenueAdapter::new(
            VenueV1::Coinbase,
            PaperVenueConfig {
                stale_book_threshold_bps: 10.0,
                base_price: 100.0,
                ..PaperVenueConfig::default()
            },
            clock,
            ids,
        );

        let ack = adapter
            .submit_order(&sample_order(Some(120.0)))
            .expect("submit");
        assert!(matches!(ack.status, contracts::OrderStatusV1::Rejected(_)));
    }
}
