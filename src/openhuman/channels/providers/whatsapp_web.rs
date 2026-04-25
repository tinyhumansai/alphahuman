//! WhatsApp Web channel backed by upstream [`whatsapp-rust`] 0.5.
//!
//! # Why the upgrade
//!
//! The previous implementation used `wa-rs` 0.2 (a fork that pinned to stable
//! Rust). That fork silently dropped `Event::Message` for LID-addressed
//! contacts and group sender-key (`skmsg`) messages: the protocol layer
//! decrypted the payload but never dispatched it to user code, breaking
//! agent dispatch for the bulk of modern WhatsApp traffic (LID is the
//! current default). Upstream `whatsapp-rust` 0.5 fixed this in PRs #170
//! (SKDM tracking) + #181 (LID/PN mapping) + sender-key dispatch, and also
//! ships its own [`SqliteStore`] — so the previous custom 1,345-line
//! `RusqliteStore` is no longer needed.
//!
//! # Feature Flag
//!
//! ```sh
//! cargo build --features whatsapp-web
//! ```
//!
//! # Configuration
//!
//! ```toml
//! [channels.whatsapp]
//! session_path = "~/.openhuman/whatsapp-session.db"  # Required for Web mode
//! pair_phone = "15551234567"                         # Optional: pair-code linking
//! allowed_numbers = ["+1234567890", "*"]             # Same shape as Cloud API
//! ```
//!
//! # Runtime negotiation
//!
//! Selected automatically by [`crate::openhuman::channels::runtime::startup`]
//! when `session_path` is set. The Cloud API channel ([`super::whatsapp`]) is
//! used when `phone_number_id` is set instead.
//!
//! # Migration note
//!
//! The on-disk SQLite schema differs between the wa-rs 0.2 fork and the
//! upstream 0.5 store. Existing paired sessions will fail to load and will
//! prompt for a fresh QR scan on first launch after this upgrade. Pairing
//! takes about 30 seconds; the old `whatsapp-session.db` can be deleted by
//! the user afterwards.
//!
//! [`whatsapp-rust`]: https://docs.rs/whatsapp-rust/0.5
//! [`SqliteStore`]: whatsapp_rust::store::SqliteStore

use crate::openhuman::channels::traits::{Channel, ChannelMessage, SendMessage};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use parking_lot::Mutex;
use std::sync::Arc;

/// WhatsApp Web channel.
///
/// Wraps a `whatsapp-rust` Bot with our `Channel` trait. The bot owns an
/// `Arc<Client>` for outbound operations (`send`, typing) and a `BotHandle`
/// for shutdown. Inbound messages are pushed onto an [`mpsc::Sender`] so
/// the existing channel inbound subscriber pipeline can process them.
#[cfg(feature = "whatsapp-web")]
pub struct WhatsAppWebChannel {
    /// Path to the SQLite session database.
    session_path: String,
    /// Optional phone number for pair-code linking (E.164 digits, no leading `+`).
    pair_phone: Option<String>,
    /// Optional pre-allocated pair code paired with `pair_phone`.
    pair_code: Option<String>,
    /// E.164 numbers (with leading `+`) allowed to interact, or `["*"]` for any.
    /// Empty also means "allow all" — same convention as the Cloud API channel.
    allowed_numbers: Vec<String>,
    /// Bot run handle, retained for graceful shutdown.
    bot_handle: Arc<Mutex<Option<whatsapp_rust::bot::BotHandle>>>,
    /// Live client used for outbound calls; populated after `Bot::build` returns.
    client: Arc<Mutex<Option<Arc<whatsapp_rust::Client>>>>,
    /// Sink for inbound `ChannelMessage`s. Populated when [`Channel::listen`]
    /// is called and shared with the event-handler closure.
    tx: Arc<Mutex<Option<tokio::sync::mpsc::Sender<ChannelMessage>>>>,
}

#[cfg(feature = "whatsapp-web")]
impl WhatsAppWebChannel {
    /// Construct a channel. The bot does not connect until [`Channel::listen`]
    /// is invoked.
    pub fn new(
        session_path: String,
        pair_phone: Option<String>,
        pair_code: Option<String>,
        allowed_numbers: Vec<String>,
    ) -> Self {
        Self {
            session_path,
            pair_phone,
            pair_code,
            allowed_numbers,
            bot_handle: Arc::new(Mutex::new(None)),
            client: Arc::new(Mutex::new(None)),
            tx: Arc::new(Mutex::new(None)),
        }
    }

