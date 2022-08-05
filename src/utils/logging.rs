use once_cell::sync::Lazy;
use tracing_subscriber::{
    filter::EnvFilter,
    fmt,
    fmt::{format::FmtSpan, time::UtcTime},
};

pub fn init_tracing() {
    // Configure the default `tracing` subscriber.
    // The `fmt` subscriber from the `tracing-subscriber` crate logs `tracing`
    // events to stdout. Other subscribers are available for integrating with
    // distributed tracing systems such as OpenTelemetry.
    fmt()
        .json()
        .with_timer::<_>(UtcTime::rfc_3339())
        .flatten_event(true)
        .with_target(true)
        .with_current_span(true)
        // Use the filter we built above to determine which traces to record.
        .with_env_filter(get_log_filter())
        .with_filter_reloading()
        // Record an event when each span closes. This can be used to time our
        // routes' durations!
        .with_span_events(FmtSpan::ACTIVE)
        .init();

    // Capture the span context in which the program panicked
    std::panic::set_hook(Box::new(|panic| {
        // If the panic has a source location, record it as structured fields.
        if let Some(location) = panic.location() {
            tracing::error!(
                message = %panic,
                panic.file = location.file(),
                panic.line = location.line(),
                panic.column = location.column(),
            );
        } else {
            tracing::error!(message = %panic);
        }
    }));
}

#[inline(always)]
fn get_rust_log() -> &'static str {
    static RUST_LOG: Lazy<String> =
        Lazy::new(|| std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_owned()));
    &RUST_LOG
}

#[inline(always)]
fn get_log_filter() -> EnvFilter {
    EnvFilter::try_new(get_rust_log()).expect("Wrong RUST_LOG filter")
}
