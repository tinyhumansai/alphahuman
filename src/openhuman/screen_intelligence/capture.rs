//! Screen capture utilities.

use chrono::Utc;

pub(crate) fn now_ms() -> i64 {
    Utc::now().timestamp_millis()
}
