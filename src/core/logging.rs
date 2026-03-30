//! Logging for `openhuman run` (and other CLI paths that need stderr output).
//!
//! Without initializing a subscriber, `log::` and `tracing::` macros are no-ops.

use std::sync::Once;

static INIT: Once = Once::new();

/// Initialize `tracing` + bridge the `log` crate so existing `log::info!` calls appear.
///
/// - If `RUST_LOG` is unset: uses `info`, or `debug` when `verbose` is true.
/// - Safe to call once; subsequent calls are ignored.
pub fn init_for_cli_run(verbose: bool) {
    INIT.call_once(|| {
        if std::env::var_os("RUST_LOG").is_none() {
            std::env::set_var("RUST_LOG", if verbose { "debug" } else { "info" });
        }

        let filter = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            tracing_subscriber::EnvFilter::new(if verbose { "debug" } else { "info" })
        });

        let _ = tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_target(true)
            .try_init();

        let _ = tracing_log::LogTracer::init();
    });
}
