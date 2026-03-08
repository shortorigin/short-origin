use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use chrono::{DateTime, Utc};
use contracts::{MarketEventV1, SignalV1, StrategyConfigV1, StrategyStateSnapshotV1};
use serde::{Deserialize, Serialize};
#[cfg(any(test, feature = "in-memory"))]
use trading_core::StrategyModule;
use trading_core::{Clock, SystemClock};
use trading_errors::{TradingError, TradingResult};
use wasmtime::{Engine, Instance, Memory, Module, Store, TypedFunc};

const WASM_PAGE_BYTES: usize = 65_536;
const INPUT_OFFSET_BYTES: usize = 8 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WasmRuntimePolicy {
    pub max_memory_bytes: usize,
    pub max_execution_ms: u64,
}

impl Default for WasmRuntimePolicy {
    fn default() -> Self {
        Self {
            max_memory_bytes: 1_048_576,
            max_execution_ms: 250,
        }
    }
}

enum StrategyExecutor {
    Wasmtime(Box<WasmtimeGuestStrategy>),
    #[cfg(any(test, feature = "in-memory"))]
    InMemory(Box<dyn StrategyModule>),
}

struct SandboxedStrategy {
    executor: StrategyExecutor,
    loaded_at: DateTime<Utc>,
    calls: u64,
}

pub struct StrategySandbox {
    policy: WasmRuntimePolicy,
    modules: HashMap<String, SandboxedStrategy>,
    clock: Arc<dyn Clock>,
}

impl std::fmt::Debug for StrategySandbox {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("StrategySandbox")
            .field("modules", &self.modules.len())
            .field("policy", &self.policy)
            .finish_non_exhaustive()
    }
}

impl StrategySandbox {
    #[must_use]
    pub fn new(policy: WasmRuntimePolicy, clock: Arc<dyn Clock>) -> Self {
        Self {
            policy,
            modules: HashMap::new(),
            clock,
        }
    }

    #[must_use]
    pub fn with_system_clock(policy: WasmRuntimePolicy) -> Self {
        Self::new(policy, Arc::new(SystemClock))
    }

    pub fn load_component_bytes(
        &mut self,
        module_id: impl Into<String>,
        wasm_bytes: &[u8],
        config: StrategyConfigV1,
    ) -> TradingResult<()> {
        let module_id = module_id.into();
        if self.modules.contains_key(&module_id) {
            return Err(TradingError::Conflict {
                details: "module already loaded".to_string(),
            });
        }

        let mut module = WasmtimeGuestStrategy::new(wasm_bytes)?;
        module.init(config)?;
        Self::enforce_memory_limit(&self.policy, &module.snapshot_state()?)?;
        self.modules.insert(
            module_id,
            SandboxedStrategy {
                executor: StrategyExecutor::Wasmtime(Box::new(module)),
                loaded_at: self.clock.now(),
                calls: 0,
            },
        );
        Ok(())
    }

    pub fn load_wat(
        &mut self,
        module_id: impl Into<String>,
        wat_source: &str,
        config: StrategyConfigV1,
    ) -> TradingResult<()> {
        let wasm = wat::parse_str(wat_source).map_err(|error| TradingError::Parse {
            source_name: "wat".to_string(),
            details: error.to_string(),
        })?;
        self.load_component_bytes(module_id, &wasm, config)
    }

    #[cfg(any(test, feature = "in-memory"))]
    pub fn load_in_memory(
        &mut self,
        module_id: impl Into<String>,
        mut module: Box<dyn StrategyModule>,
        config: StrategyConfigV1,
    ) -> TradingResult<()> {
        let module_id = module_id.into();
        if self.modules.contains_key(&module_id) {
            return Err(TradingError::Conflict {
                details: "module already loaded".to_string(),
            });
        }

        module.init(config)?;
        Self::enforce_memory_limit(&self.policy, &module.snapshot_state(self.clock.now()))?;
        self.modules.insert(
            module_id,
            SandboxedStrategy {
                executor: StrategyExecutor::InMemory(module),
                loaded_at: self.clock.now(),
                calls: 0,
            },
        );
        Ok(())
    }

