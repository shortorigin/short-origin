use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use chrono::{DateTime, Utc};
use contracts::{
    ExperimentConfigV1, FeatureRowV1, FillV1, HealthStatusV1, HistoricalDataRequestV1,
    LedgerAccountV1, LedgerEntryV1, MarketEventV1, OrderAckV1, OrderRequestV1, OrderSideV1,
    OrderStatusV1, PortfolioSnapshotV1, PositionSnapshotV1, RawMarketRecordV1, SignalSideV1,
    SignalV1, StrategyConfigV1, StrategyStateSnapshotV1, SymbolV1, TradingRiskSnapshotV1,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use trading_errors::{TradingError, TradingResult};
use uuid::Uuid;

pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

#[derive(Debug, Clone)]
pub struct FixedClock {
    instant: DateTime<Utc>,
}

impl FixedClock {
    #[must_use]
    pub fn new(instant: DateTime<Utc>) -> Self {
        Self { instant }
    }
}

impl Clock for FixedClock {
    fn now(&self) -> DateTime<Utc> {
        self.instant
    }
}

pub trait IdGenerator: Send + Sync {
    fn next_id(&self) -> String;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SystemIdGenerator;

impl IdGenerator for SystemIdGenerator {
    fn next_id(&self) -> String {
        Uuid::new_v4().to_string()
    }
}

#[derive(Debug)]
pub struct SequenceIdGenerator {
    prefix: String,
    counter: AtomicU64,
}

impl SequenceIdGenerator {
    #[must_use]
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
            counter: AtomicU64::new(0),
        }
    }
}

impl IdGenerator for SequenceIdGenerator {
    fn next_id(&self) -> String {
        let next = self.counter.fetch_add(1, Ordering::Relaxed) + 1;
        format!("{}-{next}", self.prefix)
    }
}

pub trait MarketDataAdapter: Send + Sync {
    fn fetch_historical(&self, req: HistoricalDataRequestV1) -> TradingResult<Vec<MarketEventV1>>;
    fn stream_realtime(&self, symbol: SymbolV1) -> TradingResult<Vec<MarketEventV1>>;
    fn health_check(&self) -> HealthStatusV1;
    fn normalize_to_domain(&self, raw: Vec<RawMarketRecordV1>)
    -> TradingResult<Vec<MarketEventV1>>;
}

pub trait StrategyModule: Send {
    fn init(&mut self, config: StrategyConfigV1) -> TradingResult<()>;
    fn on_market_event(
        &mut self,
        event: &MarketEventV1,
        determinism: contracts::DeterminismKeyV1,
    ) -> TradingResult<Vec<SignalV1>>;
    fn on_timer(
        &mut self,
        now: DateTime<Utc>,
        determinism: contracts::DeterminismKeyV1,
    ) -> TradingResult<Vec<SignalV1>>;
    fn snapshot_state(&self, now: DateTime<Utc>) -> StrategyStateSnapshotV1;
}

pub trait BacktestEngine {
    fn run(
        &self,
        strategy: &mut dyn StrategyModule,
        events: &[MarketEventV1],
        config_hash: &str,
        simulation: &contracts::SimulationConfigV1,
    ) -> TradingResult<contracts::BacktestResultV1>;

    fn replay(&self, serialized: &str) -> TradingResult<contracts::BacktestResultV1>;
    fn serialize(&self, result: &contracts::BacktestResultV1) -> TradingResult<String>;
}

pub trait ExecutionVenueAdapter: Send + Sync {
    fn submit_order(&mut self, order: &OrderRequestV1) -> TradingResult<OrderAckV1>;
    fn cancel_order(&mut self, order_id: &str) -> TradingResult<OrderAckV1>;
    fn amend_order(&mut self, order: &OrderRequestV1) -> TradingResult<OrderAckV1>;
    fn query_order_state(&self, order_id: &str) -> TradingResult<OrderStatusV1>;
    fn reconcile_fills(&mut self) -> TradingResult<Vec<FillV1>>;
}

pub trait PortfolioLedger: Send {
    fn post_entry(&mut self, entry: LedgerEntryV1) -> TradingResult<()>;
    fn apply_fill(&mut self, fill: &FillV1) -> TradingResult<()>;
    fn mark_to_market(&mut self, marks: &[(SymbolV1, f64)]) -> TradingResult<()>;
    fn snapshot(&self) -> TradingResult<PortfolioSnapshotV1>;
    fn entries(&self) -> &[LedgerEntryV1];
}

