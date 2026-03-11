#[cfg(all(feature = "browser-tracing", target_arch = "wasm32"))]
use std::io::{self, Write};
use std::sync::atomic::AtomicBool;

use crate::{TracingBootstrapConfig, TracingBootstrapError, TracingBootstrapState, install_once};

#[cfg(feature = "browser-tracing")]
static BROWSER_BOOTSTRAP_INSTALLED: AtomicBool = AtomicBool::new(false);
#[cfg(feature = "native-tracing")]
static NATIVE_BOOTSTRAP_INSTALLED: AtomicBool = AtomicBool::new(false);

#[cfg(all(feature = "browser-tracing", target_arch = "wasm32"))]
pub fn bootstrap_browser_tracing(
    config: &TracingBootstrapConfig,
) -> Result<TracingBootstrapState, TracingBootstrapError> {
    install_once(&BROWSER_BOOTSTRAP_INSTALLED, || {
        tracing_subscriber::fmt()
            .with_timer(BrowserIsoTimer)
            .with_env_filter(env_filter(config))
            .with_writer(BrowserConsoleWriter::default)
            .with_ansi(false)
            .compact()
            .try_init()
            .map_err(|error| {
                TracingBootstrapError::new(format!(
                    "failed to install browser tracing subscriber: {error}"
                ))
            })
    })
}

#[cfg(all(feature = "browser-tracing", not(target_arch = "wasm32")))]
pub fn bootstrap_browser_tracing(
    _config: &TracingBootstrapConfig,
) -> Result<TracingBootstrapState, TracingBootstrapError> {
    let _ = &BROWSER_BOOTSTRAP_INSTALLED;
    Err(TracingBootstrapError::new(
        "browser tracing bootstrap is only available on wasm32 targets",
    ))
}

#[cfg(all(feature = "native-tracing", not(target_arch = "wasm32")))]
pub fn bootstrap_native_tracing(
    config: &TracingBootstrapConfig,
) -> Result<TracingBootstrapState, TracingBootstrapError> {
    install_once(&NATIVE_BOOTSTRAP_INSTALLED, || init_native_tracing(config))
}

#[cfg(all(feature = "native-tracing", target_arch = "wasm32"))]
pub fn bootstrap_native_tracing(
    _config: &TracingBootstrapConfig,
) -> Result<TracingBootstrapState, TracingBootstrapError> {
    let _ = &NATIVE_BOOTSTRAP_INSTALLED;
    Err(TracingBootstrapError::new(
        "native tracing bootstrap is unavailable on wasm32 targets",
    ))
}

#[cfg(all(feature = "browser-tracing", target_arch = "wasm32"))]
fn env_filter(config: &TracingBootstrapConfig) -> tracing_subscriber::EnvFilter {
    tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(config.default_filter.clone()))
}

#[cfg(all(feature = "browser-tracing", target_arch = "wasm32"))]
#[derive(Clone, Copy, Debug, Default)]
struct BrowserIsoTimer;

#[cfg(all(feature = "browser-tracing", target_arch = "wasm32"))]
impl tracing_subscriber::fmt::time::FormatTime for BrowserIsoTimer {
    fn format_time(
        &self,
        writer: &mut tracing_subscriber::fmt::format::Writer<'_>,
    ) -> std::fmt::Result {
        write!(
            writer,
            "{}",
            String::from(js_sys::Date::new_0().to_iso_string())
        )
    }
}

#[cfg(all(feature = "browser-tracing", target_arch = "wasm32"))]
#[derive(Default)]
struct BrowserConsoleWriter {
    buffer: Vec<u8>,
}

#[cfg(all(feature = "browser-tracing", target_arch = "wasm32"))]
impl Write for BrowserConsoleWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buffer.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        let message = String::from_utf8_lossy(&self.buffer).trim().to_string();
        if !message.is_empty() {
            web_sys::console::log_1(&message.into());
        }
        self.buffer.clear();
        Ok(())
    }
}

#[cfg(all(feature = "browser-tracing", target_arch = "wasm32"))]
impl Drop for BrowserConsoleWriter {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}

#[cfg(all(feature = "native-tracing", not(target_arch = "wasm32")))]
fn env_filter(config: &TracingBootstrapConfig) -> tracing_subscriber::EnvFilter {
    tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(config.default_filter.clone()))
}

#[cfg(all(feature = "native-tracing", not(target_arch = "wasm32")))]
fn init_native_tracing(config: &TracingBootstrapConfig) -> Result<(), TracingBootstrapError> {
    if config.enable_tokio_console {
        return init_native_tracing_with_console(config);
    }
    init_native_tracing_without_console(config)
}

#[cfg(all(feature = "native-tracing", not(target_arch = "wasm32")))]
fn init_native_tracing_without_console(
    config: &TracingBootstrapConfig,
) -> Result<(), TracingBootstrapError> {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    tracing_subscriber::registry()
        .with(env_filter(config))
        .with(
            tracing_subscriber::fmt::layer()
                .json()
                .with_ansi(false)
                .flatten_event(true)
                .with_current_span(false)
                .with_span_list(false),
        )
        .try_init()
        .map_err(|error| {
            TracingBootstrapError::new(format!(
                "failed to install native tracing subscriber: {error}"
            ))
        })
}

#[cfg(all(
    feature = "native-tracing",
    not(target_arch = "wasm32"),
    tokio_unstable
))]
fn init_native_tracing_with_console(
    config: &TracingBootstrapConfig,
) -> Result<(), TracingBootstrapError> {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    let console_layer = console_subscriber::ConsoleLayer::builder()
        .with_default_env()
        .spawn();

    tracing_subscriber::registry()
        .with(env_filter(config))
        .with(
            tracing_subscriber::fmt::layer()
                .json()
                .with_ansi(false)
                .flatten_event(true)
                .with_current_span(false)
                .with_span_list(false),
        )
        .with(console_layer)
        .try_init()
        .map_err(|error| {
            TracingBootstrapError::new(format!(
                "failed to install native tracing subscriber with tokio-console: {error}"
            ))
        })
}

#[cfg(all(
    feature = "native-tracing",
    not(target_arch = "wasm32"),
    not(tokio_unstable)
))]
fn init_native_tracing_with_console(
    _config: &TracingBootstrapConfig,
) -> Result<(), TracingBootstrapError> {
    Err(TracingBootstrapError::new(
        "tokio-console requested without the required `tokio_unstable` compile-time cfg",
    ))
}