    pub fn unload(&mut self, module_id: &str) -> TradingResult<()> {
        self.modules
            .remove(module_id)
            .ok_or_else(|| TradingError::NotFound {
                resource: "module not loaded".to_string(),
            })?;
        Ok(())
    }

    pub fn on_market_event(
        &mut self,
        module_id: &str,
        event: &MarketEventV1,
        key: contracts::DeterminismKeyV1,
    ) -> TradingResult<Vec<SignalV1>> {
        #[cfg(any(test, feature = "in-memory"))]
        let clock = Arc::clone(&self.clock);
        let policy = self.policy.clone();
        let entry = self
            .modules
            .get_mut(module_id)
            .ok_or_else(|| TradingError::NotFound {
                resource: "module not found".to_string(),
            })?;

        let started = Instant::now();
        let signals = match &mut entry.executor {
            StrategyExecutor::Wasmtime(module) => module.on_market_event(event, key)?,
            #[cfg(any(test, feature = "in-memory"))]
            StrategyExecutor::InMemory(module) => module.on_market_event(event, key)?,
        };
        let elapsed_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
        Self::enforce_runtime(&policy, elapsed_ms)?;
        entry.calls += 1;
        let snapshot = match &mut entry.executor {
            StrategyExecutor::Wasmtime(module) => module.snapshot_state()?,
            #[cfg(any(test, feature = "in-memory"))]
            StrategyExecutor::InMemory(module) => module.snapshot_state(clock.now()),
        };
        Self::enforce_memory_limit(&policy, &snapshot)?;
        Ok(signals)
    }

    pub fn on_timer(
        &mut self,
        module_id: &str,
        now: DateTime<Utc>,
        key: contracts::DeterminismKeyV1,
    ) -> TradingResult<Vec<SignalV1>> {
        #[cfg(any(test, feature = "in-memory"))]
        let clock = Arc::clone(&self.clock);
        let policy = self.policy.clone();
        let entry = self
            .modules
            .get_mut(module_id)
            .ok_or_else(|| TradingError::NotFound {
                resource: "module not found".to_string(),
            })?;

        let started = Instant::now();
        let signals = match &mut entry.executor {
            StrategyExecutor::Wasmtime(module) => module.on_timer(now, key)?,
            #[cfg(any(test, feature = "in-memory"))]
            StrategyExecutor::InMemory(module) => module.on_timer(now, key)?,
        };
        let elapsed_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
        Self::enforce_runtime(&policy, elapsed_ms)?;
        entry.calls += 1;
        let snapshot = match &mut entry.executor {
            StrategyExecutor::Wasmtime(module) => module.snapshot_state()?,
            #[cfg(any(test, feature = "in-memory"))]
            StrategyExecutor::InMemory(module) => module.snapshot_state(clock.now()),
        };
        Self::enforce_memory_limit(&policy, &snapshot)?;
        Ok(signals)
    }

    pub fn snapshot(&mut self, module_id: &str) -> TradingResult<StrategyStateSnapshotV1> {
        let entry = self
            .modules
            .get_mut(module_id)
            .ok_or_else(|| TradingError::NotFound {
                resource: "module not found".to_string(),
            })?;

        match &mut entry.executor {
            StrategyExecutor::Wasmtime(module) => module.snapshot_state(),
            #[cfg(any(test, feature = "in-memory"))]
            StrategyExecutor::InMemory(module) => Ok(module.snapshot_state(self.clock.now())),
        }
    }

    pub fn module_metadata(&self, module_id: &str) -> TradingResult<(DateTime<Utc>, u64)> {
        let entry = self
            .modules
            .get(module_id)
            .ok_or_else(|| TradingError::NotFound {
                resource: "module not found".to_string(),
            })?;
        Ok((entry.loaded_at, entry.calls))
    }

    fn enforce_memory_limit(
        policy: &WasmRuntimePolicy,
        snapshot: &StrategyStateSnapshotV1,
    ) -> TradingResult<()> {
        let bytes = serde_json::to_vec(snapshot)?.len();
        if bytes > policy.max_memory_bytes {
            return Err(TradingError::RuntimePolicyViolation {
                details: "strategy state exceeds memory policy".to_string(),
            });
        }
        Ok(())
    }

