use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use contracts::{
    AssetClassV1, HealthStatusV1, HistoricalDataRequestV1, MarketDataBatchV1, MarketEventV1,
    OhlcvBarV1, RawMarketRecordV1, ServiceBoundaryV1, SymbolV1, TradeTickV1, VenueV1,
};
use error_model::{InstitutionalError, InstitutionalResult};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use trading_core::{Clock, IdGenerator, MarketDataAdapter, SystemClock};
use trading_errors::{TradingError, TradingResult};

const SERVICE_NAME: &str = "market-data-service";
const DOMAIN_NAME: &str = "capital_markets_data";
const APPROVED_WORKFLOWS: &[&str] = &["quant_strategy_promotion"];
const OWNED_AGGREGATES: &[&str] = &["market_dataset", "normalization_report"];

fn map_trading_error(error: TradingError) -> InstitutionalError {
    InstitutionalError::external(
        "trading-core",
        Some("market-data".to_string()),
        error.to_string(),
    )
}

#[derive(Debug, Default, Clone)]
struct InMemoryMarketDataCatalog {
    batches: Vec<MarketDataBatchV1>,
}

impl InMemoryMarketDataCatalog {
    fn record(&mut self, batch: MarketDataBatchV1) {
        self.batches.push(batch);
    }

    fn batches(&self) -> &[MarketDataBatchV1] {
        &self.batches
    }
}

#[derive(Debug, Default, Clone)]
pub struct MarketDataService {
    catalog: InMemoryMarketDataCatalog,
}

impl MarketDataService {
    pub fn ingest_historical(
        &mut self,
        ids: &dyn IdGenerator,
        adapter: &dyn MarketDataAdapter,
        dataset_name: impl Into<String>,
        req: HistoricalDataRequestV1,
    ) -> InstitutionalResult<MarketDataBatchV1> {
        let venue = req.symbol.venue;
        let events = adapter.fetch_historical(req).map_err(map_trading_error)?;
        self.register_batch(ids.next_id(), dataset_name.into(), venue, events)
    }

    pub fn normalize_batch(
        &mut self,
        ids: &dyn IdGenerator,
        adapter: &dyn MarketDataAdapter,
        dataset_name: impl Into<String>,
        venue: VenueV1,
        raw: Vec<RawMarketRecordV1>,
    ) -> InstitutionalResult<MarketDataBatchV1> {
        let events = adapter
            .normalize_to_domain(raw)
            .map_err(map_trading_error)?;
        self.register_batch(ids.next_id(), dataset_name.into(), venue, events)
    }

    pub fn register_batch(
        &mut self,
        dataset_id: impl Into<String>,
        dataset_name: impl Into<String>,
        venue: VenueV1,
        events: Vec<MarketEventV1>,
    ) -> InstitutionalResult<MarketDataBatchV1> {
        let Some(start_time) = events.first().map(MarketEventV1::event_time) else {
            return Err(InstitutionalError::InvariantViolation {
                invariant: "market data batch requires at least one event".to_string(),
            });
        };
        let end_time = events.last().map_or(start_time, MarketEventV1::event_time);

        let batch = MarketDataBatchV1 {
            dataset_id: dataset_id.into(),
            dataset_name: dataset_name.into(),
            venue,
            event_count: events.len(),
            start_time,
            end_time,
            events,
        };
        self.catalog.record(batch.clone());
        Ok(batch)
    }

    #[must_use]
    pub fn batches(&self) -> &[MarketDataBatchV1] {
        self.catalog.batches()
    }
}

pub struct CoinbaseAdapter {
    seed: u64,
    clock: Arc<dyn Clock>,
}

impl CoinbaseAdapter {
    #[must_use]
    pub fn new(seed: u64, clock: Arc<dyn Clock>) -> Self {
        Self { seed, clock }
    }
}

pub struct OandaAdapter {
    seed: u64,
    clock: Arc<dyn Clock>,
}

impl OandaAdapter {
    #[must_use]
    pub fn new(seed: u64, clock: Arc<dyn Clock>) -> Self {
        Self { seed, clock }
    }
}

impl MarketDataAdapter for CoinbaseAdapter {
    fn fetch_historical(&self, req: HistoricalDataRequestV1) -> TradingResult<Vec<MarketEventV1>> {
        if req.symbol.venue != VenueV1::Coinbase {
            return Err(TradingError::InvalidInput {
                details: "Coinbase adapter received non-Coinbase symbol".to_string(),
            });
        }
        generate_bars(self.seed, req)
    }

    fn stream_realtime(&self, symbol: SymbolV1) -> TradingResult<Vec<MarketEventV1>> {
        if symbol.venue != VenueV1::Coinbase {
            return Err(TradingError::InvalidInput {
                details: "Coinbase stream requested for non-Coinbase symbol".to_string(),
            });
        }
        Ok(vec![MarketEventV1::Trade(TradeTickV1 {
            symbol,
            trade_time: self.clock.now(),
            price: 50_000.0,
            size: 0.25,
            trade_id: "cb-sim-1".to_string(),
        })])
    }