    /// Allowlist check. Empty list ⇒ allow-all (matches Cloud API behaviour).
    fn is_number_allowed(&self, phone: &str) -> bool {
        self.allowed_numbers.is_empty()
            || self.allowed_numbers.iter().any(|n| n == "*" || n == phone)
    }

    /// Render an arbitrary recipient string as E.164 with a leading `+`,
    /// stripping any `@server` JID suffix the caller passed in.
    fn normalize_phone(&self, phone: &str) -> String {
        let trimmed = phone.trim();
        let user_part = trimmed
            .split_once('@')
            .map(|(user, _)| user)
            .unwrap_or(trimmed);
        let normalized_user = user_part.trim_start_matches('+');
        format!("+{normalized_user}")
    }

    /// Convert a recipient (full JID like `12345@s.whatsapp.net` or an E.164
    /// number like `+1234567890`) into a `whatsapp-rust` JID.
    fn recipient_to_jid(&self, recipient: &str) -> Result<whatsapp_rust::Jid> {
        let trimmed = recipient.trim();
        if trimmed.is_empty() {
            anyhow::bail!("Recipient cannot be empty");
        }

        if trimmed.contains('@') {
            return trimmed
                .parse::<whatsapp_rust::Jid>()
                .map_err(|e| anyhow!("Invalid WhatsApp JID `{trimmed}`: {e}"));
        }

        let digits: String = trimmed.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.is_empty() {
            anyhow::bail!("Recipient `{trimmed}` does not contain a valid phone number");
        }

        Ok(whatsapp_rust::Jid::pn(digits))
    }
}

#[cfg(feature = "whatsapp-web")]
#[async_trait]
impl Channel for WhatsAppWebChannel {
    fn name(&self) -> &str {
        "whatsapp"
    }

    async fn send(&self, message: &SendMessage) -> Result<()> {
        let client = self.client.lock().clone();
        let Some(client) = client else {
            anyhow::bail!("WhatsApp Web client not connected. Initialize the bot first.");
        };

        let normalized = self.normalize_phone(&message.recipient);
        if !self.is_number_allowed(&normalized) {
            tracing::warn!(
                "WhatsApp Web: recipient {} not in allowed list",
                message.recipient
            );
            return Ok(());
        }

        let to = self.recipient_to_jid(&message.recipient)?;
        let outgoing = whatsapp_rust::waproto::whatsapp::Message {
            conversation: Some(message.content.clone()),
            ..Default::default()
        };

        let message_id = client.send_message(to, outgoing).await?;
        tracing::debug!(
            "WhatsApp Web: sent message to {} (id: {})",
            message.recipient,
            message_id
        );
        Ok(())
    }