    fn enforce_runtime(policy: &WasmRuntimePolicy, elapsed_ms: u64) -> TradingResult<()> {
        if elapsed_ms > policy.max_execution_ms {
            return Err(TradingError::RuntimePolicyViolation {
                details: "execution time exceeded runtime policy".to_string(),
            });
        }
        Ok(())
    }
}

pub fn demo_strategy_wat(strategy_id: &str) -> String {
    let signals = "[]";
    let snapshot = serde_json::json!({
        "strategy_id": strategy_id,
        "timestamp": "2026-03-05T00:00:00Z",
        "state": {
            "runtime": "wasmtime",
            "strategy_id": strategy_id,
        }
    })
    .to_string();
    let signals_len = signals.len();
    let snapshot_len = snapshot.len();
    let signals_ptr = 64usize;
    let snapshot_ptr = 256usize;
    let signals_handle = pack_output_handle(signals_ptr, signals_len);
    let snapshot_handle = pack_output_handle(snapshot_ptr, snapshot_len);

    format!(
        r#"(module
  (memory (export "memory") 1)
  (data (i32.const {signals_ptr}) "{signals_data}")
  (data (i32.const {snapshot_ptr}) "{snapshot_data}")
  (func (export "init") (param i32 i32) (result i32)
    i32.const 0)
  (func (export "on-market-event") (param i32 i32) (result i64)
    i64.const {signals_handle})
  (func (export "on-timer") (param i32 i32) (result i64)
    i64.const {signals_handle})
  (func (export "snapshot-state") (result i64)
    i64.const {snapshot_handle})
)"#,
        signals_ptr = signals_ptr,
        snapshot_ptr = snapshot_ptr,
        signals_data = wat_string_literal(signals.as_bytes()),
        snapshot_data = wat_string_literal(snapshot.as_bytes()),
        signals_handle = signals_handle,
        snapshot_handle = snapshot_handle,
    )
}

struct WasmtimeGuestStrategy {
    store: Store<()>,
    memory: Memory,
    init: TypedFunc<(i32, i32), i32>,
    on_market_event: TypedFunc<(i32, i32), i64>,
    on_timer: TypedFunc<(i32, i32), i64>,
    snapshot_state: TypedFunc<(), i64>,
}

impl WasmtimeGuestStrategy {
    fn new(wasm_bytes: &[u8]) -> TradingResult<Self> {
        let engine = Engine::default();
        let module = Module::new(&engine, wasm_bytes).map_err(wasmtime_error)?;
        let mut store = Store::new(&engine, ());
        let instance = Instance::new(&mut store, &module, &[]).map_err(wasmtime_error)?;
        let memory =
            instance
                .get_memory(&mut store, "memory")
                .ok_or_else(|| TradingError::Parse {
                    source_name: "wasmtime".to_string(),
                    details: "strategy module did not export `memory`".to_string(),
                })?;
        let init = instance
            .get_typed_func::<(i32, i32), i32>(&mut store, "init")
            .map_err(wasmtime_error)?;
        let on_market_event = instance
            .get_typed_func::<(i32, i32), i64>(&mut store, "on-market-event")
            .map_err(wasmtime_error)?;
        let on_timer = instance
            .get_typed_func::<(i32, i32), i64>(&mut store, "on-timer")
            .map_err(wasmtime_error)?;
        let snapshot_state = instance
            .get_typed_func::<(), i64>(&mut store, "snapshot-state")
            .map_err(wasmtime_error)?;

        Ok(Self {
            store,
            memory,
            init,
            on_market_event,
            on_timer,
            snapshot_state,
        })
    }

    fn init(&mut self, config: StrategyConfigV1) -> TradingResult<()> {
        let payload = serde_json::to_vec(&config)?;
        let (ptr, len) = self.write_input(&payload)?;
        let status = self
            .init
            .call(&mut self.store, (ptr, len))
            .map_err(wasmtime_error)?;
        if status == 0 {
            Ok(())
        } else {
            Err(TradingError::RuntimePolicyViolation {
                details: format!("guest init failed with status {status}"),
            })
        }
    }