    fn health_check(&self) -> HealthStatusV1 {
        HealthStatusV1::Healthy
    }

    fn normalize_to_domain(
        &self,
        raw: Vec<RawMarketRecordV1>,
    ) -> TradingResult<Vec<MarketEventV1>> {
        normalize_records(raw, VenueV1::Coinbase, AssetClassV1::Crypto)
    }
}

impl MarketDataAdapter for OandaAdapter {
    fn fetch_historical(&self, req: HistoricalDataRequestV1) -> TradingResult<Vec<MarketEventV1>> {
        if req.symbol.venue != VenueV1::Oanda {
            return Err(TradingError::InvalidInput {
                details: "Oanda adapter received non-Oanda symbol".to_string(),
            });
        }
        generate_bars(self.seed + 1_000, req)
    }

    fn stream_realtime(&self, symbol: SymbolV1) -> TradingResult<Vec<MarketEventV1>> {
        if symbol.venue != VenueV1::Oanda {
            return Err(TradingError::InvalidInput {
                details: "Oanda stream requested for non-Oanda symbol".to_string(),
            });
        }
        Ok(vec![MarketEventV1::Trade(TradeTickV1 {
            symbol,
            trade_time: self.clock.now(),
            price: 1.075,
            size: 10_000.0,
            trade_id: "oanda-sim-1".to_string(),
        })])
    }

    fn health_check(&self) -> HealthStatusV1 {
        HealthStatusV1::Healthy
    }

    fn normalize_to_domain(
        &self,
        raw: Vec<RawMarketRecordV1>,
    ) -> TradingResult<Vec<MarketEventV1>> {
        normalize_records(raw, VenueV1::Oanda, AssetClassV1::Forex)
    }
}

fn normalize_records(
    mut raw: Vec<RawMarketRecordV1>,
    venue: VenueV1,
    asset_class: AssetClassV1,
) -> TradingResult<Vec<MarketEventV1>> {
    raw.sort_by_key(|record| record.event_time);

    let mut out = Vec::with_capacity(raw.len());
    for record in raw {
        let (base, quote) = split_symbol(&record.source_symbol)?;
        let symbol = SymbolV1::new(venue, asset_class, base, quote);

        if let (Some(open), Some(high), Some(low)) = (record.open, record.high, record.low) {
            out.push(MarketEventV1::Bar(OhlcvBarV1 {
                symbol,
                open_time: record.event_time,
                close_time: record.event_time,
                open,
                high,
                low,
                close: record.close,
                volume: record.volume,
            }));
        } else {
            out.push(MarketEventV1::Trade(TradeTickV1 {
                symbol,
                trade_time: record.event_time,
                price: record.close,
                size: record.volume,
                trade_id: record.trade_id.unwrap_or_else(|| {
                    format!(
                        "synthetic-{}",
                        record.event_time.timestamp_nanos_opt().unwrap_or_default()
                    )
                }),
            }));
        }
    }

    Ok(out)
}

fn generate_bars(seed: u64, req: HistoricalDataRequestV1) -> TradingResult<Vec<MarketEventV1>> {
    if req.interval_seconds <= 0 {
        return Err(TradingError::InvalidInput {
            details: "interval_seconds must be positive".to_string(),
        });
    }
    if req.end <= req.start {
        return Err(TradingError::InvalidInput {
            details: "end must be after start".to_string(),
        });
    }

    let mut rng = StdRng::seed_from_u64(seed);
    let mut events = Vec::new();
    let mut cursor: DateTime<Utc> = req.start;
    let mut price = 100.0 + rng.random_range(0.0f64..5.0f64);
    let interval = Duration::seconds(req.interval_seconds);

    while cursor < req.end {
        let next = cursor + interval;
        let drift = rng.random_range(-0.005f64..0.005f64);
        let open = price;
        let close = (open * (1.0 + drift)).max(0.0001);
        let high = open.max(close) * (1.0 + rng.random_range(0.0f64..0.0015f64));
        let low = open.min(close) * (1.0 - rng.random_range(0.0f64..0.0015f64));
        let volume = rng.random_range(50.0f64..10_000.0f64);
        events.push(MarketEventV1::Bar(OhlcvBarV1 {
            symbol: req.symbol.clone(),
            open_time: cursor,
            close_time: next,
            open,
            high,
            low,
            close,
            volume,
        }));
        price = close;
        cursor = next;
    }

    Ok(events)
}

fn split_symbol(value: &str) -> TradingResult<(&str, &str)> {
    for separator in ['-', '/', '_'] {
        if let Some((base, quote)) = value.split_once(separator) {
            return Ok((base, quote));
        }
    }
    if value.len() >= 6 {
        let split = value.len() / 2;
        return Ok((&value[..split], &value[split..]));
    }
    Err(TradingError::Parse {
        source_name: "source_symbol".to_string(),
        details: format!("unable to split symbol {value}"),
    })
}

