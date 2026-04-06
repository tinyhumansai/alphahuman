//! Event bus handlers for the webhook domain.
//!
//! Placeholder for future cross-domain subscribers that react to webhook events
//! (e.g. metrics collection, audit logging, rate limiting).
//!
//! Webhook events are currently consumed by the [`TracingSubscriber`] for
//! observability. Add domain-specific handlers here as needed following the
//! pattern in `crate::openhuman::cron::bus`.