    async fn listen(&self, tx: tokio::sync::mpsc::Sender<ChannelMessage>) -> Result<()> {
        *self.tx.lock() = Some(tx.clone());

        use wacore::types::events::Event;
        use whatsapp_rust::bot::Bot;
        use whatsapp_rust::pair_code::PairCodeOptions;
        use whatsapp_rust::store::SqliteStore;
        use whatsapp_rust::TokioRuntime;
        use whatsapp_rust_tokio_transport::TokioWebSocketTransportFactory;
        use whatsapp_rust_ureq_http_client::UreqHttpClient;

        tracing::info!(
            "WhatsApp Web channel starting (session: {})",
            self.session_path
        );

        // Upstream's SqliteStore implements all four storage traits the bot
        // needs (Signal, AppSync, Protocol, Device). It also handles
        // first-run schema creation, so no separate `exists`/`load` dance.
        let backend = Arc::new(SqliteStore::new(&self.session_path).await?);

        let mut transport_factory = TokioWebSocketTransportFactory::new();
        if let Ok(ws_url) = std::env::var("WHATSAPP_WS_URL") {
            transport_factory = transport_factory.with_url(ws_url);
        }

        let http_client = UreqHttpClient::new();

        let tx_for_handler = tx.clone();
        let allowed_numbers = self.allowed_numbers.clone();

        let mut builder = Bot::builder()
            .with_backend(backend)
            .with_transport_factory(transport_factory)
            .with_http_client(http_client)
            .with_runtime(TokioRuntime)
            .on_event(move |event, _client| {
                let tx_inner = tx_for_handler.clone();
                let allowed_numbers = allowed_numbers.clone();
                async move {
                    match event {
                        Event::Message(msg, info) => {
                            // Self-echoes (messages this user sent from another
                            // linked device) are mirrored to all devices via
                            // the WhatsApp protocol. Drop them so the agent
                            // doesn't react to its own outgoing messages.
                            if info.source.is_from_me {
                                return;
                            }

                            let text = msg.conversation.clone().unwrap_or_else(|| {
                                msg.extended_text_message
                                    .as_ref()
                                    .and_then(|e| e.text.clone())
                                    .unwrap_or_default()
                            });

                            // Sender JID can use either the legacy `s.whatsapp.net`
                            // server (phone-number addressing) or the newer `lid`
                            // server (privacy-preserving identifier). Render the
                            // user portion in E.164 with a leading `+` for the
                            // allowed-list check + downstream subscriber.
                            let sender_user = info.source.sender.user.clone();
                            let normalized = if sender_user.starts_with('+') {
                                sender_user.clone()
                            } else {
                                format!("+{sender_user}")
                            };
                            let chat = info.source.chat.to_string();

                            tracing::info!(
                                "📨 WhatsApp message from {} in {}: {}",
                                normalized,
                                chat,
                                text
                            );

                            if allowed_numbers.is_empty()
                                || allowed_numbers.iter().any(|n| n == "*" || n == &normalized)
                            {
                                if let Err(e) = tx_inner
                                    .send(ChannelMessage {
                                        id: uuid::Uuid::new_v4().to_string(),
                                        channel: "whatsapp".to_string(),
                                        sender: normalized.clone(),
                                        reply_target: normalized.clone(),
                                        content: text,
                                        timestamp: chrono::Utc::now().timestamp_millis() as u64,
                                        thread_ts: None,
                                    })
                                    .await
                                {
                                    tracing::error!(
                                        "Failed to forward WhatsApp message to channel: {}",
                                        e
                                    );
                                }
                            } else {
                                tracing::warn!(
                                    "WhatsApp Web: message from {} not in allowed list",
                                    normalized
                                );
                            }
                        }
                        Event::Connected(_) => {
                            tracing::info!("✅ WhatsApp Web connected successfully!");
                        }
                        Event::LoggedOut(_) => {
                            tracing::warn!("❌ WhatsApp Web was logged out!");
                        }
                        Event::StreamError(stream_error) => {
                            tracing::error!("❌ WhatsApp Web stream error: {:?}", stream_error);
                        }
                        Event::PairingCode { code, .. } => {
                            tracing::info!("🔑 Pair code received: {}", code);
                            tracing::info!(
                                "Link your phone by entering this code in WhatsApp > Linked Devices"
                            );
                        }
                        Event::PairingQrCode { code, .. } => {
                            tracing::info!(
                                "📱 QR code received (scan with WhatsApp > Linked Devices)"
                            );
                            tracing::debug!("QR code: {}", code);
                        }
                        _ => {}
                    }
                }
            });

        if let Some(ref phone) = self.pair_phone {
            tracing::info!("WhatsApp Web: pair-code flow enabled for configured phone number");
            builder = builder.with_pair_code(PairCodeOptions {
                phone_number: phone.clone(),
                custom_code: self.pair_code.clone(),
                ..Default::default()
            });
        } else if self.pair_code.is_some() {
            tracing::warn!(
                "WhatsApp Web: pair_code is set but pair_phone is missing; pair code config is ignored"
            );
        }

        let mut bot = builder.build().await?;
        *self.client.lock() = Some(bot.client());

        let bot_handle = bot.run().await?;
        *self.bot_handle.lock() = Some(bot_handle);

        tokio::signal::ctrl_c().await.ok();
        tracing::info!("WhatsApp Web channel received Ctrl+C — shutting down");

        *self.client.lock() = None;
        if let Some(handle) = self.bot_handle.lock().take() {
            handle.abort();
        }

        Ok(())
    }

    async fn health_check(&self) -> bool {
        self.bot_handle.lock().is_some()
    }

    async fn start_typing(&self, recipient: &str) -> Result<()> {
        let client = self.client.lock().clone();
        let Some(client) = client else {
            anyhow::bail!("WhatsApp Web client not connected. Initialize the bot first.");
        };

        let normalized = self.normalize_phone(recipient);
        if !self.is_number_allowed(&normalized) {
            tracing::warn!(
                "WhatsApp Web: typing target {} not in allowed list",
                recipient
            );
            return Ok(());
        }

        let to = self.recipient_to_jid(recipient)?;
        client
            .chatstate()
            .send_composing(&to)
            .await
            .map_err(|e| anyhow!("Failed to send typing state (composing): {e}"))?;

        tracing::debug!("WhatsApp Web: start typing for {}", recipient);
        Ok(())
    }

