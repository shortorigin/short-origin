use std::collections::HashMap;
use std::sync::Arc;

use contracts::{
    FillV1, LedgerEntryV1, PortfolioSnapshotV1, ServiceBoundaryV1, SignalV1, SymbolV1,
};
use trading_core::{
    AllocatedSignal, Clock, DoubleEntryLedger, MultiStrategyAllocator, PortfolioLedger,
    StrategyAllocationRule, reconcile_snapshot,
};
use trading_errors::TradingResult;

#[derive(Debug, Clone)]
pub struct PortfolioService {
    ledger: DoubleEntryLedger,
    allocator: MultiStrategyAllocator,
}

impl PortfolioService {
    #[must_use]
    pub fn new(
        initial_cash: f64,
        clock: Arc<dyn Clock>,
        rules: Vec<StrategyAllocationRule>,
    ) -> Self {
        Self {
            ledger: DoubleEntryLedger::new(initial_cash, clock),
            allocator: MultiStrategyAllocator::new(initial_cash, rules),
        }
    }

    #[must_use]
    pub fn allocate(
        &self,
        signals: &[SignalV1],
        marks: &HashMap<String, f64>,
    ) -> Vec<AllocatedSignal> {
        self.allocator.allocate(signals, marks)
    }

    pub fn reconcile(&self) -> TradingResult<()> {
        reconcile_snapshot(&self.ledger.snapshot()?)
    }
}

impl PortfolioLedger for PortfolioService {
    fn post_entry(&mut self, entry: LedgerEntryV1) -> TradingResult<()> {
        self.ledger.post_entry(entry)
    }

    fn apply_fill(&mut self, fill: &FillV1) -> TradingResult<()> {
        self.ledger.apply_fill(fill)
    }

    fn mark_to_market(&mut self, marks: &[(SymbolV1, f64)]) -> TradingResult<()> {
        self.ledger.mark_to_market(marks)
    }

    fn snapshot(&self) -> TradingResult<PortfolioSnapshotV1> {
        self.ledger.snapshot()
    }

    fn entries(&self) -> &[LedgerEntryV1] {
        self.ledger.entries()
    }
}

#[must_use]
pub fn service_boundary() -> ServiceBoundaryV1 {
    ServiceBoundaryV1 {
        service_name: "portfolio-service".to_owned(),
        domain: "capital_markets_portfolio".to_owned(),
        approved_workflows: vec!["quant_strategy_promotion".to_owned()],
        owned_aggregates: vec![
            "portfolio_snapshot".to_owned(),
            "allocation_policy".to_owned(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use chrono::TimeZone;
    use contracts::{AssetClassV1, FillV1, Side, SignalSideV1, SignalV1, SymbolV1, VenueV1};
    use trading_core::{Clock, FixedClock, PortfolioLedger, StrategyAllocationRule};

    use super::PortfolioService;

    #[test]
    fn ledger_reconciles_after_fills() {
        let clock = Arc::new(FixedClock::new(
            chrono::Utc
                .with_ymd_and_hms(2026, 3, 1, 0, 0, 0)
                .single()
                .expect("clock"),
        ));
        let symbol = SymbolV1::new(VenueV1::Coinbase, AssetClassV1::Crypto, "BTC", "USD");
        let mut service = PortfolioService::new(100_000.0, clock.clone(), Vec::new());
        service
            .apply_fill(&FillV1 {
                fill_id: "fill-1".to_string(),
                order_id: "order-1".to_string(),
                symbol: symbol.clone(),
                side: Side::Buy,
                quantity: 1.0,
                price: 100.0,
                fee: 0.1,
                timestamp: clock.now(),
            })
            .expect("buy");
        service
            .apply_fill(&FillV1 {
                fill_id: "fill-2".to_string(),
                order_id: "order-2".to_string(),
                symbol,
                side: Side::Sell,
                quantity: 0.5,
                price: 110.0,
                fee: 0.1,
                timestamp: clock.now(),
            })
            .expect("sell");

        let snapshot = service.snapshot().expect("snapshot");
        assert!(snapshot.cash > 99_800.0);
        service.reconcile().expect("reconcile");
    }

    #[test]
    fn allocator_caps_strategy_notional() {
        let clock = Arc::new(FixedClock::new(
            chrono::Utc
                .with_ymd_and_hms(2026, 3, 1, 0, 0, 0)
                .single()
                .expect("clock"),
        ));
        let service = PortfolioService::new(
            1000.0,
            clock,
            vec![StrategyAllocationRule {
                strategy_id: "trend".to_string(),
                max_notional: 100.0,
            }],
        );
        let symbol = SymbolV1::new(VenueV1::Coinbase, AssetClassV1::Crypto, "BTC", "USD");
        let signals = vec![SignalV1 {
            strategy_id: "trend".to_string(),
            symbol: symbol.clone(),
            side: SignalSideV1::Buy,
            quantity: 2.0,
            confidence: 0.7,
            reason: "test".to_string(),
            determinism: contracts::DeterminismKeyV1::new("event-1", "v1", "cfg"),
        }];
        let allocated = service.allocate(&signals, &HashMap::from([(symbol.ticker(), 100.0)]));
        assert!((allocated[0].approved_qty - 1.0).abs() < f64::EPSILON);
    }
}