    fn on_market_event(
        &mut self,
        event: &MarketEventV1,
        key: contracts::DeterminismKeyV1,
    ) -> TradingResult<Vec<SignalV1>> {
        #[derive(Serialize)]
        struct Payload<'a> {
            event: &'a MarketEventV1,
            determinism: contracts::DeterminismKeyV1,
        }

        let payload = serde_json::to_vec(&Payload {
            event,
            determinism: key,
        })?;
        let (ptr, len) = self.write_input(&payload)?;
        let handle = self
            .on_market_event
            .call(&mut self.store, (ptr, len))
            .map_err(wasmtime_error)?;
        self.read_json_output(handle, "on-market-event")
    }

    fn on_timer(
        &mut self,
        now: DateTime<Utc>,
        key: contracts::DeterminismKeyV1,
    ) -> TradingResult<Vec<SignalV1>> {
        #[derive(Serialize)]
        struct Payload {
            now: DateTime<Utc>,
            determinism: contracts::DeterminismKeyV1,
        }

        let payload = serde_json::to_vec(&Payload {
            now,
            determinism: key,
        })?;
        let (ptr, len) = self.write_input(&payload)?;
        let handle = self
            .on_timer
            .call(&mut self.store, (ptr, len))
            .map_err(wasmtime_error)?;
        self.read_json_output(handle, "on-timer")
    }

    fn snapshot_state(&mut self) -> TradingResult<StrategyStateSnapshotV1> {
        let handle = self
            .snapshot_state
            .call(&mut self.store, ())
            .map_err(wasmtime_error)?;
        self.read_json_output(handle, "snapshot-state")
    }

    fn write_input(&mut self, input: &[u8]) -> TradingResult<(i32, i32)> {
        let required_end = INPUT_OFFSET_BYTES + input.len();
        let current_pages = usize::try_from(self.memory.size(&self.store)).map_err(|error| {
            TradingError::Parse {
                source_name: "wasmtime".to_string(),
                details: error.to_string(),
            }
        })?;
        let current_bytes = current_pages * WASM_PAGE_BYTES;
        if required_end > current_bytes {
            let missing = required_end - current_bytes;
            let additional_pages =
                u64::try_from(missing.div_ceil(WASM_PAGE_BYTES)).map_err(|error| {
                    TradingError::Parse {
                        source_name: "wasmtime".to_string(),
                        details: error.to_string(),
                    }
                })?;
            self.memory
                .grow(&mut self.store, additional_pages)
                .map_err(wasmtime_error)?;
        }

        self.memory
            .write(&mut self.store, INPUT_OFFSET_BYTES, input)
            .map_err(wasmtime_error)?;
        Ok((
            i32::try_from(INPUT_OFFSET_BYTES).unwrap_or(i32::MAX),
            i32::try_from(input.len()).unwrap_or(i32::MAX),
        ))
    }

    fn read_json_output<T>(&mut self, handle: i64, source_name: &str) -> TradingResult<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let (ptr, len) = unpack_output_handle(handle)?;
        let data = self.memory.data(&self.store);
        let end = ptr.saturating_add(len);
        let bytes = data.get(ptr..end).ok_or_else(|| TradingError::Parse {
            source_name: source_name.to_string(),
            details: "guest returned an out-of-bounds memory range".to_string(),
        })?;
        serde_json::from_slice(bytes).map_err(|error| TradingError::Parse {
            source_name: source_name.to_string(),
            details: error.to_string(),
        })
    }
}

fn pack_output_handle(ptr: usize, len: usize) -> i64 {
    let ptr = u64::try_from(ptr).unwrap_or(u64::MAX) & 0xffff_ffff;
    let len = u64::try_from(len).unwrap_or(u64::MAX) & 0xffff_ffff;
    i64::try_from((ptr << 32) | len).unwrap_or(i64::MAX)
}

