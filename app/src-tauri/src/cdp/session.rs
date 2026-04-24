//! Per-account CDP session opener. One long-lived task per webview account
//! that keeps a session attached to the target for the lifetime of the
//! webview so the UA override (and any future per-target overrides) stays
//! applied.
//!
//! Why long-lived: `Emulation.setUserAgentOverride` reverts when the
//! session detaches. If we attached just once and dropped, subsequent HTTP
//! requests + navigator reads would revert to WKWebView defaults.
//!
//! Pairs with the `data:` placeholder URL the webview is created with —
//! the opener finds the target by its unique `openhuman:{account_id}`
//! marker in the initial URL, applies the UA override, then navigates the
//! target to the real provider URL with a `#openhuman-account-{id}`
//! fragment appended so other scanners (discord/telegram/slack/whatsapp)
//! can disambiguate multi-account setups without title-marker injection.

use std::time::Duration;

use serde_json::json;
use tauri::{AppHandle, Runtime};
use tokio::task::JoinHandle;
use tokio::time::sleep;

use super::{browser_ws_url, find_page_target_where, set_user_agent_override, CdpConn, UaSpec};
use crate::webview_accounts::emit_load_finished;

/// Backoff between failed attach attempts / reconnects. Intentionally
/// short — once the webview is open, the target usually shows up within
/// 500ms.
const ATTACH_BACKOFF: Duration = Duration::from_secs(2);

/// Watchdog budget before we synthesise a `webview-account:load` event with
/// `state: "timeout"` so the frontend never holds its loading spinner open on
/// a flaky network. Matches the timeout documented in issue #867.
const LOAD_TIMEOUT: Duration = Duration::from_secs(15);

/// Returns the unique marker substring that the account's initial
/// placeholder URL contains so `Target.getTargets` can identify it. Same
/// marker is embedded into the document title of the placeholder so
/// `TargetInfo.title` can also be used as a fallback match key.
pub fn placeholder_marker(account_id: &str) -> String {
    format!("openhuman-acct-{account_id}")
}

/// Fragment appended to the real provider URL so scanners can match this
/// account uniquely even when several accounts share an origin.
pub fn target_url_fragment(account_id: &str) -> String {
    format!("#openhuman-account-{account_id}")
}

/// Build the `data:` URL used as the webview's initial location. Holding
/// here for the ~hundreds of ms we need to attach CDP + apply overrides
/// before the first real HTTP request. URL-encoded by hand (the payload
/// is tiny, no external dep).
pub fn placeholder_data_url(account_id: &str) -> String {
    let marker = placeholder_marker(account_id);
    format!(
        "data:text/html;charset=utf-8,%3C%21DOCTYPE%20html%3E%3Ctitle%3E{marker}%3C%2Ftitle%3E%3Cbody%20style%3D%22background%3A%23111%22%3E%3C%2Fbody%3E"
    )
}

/// Spawn the per-account CDP session. Returns immediately; the background
/// task keeps the session alive and retries on disconnect. Idempotent at
/// the call site — the caller is expected to only call this once per
/// `webview_account_open`.
///
/// **Shutdown**: returns the `JoinHandle` for the spawned loop so the
/// caller (`webview_account_close` / `webview_account_purge`) can
/// `abort()` it when the account goes away. Without abort the loop
/// would keep retrying `attach_to_target` against a vanished target
/// forever and accumulate across reopen cycles.
pub fn spawn_session<R: Runtime>(
    app: AppHandle<R>,
    account_id: String,
    real_url: String,
) -> JoinHandle<()> {
    // Load-overlay watchdog — independent of the session loop. Emits a
    // `timeout` signal after LOAD_TIMEOUT so the frontend's loading spinner
    // is always released even if neither the native `on_page_load` nor the
    // CDP `Page.loadEventFired` signal arrives (flaky network, provider
    // blocking, CDP socket hiccup).
    //
    // `emit_load_finished` dedups via `WebviewAccountsState.loaded_accounts`
    // so a late watchdog is a no-op once either signal has fired. Spawned
    // detached because we only need the one wake-up.
    {
        let app = app.clone();
        let account_id = account_id.clone();
        let real_url = real_url.clone();
        tokio::spawn(async move {
            sleep(LOAD_TIMEOUT).await;
            emit_load_finished(&app, &account_id, "timeout", &real_url);
        });
    }
    tokio::spawn(async move { run_session_forever(app, account_id, real_url).await })
}

async fn run_session_forever<R: Runtime>(app: AppHandle<R>, account_id: String, real_url: String) {
    log::info!(
        "[cdp-session][{}] up real_url={} marker={}",
        account_id,
        real_url,
        placeholder_marker(&account_id)
    );
    // Let the webview's target appear in CDP before we start hammering
    // `/json/version`. The placeholder URL is tiny so this is quick.
    sleep(Duration::from_millis(500)).await;
    loop {
        match run_session_cycle(&app, &account_id, &real_url).await {
            Ok(()) => {
                log::info!(
                    "[cdp-session][{}] session ended cleanly, reconnecting",
                    account_id
                );
            }
            Err(e) => {
                log::debug!("[cdp-session][{}] cycle failed: {}", account_id, e);
            }
        }
        sleep(ATTACH_BACKOFF).await;
    }
}

