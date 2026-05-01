//! Decision logic — turn raw [`Signals`] + user config into a [`Policy`].

use crate::openhuman::config::SchedulerGateConfig;
use crate::openhuman::scheduler_gate::signals::Signals;

/// Background-AI scheduling tier. See module docs in `mod.rs` for semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Policy {
    Aggressive,
    Normal,
    Throttled,
    Paused,
}

impl Policy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Aggressive => "aggressive",
            Self::Normal => "normal",
            Self::Throttled => "throttled",
            Self::Paused => "paused",
        }
    }
}

/// Compute the current [`Policy`] from sampled signals + user config.
///
/// Order of evaluation matters — explicit user overrides win first, then
/// deployment mode, then dynamic host signals.
pub fn decide(signals: &Signals, cfg: &SchedulerGateConfig) -> Policy {
    use crate::openhuman::config::SchedulerGateMode;

    match cfg.mode {
        SchedulerGateMode::Off => return Policy::Paused,
        SchedulerGateMode::AlwaysOn => return Policy::Aggressive,
        SchedulerGateMode::Auto => {}
    }

    if signals.server_mode {
        return Policy::Aggressive;
    }

    let battery_ok = signals.on_ac_power
        || signals
            .battery_charge
            .map(|c| c >= cfg.battery_floor)
            .unwrap_or(true); // no battery present == treat as plugged in

    let cpu_ok = signals.cpu_usage_pct <= cfg.cpu_busy_threshold_pct;

    if battery_ok && cpu_ok {
        Policy::Normal
    } else {
        Policy::Throttled
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openhuman::config::{SchedulerGateConfig, SchedulerGateMode};

    fn cfg(mode: SchedulerGateMode) -> SchedulerGateConfig {
        SchedulerGateConfig {
            mode,
            battery_floor: 0.8,
            cpu_busy_threshold_pct: 70.0,
            throttled_backoff_ms: 30_000,
            paused_poll_ms: 60_000,
        }
    }

    fn signals(on_ac: bool, charge: Option<f32>, cpu: f32, server: bool) -> Signals {
        Signals {
            on_ac_power: on_ac,
            battery_charge: charge,
            cpu_usage_pct: cpu,
            server_mode: server,
        }
    }

    #[test]
    fn off_mode_pauses() {
        let p = decide(
            &signals(true, None, 5.0, true),
            &cfg(SchedulerGateMode::Off),
        );
        assert_eq!(p, Policy::Paused);
    }

    #[test]
    fn always_on_overrides_signals() {
        // discharging laptop at 10% with 99% CPU — still Aggressive.
        let p = decide(
            &signals(false, Some(0.10), 99.0, false),
            &cfg(SchedulerGateMode::AlwaysOn),
        );
        assert_eq!(p, Policy::Aggressive);
    }

    #[test]
    fn server_mode_is_aggressive() {
        let p = decide(
            &signals(false, None, 50.0, true),
            &cfg(SchedulerGateMode::Auto),
        );
        assert_eq!(p, Policy::Aggressive);
    }

    #[test]
    fn plugged_in_idle_is_normal() {
        let p = decide(
            &signals(true, Some(0.45), 20.0, false),
            &cfg(SchedulerGateMode::Auto),
        );
        assert_eq!(p, Policy::Normal);
    }

    #[test]
    fn battery_above_floor_is_normal() {
        let p = decide(
            &signals(false, Some(0.85), 20.0, false),
            &cfg(SchedulerGateMode::Auto),
        );
        assert_eq!(p, Policy::Normal);
    }

    #[test]
    fn battery_below_floor_throttles() {
        let p = decide(
            &signals(false, Some(0.30), 20.0, false),
            &cfg(SchedulerGateMode::Auto),
        );
        assert_eq!(p, Policy::Throttled);
    }

    #[test]
    fn busy_cpu_throttles_even_when_plugged_in() {
        let p = decide(
            &signals(true, Some(0.95), 90.0, false),
            &cfg(SchedulerGateMode::Auto),
        );
        assert_eq!(p, Policy::Throttled);
    }

    #[test]
    fn no_battery_treated_as_plugged_in() {
        // Desktop / server with no battery sensor — treat as AC.
        let p = decide(
            &signals(false, None, 20.0, false),
            &cfg(SchedulerGateMode::Auto),
        );
        assert_eq!(p, Policy::Normal);
    }
}