pub trait RiskPolicyEngine: Send {
    fn pre_trade_check(
        &mut self,
        order: &OrderRequestV1,
    ) -> TradingResult<contracts::RiskDecisionV1>;
    fn observe_fill(&mut self, _fill: &FillV1) -> TradingResult<()> {
        Ok(())
    }
    fn intra_day_limits(&self) -> TradingResult<TradingRiskSnapshotV1>;
    fn kill_switch(&mut self, reason: &str) -> TradingResult<()>;
    fn post_trade_exceptions(&self) -> TradingResult<Vec<String>>;
}

#[must_use]
pub fn build_determinism_key(
    ids: &dyn IdGenerator,
    model_version: impl Into<String>,
    config_hash: impl Into<String>,
) -> contracts::DeterminismKeyV1 {
    contracts::DeterminismKeyV1::new(ids.next_id(), model_version, config_hash)
}

#[must_use]
pub fn build_order_request(
    signal: &SignalV1,
    ids: &dyn IdGenerator,
    clock: &dyn Clock,
) -> Option<OrderRequestV1> {
    let side = match signal.side {
        SignalSideV1::Buy => OrderSideV1::Buy,
        SignalSideV1::Sell => OrderSideV1::Sell,
        SignalSideV1::Hold => return None,
    };

    Some(OrderRequestV1 {
        order_id: ids.next_id(),
        strategy_id: signal.strategy_id.clone(),
        symbol: signal.symbol.clone(),
        venue: signal.symbol.venue,
        side,
        quantity: signal.quantity,
        limit_price: None,
        order_type: contracts::OrderTypeV1::Market,
        tif: contracts::TimeInForceV1::Ioc,
        submitted_at: clock.now(),
    })
}

pub fn hash_payload<T: Serialize>(payload: &T) -> TradingResult<String> {
    let encoded = serde_json::to_vec(payload)?;
    let mut hasher = Sha256::new();
    hasher.update(encoded);
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn experiment_config_hash(config: &ExperimentConfigV1) -> TradingResult<String> {
    hash_payload(config)
}

#[derive(Debug, Clone, PartialEq)]
pub struct WalkForwardSplit {
    pub train: Vec<FeatureRowV1>,
    pub test: Vec<FeatureRowV1>,
}

pub fn build_feature_rows(events: &[MarketEventV1]) -> Vec<FeatureRowV1> {
    let mut rows = Vec::new();
    let mut closes = Vec::new();

    for event in events {
        let close = event.price();
        closes.push(close);
        let idx = closes.len() - 1;
        let ret = if idx > 0 {
            (close / closes[idx - 1]) - 1.0
        } else {
            0.0
        };
        let vol = rolling_volatility(&closes, 20);
        let micro = microstructure_proxy(event, ret);

        rows.push(FeatureRowV1 {
            timestamp: event.event_time(),
            symbol: event.symbol().ticker(),
            close,
            return_1: ret,
            rolling_vol_20: vol,
            microstructure_proxy: micro,
        });
    }

    rows
}

pub fn walk_forward(
    rows: &[FeatureRowV1],
    train_window: usize,
    test_window: usize,
) -> TradingResult<Vec<WalkForwardSplit>> {
    if train_window == 0 || test_window == 0 {
        return Err(TradingError::InvalidInput {
            details: "train_window and test_window must be > 0".to_string(),
        });
    }

    let mut out = Vec::new();
    let mut start = 0usize;
    while start + train_window + test_window <= rows.len() {
        out.push(WalkForwardSplit {
            train: rows[start..start + train_window].to_vec(),
            test: rows[start + train_window..start + train_window + test_window].to_vec(),
        });
        start += test_window;
    }

    Ok(out)
}

#[derive(Debug, Clone)]
pub struct BasicLinearModel {
    pub weights: [f64; 3],
}

impl BasicLinearModel {
    #[must_use]
    pub fn fit(train: &[FeatureRowV1]) -> Self {
        if train.is_empty() {
            return Self { weights: [0.0; 3] };
        }

        let train_len = usize_to_f64(train.len());
        let avg_ret = train.iter().map(|row| row.return_1).sum::<f64>() / train_len;
        let avg_vol = train.iter().map(|row| row.rolling_vol_20).sum::<f64>() / train_len;
        let avg_micro = train
            .iter()
            .map(|row| row.microstructure_proxy)
            .sum::<f64>()
            / train_len;

        Self {
            weights: [avg_ret * 10.0, -avg_vol, avg_micro * 5.0],
        }
    }

    #[must_use]
    pub fn score(&self, row: &FeatureRowV1) -> f64 {
        self.weights[0] * row.return_1
            + self.weights[1] * row.rolling_vol_20
            + self.weights[2] * row.microstructure_proxy
    }
}

fn rolling_volatility(closes: &[f64], lookback: usize) -> f64 {
    let len = closes.len();
    if len <= 1 {
        return 0.0;
    }

    let start = len.saturating_sub(lookback);
    let slice = &closes[start..];
    if slice.len() <= 1 {
        return 0.0;
    }

    let returns: Vec<f64> = slice
        .windows(2)
        .map(|pair| (pair[1] / pair[0]) - 1.0)
        .collect();
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
    variance.sqrt()
}

fn usize_to_f64(value: usize) -> f64 {
    f64::from(u32::try_from(value).unwrap_or(u32::MAX))
}

fn microstructure_proxy(event: &MarketEventV1, ret: f64) -> f64 {
    let volume = match event {
        MarketEventV1::Bar(bar) => bar.volume,
        MarketEventV1::Trade(tick) => tick.size,
    };

    ret.abs() * volume.max(1.0).ln()
}

#[derive(Debug, Clone)]
struct PositionState {
    qty: f64,
    avg_price: f64,
    mark_price: f64,
}

#[derive(Clone)]
pub struct DoubleEntryLedger {
    cash: f64,
    realized_pnl: f64,
    entries: Vec<LedgerEntryV1>,
    positions: HashMap<String, (SymbolV1, PositionState)>,
    clock: Arc<dyn Clock>,
}

impl std::fmt::Debug for DoubleEntryLedger {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("DoubleEntryLedger")
            .field("cash", &self.cash)
            .field("realized_pnl", &self.realized_pnl)
            .field("entries", &self.entries.len())
            .field("positions", &self.positions.len())
            .finish_non_exhaustive()
    }
}

