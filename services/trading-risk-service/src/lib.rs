use std::sync::Arc;

use contracts::{FillV1, OrderRequestV1, ServiceBoundaryV1, SymbolV1, TradingRiskSnapshotV1};
use trading_core::{Clock, LimitConfig, LimitRiskEngine, RiskPolicyEngine};
use trading_errors::TradingResult;

#[derive(Debug, Clone)]
pub struct TradingRiskService {
    engine: LimitRiskEngine,
}

impl TradingRiskService {
    #[must_use]
    pub fn new(config: LimitConfig, clock: Arc<dyn Clock>) -> Self {
        Self {
            engine: LimitRiskEngine::new(config, clock),
        }
    }

    pub fn update_fill(&mut self, symbol: &SymbolV1, notional: f64) {
        self.engine.update_fill(symbol, notional);
    }

    pub fn snapshot(&self) -> TradingResult<TradingRiskSnapshotV1> {
        self.engine.intra_day_limits()
    }
}

impl RiskPolicyEngine for TradingRiskService {
    fn pre_trade_check(
        &mut self,
        order: &OrderRequestV1,
    ) -> TradingResult<contracts::RiskDecisionV1> {
        self.engine.pre_trade_check(order)
    }

    fn observe_fill(&mut self, fill: &FillV1) -> TradingResult<()> {
        self.engine.observe_fill(fill)
    }

    fn intra_day_limits(&self) -> TradingResult<TradingRiskSnapshotV1> {
        self.engine.intra_day_limits()
    }

    fn kill_switch(&mut self, reason: &str) -> TradingResult<()> {
        self.engine.kill_switch(reason)
    }

    fn post_trade_exceptions(&self) -> TradingResult<Vec<String>> {
        self.engine.post_trade_exceptions()
    }
}

#[must_use]
pub fn service_boundary() -> ServiceBoundaryV1 {
    ServiceBoundaryV1 {
        service_name: "trading-risk-service".to_owned(),
        domain: "capital_markets_risk".to_owned(),
        approved_workflows: vec!["quant_strategy_promotion".to_owned()],
        owned_aggregates: vec![
            "trading_risk_state".to_owned(),
            "kill_switch".to_owned(),
            "risk_limit_breach".to_owned(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chrono::TimeZone;
    use contracts::{
        AssetClassV1, OrderRequestV1, OrderTypeV1, Side, SymbolV1, TimeInForceV1, VenueV1,
    };
    use trading_core::{Clock, FixedClock, LimitConfig, RiskPolicyEngine};

    use super::TradingRiskService;

    #[test]
    fn kill_switch_blocks_orders() {
        let clock: Arc<dyn Clock> = Arc::new(FixedClock::new(
            chrono::Utc
                .with_ymd_and_hms(2026, 3, 1, 0, 0, 0)
                .single()
                .expect("clock"),
        ));
        let mut service = TradingRiskService::new(
            LimitConfig {
                max_order_notional: 1_000_000.0,
                max_gross_exposure: 5_000_000.0,
                max_open_orders: 10,
            },
            Arc::clone(&clock),
        );
        service.kill_switch("manual stop").expect("kill");

        let order = OrderRequestV1 {
            order_id: "order-1".to_string(),
            strategy_id: "s1".to_string(),
            symbol: SymbolV1::new(VenueV1::Coinbase, AssetClassV1::Crypto, "BTC", "USD"),
            venue: VenueV1::Coinbase,
            side: Side::Buy,
            quantity: 1.0,
            limit_price: Some(100.0),
            order_type: OrderTypeV1::Limit,
            tif: TimeInForceV1::Gtc,
            submitted_at: clock.now(),
        };

        let decision = service.pre_trade_check(&order).expect("check");
        assert!(!decision.approved);
    }
}