    async fn stop_typing(&self, recipient: &str) -> Result<()> {
        let client = self.client.lock().clone();
        let Some(client) = client else {
            anyhow::bail!("WhatsApp Web client not connected. Initialize the bot first.");
        };

        let normalized = self.normalize_phone(recipient);
        if !self.is_number_allowed(&normalized) {
            tracing::warn!(
                "WhatsApp Web: typing target {} not in allowed list",
                recipient
            );
            return Ok(());
        }

        let to = self.recipient_to_jid(recipient)?;
        client
            .chatstate()
            .send_paused(&to)
            .await
            .map_err(|e| anyhow!("Failed to send typing state (paused): {e}"))?;

        tracing::debug!("WhatsApp Web: stop typing for {}", recipient);
        Ok(())
    }
}

// Stub implementation when the feature is not enabled. Keeps the public ctor
// signature compatible so `runtime/startup.rs` compiles unchanged.
#[cfg(not(feature = "whatsapp-web"))]
pub struct WhatsAppWebChannel {
    _private: (),
}

#[cfg(not(feature = "whatsapp-web"))]
impl WhatsAppWebChannel {
    pub fn new(
        _session_path: String,
        _pair_phone: Option<String>,
        _pair_code: Option<String>,
        _allowed_numbers: Vec<String>,
    ) -> Self {
        Self { _private: () }
    }
}

#[cfg(not(feature = "whatsapp-web"))]
#[async_trait]
impl Channel for WhatsAppWebChannel {
    fn name(&self) -> &str {
        "whatsapp"
    }

    async fn send(&self, _message: &SendMessage) -> Result<()> {
        anyhow::bail!(
            "WhatsApp Web channel requires the 'whatsapp-web' feature. \
            Enable with: cargo build --features whatsapp-web"
        );
    }

    async fn listen(&self, _tx: tokio::sync::mpsc::Sender<ChannelMessage>) -> Result<()> {
        anyhow::bail!(
            "WhatsApp Web channel requires the 'whatsapp-web' feature. \
            Enable with: cargo build --features whatsapp-web"
        );
    }

    async fn health_check(&self) -> bool {
        false
    }

    async fn start_typing(&self, _recipient: &str) -> Result<()> {
        anyhow::bail!(
            "WhatsApp Web channel requires the 'whatsapp-web' feature. \
            Enable with: cargo build --features whatsapp-web"
        );
    }

    async fn stop_typing(&self, _recipient: &str) -> Result<()> {
        anyhow::bail!(
            "WhatsApp Web channel requires the 'whatsapp-web' feature. \
            Enable with: cargo build --features whatsapp-web"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "whatsapp-web")]
    fn make_channel() -> WhatsAppWebChannel {
        WhatsAppWebChannel::new(
            "/tmp/test-whatsapp.db".into(),
            None,
            None,
            vec!["+1234567890".into()],
        )
    }

    #[test]
    #[cfg(feature = "whatsapp-web")]
    fn whatsapp_web_channel_name() {
        let ch = make_channel();
        assert_eq!(ch.name(), "whatsapp");
    }

    #[test]
    #[cfg(feature = "whatsapp-web")]
    fn whatsapp_web_number_allowed_exact() {
        let ch = make_channel();
        assert!(ch.is_number_allowed("+1234567890"));
        assert!(!ch.is_number_allowed("+9876543210"));
    }

    #[test]
    #[cfg(feature = "whatsapp-web")]
    fn whatsapp_web_number_allowed_wildcard() {
        let ch = WhatsAppWebChannel::new("/tmp/test.db".into(), None, None, vec!["*".into()]);
        assert!(ch.is_number_allowed("+1234567890"));
        assert!(ch.is_number_allowed("+9999999999"));
    }

    #[test]
    #[cfg(feature = "whatsapp-web")]
    fn whatsapp_web_number_denied_empty() {
        let ch = WhatsAppWebChannel::new("/tmp/test.db".into(), None, None, vec![]);
        // Empty allowed_numbers means "allow all" (same behavior as Cloud API)
        assert!(ch.is_number_allowed("+1234567890"));
    }

    #[test]
    #[cfg(feature = "whatsapp-web")]
    fn whatsapp_web_normalize_phone_adds_plus() {
        let ch = make_channel();
        assert_eq!(ch.normalize_phone("1234567890"), "+1234567890");
    }

    #[test]
    #[cfg(feature = "whatsapp-web")]
    fn whatsapp_web_normalize_phone_preserves_plus() {
        let ch = make_channel();
        assert_eq!(ch.normalize_phone("+1234567890"), "+1234567890");
    }

    #[tokio::test]
    #[cfg(feature = "whatsapp-web")]
    async fn whatsapp_web_health_check_disconnected() {
        let ch = make_channel();
        assert!(!ch.health_check().await);
    }
}