impl DoubleEntryLedger {
    #[must_use]
    pub fn new(initial_cash: f64, clock: Arc<dyn Clock>) -> Self {
        Self {
            cash: initial_cash,
            realized_pnl: 0.0,
            entries: Vec::new(),
            positions: HashMap::new(),
            clock,
        }
    }

    fn key(symbol: &SymbolV1) -> String {
        format!("{:?}:{}", symbol.venue, symbol.ticker())
    }
}

impl PortfolioLedger for DoubleEntryLedger {
    fn post_entry(&mut self, entry: LedgerEntryV1) -> TradingResult<()> {
        self.entries.push(entry);
        Ok(())
    }

    fn apply_fill(&mut self, fill: &FillV1) -> TradingResult<()> {
        let key = Self::key(&fill.symbol);
        let cash_delta = match fill.side {
            OrderSideV1::Buy => -(fill.price * fill.quantity + fill.fee),
            OrderSideV1::Sell => fill.price * fill.quantity - fill.fee,
        };

        let symbol_for_entries = {
            let (symbol, state) = self.positions.entry(key).or_insert_with(|| {
                (
                    fill.symbol.clone(),
                    PositionState {
                        qty: 0.0,
                        avg_price: fill.price,
                        mark_price: fill.price,
                    },
                )
            });

            let direction = match fill.side {
                OrderSideV1::Buy => 1.0,
                OrderSideV1::Sell => -1.0,
            };

            let signed_qty = direction * fill.quantity;
            let prev_qty = state.qty;
            let new_qty = prev_qty + signed_qty;

            if prev_qty.signum() != 0.0 && prev_qty.signum() != new_qty.signum() {
                self.realized_pnl += (fill.price - state.avg_price) * prev_qty;
                state.avg_price = fill.price;
            } else if signed_qty.signum() == prev_qty.signum() || prev_qty.abs() < f64::EPSILON {
                let notional = (state.avg_price * prev_qty.abs()) + (fill.price * fill.quantity);
                let denom = prev_qty.abs() + fill.quantity;
                if denom > 0.0 {
                    state.avg_price = notional / denom;
                }
            } else {
                self.realized_pnl += (fill.price - state.avg_price)
                    * signed_qty.abs().min(prev_qty.abs())
                    * prev_qty.signum();
                if new_qty.abs() < f64::EPSILON {
                    state.avg_price = fill.price;
                }
            }

            state.qty = new_qty;
            state.mark_price = fill.price;
            symbol.clone()
        };

        self.cash += cash_delta;
        self.post_entry(LedgerEntryV1 {
            timestamp: fill.timestamp,
            account: LedgerAccountV1::Cash,
            symbol: Some(symbol_for_entries.clone()),
            amount: cash_delta,
            description: format!("fill {}", fill.order_id),
        })?;
        self.post_entry(LedgerEntryV1 {
            timestamp: fill.timestamp,
            account: LedgerAccountV1::Fees,
            symbol: Some(symbol_for_entries),
            amount: -fill.fee,
            description: format!("fee {}", fill.order_id),
        })?;

        Ok(())
    }