async fn run_session_cycle<R: Runtime>(
    app: &AppHandle<R>,
    account_id: &str,
    real_url: &str,
) -> Result<(), String> {
    let browser_ws = browser_ws_url().await?;
    let mut cdp = CdpConn::open(&browser_ws).await?;

    // Account-unique match. Both the placeholder title and the real-URL
    // fragment are appended verbatim, so we can use ends_with / exact
    // equality instead of substring contains — that avoids cross-account
    // collisions like `…account-abc` vs `…account-abcdef`.
    let marker = placeholder_marker(account_id);
    let fragment = target_url_fragment(account_id);
    let target = find_page_target_where(&mut cdp, |t| {
        t.title == marker || t.url.ends_with(&fragment)
    })
    .await?;
    log::info!(
        "[cdp-session][{}] attaching to target {} url={}",
        account_id,
        target.id,
        target.url
    );

    let attach = cdp
        .call(
            "Target.attachToTarget",
            json!({ "targetId": target.id, "flatten": true }),
            None,
        )
        .await?;
    let session_id = attach
        .get("sessionId")
        .and_then(|x| x.as_str())
        .ok_or_else(|| "attach missing sessionId".to_string())?
        .to_string();

    // UA override BEFORE navigate so the first real HTTP request carries
    // the Chrome UA at the network layer AND navigator.* readouts return
    // the spoofed values from the very first page script.
    let ua = UaSpec::chrome_mac();
    set_user_agent_override(&mut cdp, &session_id, &ua).await?;
    log::info!(
        "[cdp-session][{}] ua override applied session={}",
        account_id,
        session_id
    );

    // Stub the Web Notifications permission API before any provider JS
    // runs. Without this, providers like Slack and Gmail show in-app
    // "please enable notifications" banners because Notification.permission
    // returns "default" in the CEF context. The real notification path runs
    // through the CEF IPC hook registered in webview_accounts — this just
    // makes the page's permission check pass.
    cdp.call(
        "Page.addScriptToEvaluateOnNewDocument",
        json!({
            "source": "(function(){\
                function ensureNotificationGranted(){\
                    try {\
                        var NativeNotification = window.Notification;\
                        if (typeof NativeNotification === 'function') {\
                            var OpenHumanNotification = function(title, options){\
                                try { return new NativeNotification(title, options); }\
                                catch (_) { return {}; }\
                            };\
                            OpenHumanNotification.prototype = NativeNotification.prototype;\
                            try {\
                                Object.defineProperty(OpenHumanNotification, 'permission', {\
                                    get: function(){ return 'granted'; },\
                                    configurable: true\
                                });\
                            } catch (_) {}\
                            OpenHumanNotification.requestPermission = function(){\
                                return Promise.resolve('granted');\
                            };\
                            window.Notification = OpenHumanNotification;\
                        }\
                    } catch (_) {}\
                    try {\
                        var p = navigator && navigator.permissions;\
                        if (p && typeof p.query === 'function') {\
                            var q = p.query.bind(p);\
                            var fp = {\
                                query: function(d){\
                                    if (d && d.name === 'notifications') {\
                                        return Promise.resolve({ state: 'granted', onchange: null });\
                                    }\
                                    return q(d);\
                                }\
                            };\
                            Object.defineProperty(navigator, 'permissions', {\
                                get: function(){ return fp; },\
                                configurable: true\
                            });\
                        }\
                    } catch (_) {}\
                }\
                ensureNotificationGranted();\
                try { setInterval(ensureNotificationGranted, 1000); } catch (_) {}\
            })();"
        }),
        Some(&session_id),
    )
    .await?;
    log::debug!(
        "[cdp-session][{}] notification permission stub injected",
        account_id
    );

    // Enable the Page domain so `Page.loadEventFired` reaches our
    // `pump_events` callback below. Must happen BEFORE `Page.navigate` so
    // the first top-level load event for the real provider URL isn't missed.
    cdp.call("Page.enable", json!({}), Some(&session_id))
        .await?;

    // Drive the webview from the placeholder to the real provider URL.
    // Fragment survives same-origin navigations so scanners can match on
    // it indefinitely. Skip navigation if the target is already on the
    // real URL (e.g. we reconnected after a ws drop). Boundary-check
    // the prefix so `https://discord.com` doesn't spuriously match
    // `https://discord.com.evil/…`.
    let at_real_url = target.url.starts_with(real_url)
        && target.url[real_url.len()..]
            .chars()
            .next()
            .is_none_or(|c| matches!(c, '/' | '?' | '#'));
    if !at_real_url {
        let dest = if real_url.contains('#') {
            real_url.to_string()
        } else {
            format!("{real_url}{fragment}")
        };
        log::info!("[cdp-session][{}] navigating to {}", account_id, dest);
        cdp.call("Page.navigate", json!({ "url": dest }), Some(&session_id))
            .await?;
    }

    // Hold the session open for the lifetime of the webview. The UA
    // override reverts when we detach, so we intentionally block here.
    // pump_events returns when the CDP ws closes (browser process exits
    // or `Target.detachFromTarget` is called from elsewhere).
    //
    // The callback emits `webview-account:load{state:"finished"}` on the
    // first `Page.loadEventFired` as a belt-and-braces fallback to the
    // native `WebviewBuilder::on_page_load` handler wired in
    // `webview_account_open`. `emit_load_finished` dedups across both paths
    // so the frontend only sees one signal per cold open.
    let cb_app = app.clone();
    let cb_account_id = account_id.to_string();
    let cb_real_url = real_url.to_string();
    cdp.pump_events(&session_id, move |method, _params| {
        if method == "Page.loadEventFired" {
            emit_load_finished(&cb_app, &cb_account_id, "finished", &cb_real_url);
        }
    })
    .await
}