fn unpack_output_handle(handle: i64) -> TradingResult<(usize, usize)> {
    let raw = u64::try_from(handle).map_err(|error| TradingError::Parse {
        source_name: "wasmtime".to_string(),
        details: error.to_string(),
    })?;
    let ptr = usize::try_from(raw >> 32).map_err(|error| TradingError::Parse {
        source_name: "wasmtime".to_string(),
        details: error.to_string(),
    })?;
    let len = usize::try_from(raw & 0xffff_ffff).map_err(|error| TradingError::Parse {
        source_name: "wasmtime".to_string(),
        details: error.to_string(),
    })?;
    Ok((ptr, len))
}

fn wat_string_literal(bytes: &[u8]) -> String {
    let mut output = String::new();
    for byte in bytes {
        match byte {
            b'"' => output.push_str("\\22"),
            b'\\' => output.push_str("\\5c"),
            0x20..=0x7e => output.push(char::from(*byte)),
            _ => {
                let _ = std::fmt::Write::write_fmt(&mut output, format_args!("\\{:02x}", byte));
            }
        }
    }
    output
}

fn wasmtime_error(error: impl std::fmt::Display) -> TradingError {
    TradingError::Parse {
        source_name: "wasmtime".to_string(),
        details: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chrono::TimeZone;
    use contracts::{AssetClassV1, MarketEventV1, OhlcvBarV1, StrategyConfigV1, SymbolV1, VenueV1};
    use trading_core::FixedClock;
    use trading_sim::TrendFollower;

    use super::{demo_strategy_wat, StrategySandbox, WasmRuntimePolicy};

    #[test]
    fn wasmtime_runtime_loads_inline_module() {
        let clock = Arc::new(FixedClock::new(
            chrono::Utc
                .with_ymd_and_hms(2026, 3, 1, 0, 0, 0)
                .single()
                .expect("clock"),
        ));
        let mut runtime = StrategySandbox::new(WasmRuntimePolicy::default(), clock);
        let wat = demo_strategy_wat("trend");

        runtime
            .load_wat(
                "trend_v1",
                &wat,
                StrategyConfigV1 {
                    strategy_id: "trend".to_string(),
                    model_version: "v1".to_string(),
                    config_hash: "abc".to_string(),
                    parameters: serde_json::json!({}),
                },
            )
            .expect("load");

        let signals = runtime
            .on_market_event(
                "trend_v1",
                &MarketEventV1::Bar(OhlcvBarV1 {
                    symbol: SymbolV1::new(VenueV1::Coinbase, AssetClassV1::Crypto, "BTC", "USD"),
                    open_time: chrono::Utc
                        .with_ymd_and_hms(2026, 3, 1, 0, 0, 0)
                        .single()
                        .expect("open"),
                    close_time: chrono::Utc
                        .with_ymd_and_hms(2026, 3, 1, 0, 1, 0)
                        .single()
                        .expect("close"),
                    open: 100.0,
                    high: 101.0,
                    low: 99.0,
                    close: 100.2,
                    volume: 100.0,
                }),
                contracts::DeterminismKeyV1::new("event-1", "v1", "abc"),
            )
            .expect("event");

        assert!(signals.is_empty());
        runtime.unload("trend_v1").expect("unload");
    }

    #[test]
    fn in_memory_loader_remains_available_for_tests() {
        let clock = Arc::new(FixedClock::new(
            chrono::Utc
                .with_ymd_and_hms(2026, 3, 1, 0, 0, 0)
                .single()
                .expect("clock"),
        ));
        let mut runtime = StrategySandbox::new(WasmRuntimePolicy::default(), clock);
        let strategy = TrendFollower::new("trend", 5, 0.1);

        runtime
            .load_in_memory(
                "trend_v1",
                Box::new(strategy),
                StrategyConfigV1 {
                    strategy_id: "trend".to_string(),
                    model_version: "v1".to_string(),
                    config_hash: "abc".to_string(),
                    parameters: serde_json::json!({}),
                },
            )
            .expect("load");

        runtime.unload("trend_v1").expect("unload");
    }
}