    fn mark_to_market(&mut self, marks: &[(SymbolV1, f64)]) -> TradingResult<()> {
        for (symbol, mark) in marks {
            let key = Self::key(symbol);
            if let Some((_stored_symbol, state)) = self.positions.get_mut(&key) {
                state.mark_price = *mark;
            }
        }
        Ok(())
    }

    fn snapshot(&self) -> TradingResult<PortfolioSnapshotV1> {
        let mut positions = Vec::new();
        let mut unrealized = 0.0;

        for (symbol, state) in self.positions.values() {
            let unrealized_pnl = (state.mark_price - state.avg_price) * state.qty;
            unrealized += unrealized_pnl;
            positions.push(PositionSnapshotV1 {
                symbol: symbol.clone(),
                quantity: state.qty,
                avg_price: state.avg_price,
                mark_price: state.mark_price,
                unrealized_pnl,
            });
        }

        Ok(PortfolioSnapshotV1 {
            as_of: self.clock.now(),
            cash: self.cash,
            realized_pnl: self.realized_pnl,
            unrealized_pnl: unrealized,
            positions,
        })
    }

    fn entries(&self) -> &[LedgerEntryV1] {
        &self.entries
    }
}

pub fn reconcile_snapshot(snapshot: &PortfolioSnapshotV1) -> TradingResult<()> {
    if !snapshot.cash.is_finite() {
        return Err(TradingError::InvalidInput {
            details: "cash is not finite".to_string(),
        });
    }
    if !snapshot.realized_pnl.is_finite() || !snapshot.unrealized_pnl.is_finite() {
        return Err(TradingError::InvalidInput {
            details: "pnl is not finite".to_string(),
        });
    }
    for position in &snapshot.positions {
        if !position.quantity.is_finite() || !position.avg_price.is_finite() {
            return Err(TradingError::InvalidInput {
                details: "invalid position numeric state".to_string(),
            });
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub struct StrategyAllocationRule {
    pub strategy_id: String,
    pub max_notional: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllocatedSignal {
    pub strategy_id: String,
    pub symbol: SymbolV1,
    pub requested_qty: f64,
    pub approved_qty: f64,
}

#[derive(Debug, Clone)]
pub struct MultiStrategyAllocator {
    total_capital: f64,
    rules: HashMap<String, StrategyAllocationRule>,
}

impl MultiStrategyAllocator {
    #[must_use]
    pub fn new(total_capital: f64, rules: Vec<StrategyAllocationRule>) -> Self {
        Self {
            total_capital,
            rules: rules
                .into_iter()
                .map(|rule| (rule.strategy_id.clone(), rule))
                .collect(),
        }
    }

    #[must_use]
    pub fn allocate(
        &self,
        signals: &[SignalV1],
        marks: &HashMap<String, f64>,
    ) -> Vec<AllocatedSignal> {
        signals
            .iter()
            .map(|signal| {
                let mark = marks
                    .get(&signal.symbol.ticker())
                    .copied()
                    .unwrap_or(1.0)
                    .max(1e-9);
                let requested_notional = signal.quantity * mark;
                let allowed = self
                    .rules
                    .get(&signal.strategy_id)
                    .map_or(self.total_capital, |rule| rule.max_notional)
                    .min(self.total_capital);
                let approved_qty = if requested_notional <= allowed {
                    signal.quantity
                } else {
                    allowed / mark
                };

                AllocatedSignal {
                    strategy_id: signal.strategy_id.clone(),
                    symbol: signal.symbol.clone(),
                    requested_qty: signal.quantity,
                    approved_qty,
                }
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitConfig {
    pub max_order_notional: f64,
    pub max_gross_exposure: f64,
    pub max_open_orders: usize,
}

#[derive(Clone)]
pub struct LimitRiskEngine {
    config: LimitConfig,
    kill_switch_armed: bool,
    kill_reason: Option<String>,
    open_orders: usize,
    gross_exposure: f64,
    per_symbol_exposure: HashMap<String, f64>,
    exceptions: Vec<String>,
    clock: Arc<dyn Clock>,
}

impl std::fmt::Debug for LimitRiskEngine {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("LimitRiskEngine")
            .field("open_orders", &self.open_orders)
            .field("gross_exposure", &self.gross_exposure)
            .field("kill_switch_armed", &self.kill_switch_armed)
            .finish_non_exhaustive()
    }
}

impl LimitRiskEngine {
    #[must_use]
    pub fn new(config: LimitConfig, clock: Arc<dyn Clock>) -> Self {
        Self {
            config,
            kill_switch_armed: false,
            kill_reason: None,
            open_orders: 0,
            gross_exposure: 0.0,
            per_symbol_exposure: HashMap::new(),
            exceptions: Vec::new(),
            clock,
        }
    }

    pub fn update_fill(&mut self, symbol: &SymbolV1, notional: f64) {
        let key = format!("{:?}:{}", symbol.venue, symbol.ticker());
        *self.per_symbol_exposure.entry(key).or_insert(0.0) += notional;
        self.gross_exposure = self
            .per_symbol_exposure
            .values()
            .map(|value| value.abs())
            .sum();
        self.open_orders = self.open_orders.saturating_sub(1);
    }
}

impl RiskPolicyEngine for LimitRiskEngine {
    fn pre_trade_check(
        &mut self,
        order: &OrderRequestV1,
    ) -> TradingResult<contracts::RiskDecisionV1> {
        if self.kill_switch_armed {
            return Ok(contracts::RiskDecisionV1 {
                approved: false,
                reason: format!(
                    "kill switch armed: {}",
                    self.kill_reason
                        .clone()
                        .unwrap_or_else(|| "unspecified".to_string())
                ),
            });
        }

        let notional = order.quantity * order.limit_price.unwrap_or(1.0);
        if notional > self.config.max_order_notional {
            let reason = format!("order notional {:.2} exceeds limit", notional);
            self.exceptions.push(reason.clone());
            return Ok(contracts::RiskDecisionV1 {
                approved: false,
                reason,
            });
        }
        if self.open_orders >= self.config.max_open_orders {
            let reason = "open order count exceeds limit".to_string();
            self.exceptions.push(reason.clone());
            return Ok(contracts::RiskDecisionV1 {
                approved: false,
                reason,
            });
        }
        if self.gross_exposure + notional > self.config.max_gross_exposure {
            let reason = "gross exposure would exceed limit".to_string();
            self.exceptions.push(reason.clone());
            return Ok(contracts::RiskDecisionV1 {
                approved: false,
                reason,
            });
        }

        self.open_orders += 1;
        Ok(contracts::RiskDecisionV1 {
            approved: true,
            reason: "approved".to_string(),
        })
    }

    fn observe_fill(&mut self, fill: &FillV1) -> TradingResult<()> {
        self.update_fill(&fill.symbol, fill.price * fill.quantity);
        Ok(())
    }

    fn intra_day_limits(&self) -> TradingResult<TradingRiskSnapshotV1> {
        Ok(TradingRiskSnapshotV1 {
            as_of: self.clock.now(),
            gross_exposure: self.gross_exposure,
            net_exposure: self.per_symbol_exposure.values().sum(),
            open_orders: self.open_orders,
            kill_switch_armed: self.kill_switch_armed,
        })
    }

    fn kill_switch(&mut self, reason: &str) -> TradingResult<()> {
        self.kill_switch_armed = true;
        self.kill_reason = Some(reason.to_string());
        self.exceptions
            .push(format!("{}: {reason}", self.clock.now().to_rfc3339()));
        Ok(())
    }

    fn post_trade_exceptions(&self) -> TradingResult<Vec<String>> {
        Ok(self.exceptions.clone())
    }
}