#[must_use]
pub fn service_boundary() -> ServiceBoundaryV1 {
    ServiceBoundaryV1 {
        service_name: SERVICE_NAME.to_owned(),
        domain: DOMAIN_NAME.to_owned(),
        approved_workflows: APPROVED_WORKFLOWS
            .iter()
            .map(|value| (*value).to_owned())
            .collect(),
        owned_aggregates: OWNED_AGGREGATES
            .iter()
            .map(|value| (*value).to_owned())
            .collect(),
    }
}

impl Default for CoinbaseAdapter {
    fn default() -> Self {
        Self::new(42, Arc::new(SystemClock))
    }
}

impl Default for OandaAdapter {
    fn default() -> Self {
        Self::new(84, Arc::new(SystemClock))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chrono::{Duration, TimeZone, Utc};
    use contracts::{AssetClassV1, HistoricalDataRequestV1, RawMarketRecordV1, SymbolV1, VenueV1};
    use trading_core::{FixedClock, MarketDataAdapter, SequenceIdGenerator};

    use super::{
        service_boundary, CoinbaseAdapter, MarketDataService, OandaAdapter, APPROVED_WORKFLOWS,
        DOMAIN_NAME, OWNED_AGGREGATES, SERVICE_NAME,
    };

    #[test]
    fn service_boundary_matches_enterprise_catalog() {
        let source = include_str!(
            "../../../enterprise/domains/capital_markets_data/service_boundaries.toml"
        );
        let boundary = service_boundary();

        assert_eq!(boundary.service_name, SERVICE_NAME);
        assert_eq!(boundary.domain, DOMAIN_NAME);
        for workflow in APPROVED_WORKFLOWS {
            assert!(source.contains(workflow));
        }
        for aggregate in OWNED_AGGREGATES {
            assert!(source.contains(aggregate));
        }
    }

    #[test]
    fn coinbase_historical_is_sorted() {
        let clock = Arc::new(FixedClock::new(
            Utc.with_ymd_and_hms(2026, 3, 1, 0, 0, 0)
                .single()
                .expect("clock"),
        ));
        let adapter = CoinbaseAdapter::new(42, clock);
        let req = HistoricalDataRequestV1 {
            symbol: SymbolV1::new(VenueV1::Coinbase, AssetClassV1::Crypto, "BTC", "USD"),
            start: Utc
                .with_ymd_and_hms(2026, 3, 1, 0, 0, 0)
                .single()
                .expect("start"),
            end: Utc
                .with_ymd_and_hms(2026, 3, 1, 1, 0, 0)
                .single()
                .expect("end"),
            interval_seconds: 60,
        };

        let events = adapter.fetch_historical(req).expect("fetch");
        assert!(!events.is_empty());
        let mut last = events[0].event_time();
        for event in &events[1..] {
            assert!(event.event_time() >= last);
            last = event.event_time();
        }
    }

    #[test]
    fn oanda_normalization_orders_by_time() {
        let clock = Arc::new(FixedClock::new(
            Utc.with_ymd_and_hms(2026, 3, 1, 0, 0, 0)
                .single()
                .expect("clock"),
        ));
        let adapter = OandaAdapter::new(99, clock);
        let now = Utc
            .with_ymd_and_hms(2026, 3, 1, 0, 0, 0)
            .single()
            .expect("now");
        let raw = vec![
            RawMarketRecordV1 {
                source_symbol: "EUR_USD".to_string(),
                event_time: now + Duration::seconds(5),
                open: Some(1.1),
                high: Some(1.2),
                low: Some(1.0),
                close: 1.15,
                volume: 10.0,
                trade_id: None,
            },
            RawMarketRecordV1 {
                source_symbol: "EUR_USD".to_string(),
                event_time: now,
                open: Some(1.0),
                high: Some(1.1),
                low: Some(0.9),
                close: 1.05,
                volume: 12.0,
                trade_id: None,
            },
        ];

        let normalized = adapter.normalize_to_domain(raw).expect("normalize");
        assert_eq!(normalized.len(), 2);
        assert!(normalized[0].event_time() <= normalized[1].event_time());
    }

    #[test]
    fn service_registers_batches() {
        let clock = Arc::new(FixedClock::new(
            Utc.with_ymd_and_hms(2026, 3, 1, 0, 0, 0)
                .single()
                .expect("clock"),
        ));
        let adapter = CoinbaseAdapter::new(42, clock);
        let mut service = MarketDataService::default();
        let ids = SequenceIdGenerator::new("dataset");
        let batch = service
            .ingest_historical(
                &ids,
                &adapter,
                "coinbase_btcusd_1m",
                HistoricalDataRequestV1 {
                    symbol: SymbolV1::new(VenueV1::Coinbase, AssetClassV1::Crypto, "BTC", "USD"),
                    start: Utc
                        .with_ymd_and_hms(2026, 3, 1, 0, 0, 0)
                        .single()
                        .expect("start"),
                    end: Utc
                        .with_ymd_and_hms(2026, 3, 1, 1, 0, 0)
                        .single()
                        .expect("end"),
                    interval_seconds: 60,
                },
            )
            .expect("batch");
        assert_eq!(batch.dataset_name, "coinbase_btcusd_1m");
        assert_eq!(service.batches().len(), 1);
    }
}
