use std::collections::HashMap;
use std::sync::Arc;

use contracts::{
    LimitBreachRecordV1, MarketEventV1, OrderRequestV1, OrderStatusV1, ServiceBoundaryV1, VenueV1,
};
use error_model::{InstitutionalError, InstitutionalResult, OperationContext};
use events::{
    CapitalMarketsEventPayloadV1, FillRecordedV1, OrderSubmittedV1, PortfolioSnapshottedV1,
    RiskLimitBreachedV1, SignalGeneratedV1,
};
use serde::{Deserialize, Serialize};
use trading_core::{
    Clock, ExecutionVenueAdapter, IdGenerator, PortfolioLedger, RiskPolicyEngine, StrategyModule,
    build_determinism_key, build_order_request,
};
use trading_errors::TradingError;

fn map_trading_error(error: TradingError) -> InstitutionalError {
    InstitutionalError::external(
        "trading-core",
        Some("execution".to_string()),
        error.to_string(),
    )
}

pub struct ExecutionRouter {
    adapters: HashMap<VenueV1, Box<dyn ExecutionVenueAdapter>>,
}

impl Default for ExecutionRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutionRouter {
    #[must_use]
    pub fn new() -> Self {
        Self {
            adapters: HashMap::new(),
        }
    }

    pub fn register(&mut self, venue: VenueV1, adapter: Box<dyn ExecutionVenueAdapter>) {
        self.adapters.insert(venue, adapter);
    }

    fn submit(&mut self, order: &OrderRequestV1) -> InstitutionalResult<contracts::OrderAckV1> {
        let Some(adapter) = self.adapters.get_mut(&order.venue) else {
            return Err(InstitutionalError::not_found(
                OperationContext::new("services/execution-service", "submit"),
                "missing adapter for venue",
            ));
        };
        adapter.submit_order(order).map_err(map_trading_error)
    }

    fn reconcile_all(&mut self) -> InstitutionalResult<Vec<contracts::FillV1>> {
        let mut fills = Vec::new();
        for adapter in self.adapters.values_mut() {
            fills.extend(adapter.reconcile_fills().map_err(map_trading_error)?);
        }
        Ok(fills)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EngineCycle {
    pub accepted_orders: usize,
    pub rejected_orders: usize,
    pub fills: usize,
}

pub struct EventDrivenStrategyEngine {
    strategy: Box<dyn StrategyModule>,
    risk: Box<dyn RiskPolicyEngine>,
    portfolio: Box<dyn PortfolioLedger>,
    router: ExecutionRouter,
    recorded_events: Vec<CapitalMarketsEventPayloadV1>,
    config_hash: String,
    model_version: String,
    ids: Arc<dyn IdGenerator>,
    clock: Arc<dyn Clock>,
}

impl std::fmt::Debug for EventDrivenStrategyEngine {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("EventDrivenStrategyEngine")
            .field("recorded_events", &self.recorded_events.len())
            .field("config_hash", &self.config_hash)
            .finish_non_exhaustive()
    }
}

impl EventDrivenStrategyEngine {
    #[must_use]
    pub fn new(
        strategy: Box<dyn StrategyModule>,
        risk: Box<dyn RiskPolicyEngine>,
        portfolio: Box<dyn PortfolioLedger>,
        router: ExecutionRouter,
        config_hash: impl Into<String>,
        model_version: impl Into<String>,
        ids: Arc<dyn IdGenerator>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            strategy,
            risk,
            portfolio,
            router,
            recorded_events: Vec::new(),
            config_hash: config_hash.into(),
            model_version: model_version.into(),
            ids,
            clock,
        }
    }

    pub fn on_market_event(&mut self, event: &MarketEventV1) -> InstitutionalResult<EngineCycle> {
        let determinism = build_determinism_key(
            self.ids.as_ref(),
            self.model_version.clone(),
            self.config_hash.clone(),
        );
        let signals = self
            .strategy
            .on_market_event(event, determinism)
            .map_err(map_trading_error)?;

        let mut accepted_orders = 0usize;
        let mut rejected_orders = 0usize;

        for signal in signals {
            self.recorded_events
                .push(CapitalMarketsEventPayloadV1::SignalGenerated(
                    SignalGeneratedV1 {
                        strategy_id: signal.strategy_id.clone(),
                        signal: signal.clone(),
                    },
                ));

            let Some(order) = build_order_request(&signal, self.ids.as_ref(), self.clock.as_ref())
            else {
                continue;
            };

            let risk_decision = self
                .risk
                .pre_trade_check(&order)
                .map_err(map_trading_error)?;
            if !risk_decision.approved {
                rejected_orders += 1;
                self.record_breach("pre_trade_check", risk_decision.reason);
                continue;
            }

            self.recorded_events
                .push(CapitalMarketsEventPayloadV1::OrderSubmitted(
                    OrderSubmittedV1 {
                        order: order.clone(),
                    },
                ));

            match self.router.submit(&order)? {
                ack if matches!(ack.status, OrderStatusV1::Rejected(_)) => {
                    rejected_orders += 1;
                    self.record_breach("execution_router", ack.message);
                }
                _ack => {
                    accepted_orders += 1;
                }
            }
        }

        let fills = self.router.reconcile_all()?;
        for fill in &fills {
            self.portfolio.apply_fill(fill).map_err(map_trading_error)?;
            self.risk.observe_fill(fill).map_err(map_trading_error)?;
            self.recorded_events
                .push(CapitalMarketsEventPayloadV1::FillRecorded(FillRecordedV1 {
                    fill: fill.clone(),
                }));
        }

        self.portfolio
            .mark_to_market(&[(event.symbol().clone(), event.price())])
            .map_err(map_trading_error)?;
        let snapshot = self.portfolio.snapshot().map_err(map_trading_error)?;
        self.recorded_events
            .push(CapitalMarketsEventPayloadV1::PortfolioSnapshotted(
                PortfolioSnapshottedV1 { snapshot },
            ));

        Ok(EngineCycle {
            accepted_orders,
            rejected_orders,
            fills: fills.len(),
        })
    }

    #[must_use]
    pub fn recorded_events(&self) -> &[CapitalMarketsEventPayloadV1] {
        &self.recorded_events
    }

    fn record_breach(&mut self, control: &str, details: String) {
        self.recorded_events
            .push(CapitalMarketsEventPayloadV1::RiskLimitBreached(
                RiskLimitBreachedV1 {
                    breach: LimitBreachRecordV1 {
                        breach_id: self.ids.next_id(),
                        detected_at: self.clock.now(),
                        control: control.to_string(),
                        severity: "medium".to_string(),
                        details,
                    },
                },
            ));
    }
}

#[must_use]
pub fn service_boundary() -> ServiceBoundaryV1 {
    ServiceBoundaryV1 {
        service_name: "execution-service".into(),
        domain: "capital_markets_execution".to_owned(),
        approved_workflows: vec!["quant_strategy_promotion".into()],
        owned_aggregates: vec![
            "order_record".into(),
            "execution_session".into(),
            "fill_record".into(),
        ],
    }
}
