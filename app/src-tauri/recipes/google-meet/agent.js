// Google Meet Agent — Stage 1: auto-join as a headless attendee.
//
// Role gating:
//   This script runs inside every google-meet webview (injected via
//   initialization_script). It bails immediately if __OPENHUMAN_RECIPE_CTX__.role
//   is not "agent" so the user-facing recipe.js path is unaffected.
//
// Lifecycle events emitted (via window.__openhumanRecipe.emit):
//   meet_agent_joined  { code: string, joinedAt: number }
//   meet_agent_left    { reason: 'leave-button-gone' | 'navigated-away' }
//   meet_agent_failed  { reason: 'timeout' | 'meeting-not-found' | 'meeting-ended' |
//                                'access-denied' | 'invalid-link' | 'unable-to-join' }
//
// Selector library:
//   Ported from Vexa-ai/vexa's platforms/googlemeet/selectors.ts.
//   Vexa uses Playwright selector syntax; we translate to plain DOM via the
//   queryByCssOrText / firstFromList helpers below.
//
// Public API:
//   window.__openhumanMeetAgent.leave()           — best-effort leave click
//   window.__openhumanMeetAgent.pure.*            — testable pure helpers
//
// Out of scope for Stage 1:
//   No avatar rendering, no audio capture, no TTS/STT, no LLM loop.
//   Those come in stages 2-5.

(function () {
  var ctx = window.__OPENHUMAN_RECIPE_CTX__;
  var api = window.__openhumanRecipe;

  // Role gate — bail out immediately if we're not the agent webview.
  if (!ctx || ctx.role !== 'agent') {
    return;
  }

  var meetingUrl = ctx.meetingUrl || '';
  var accountId = ctx.accountId || '';

  if (api) {
    api.log('info', '[meet-agent] starting accountId=' + accountId + ' meetingUrl=' + meetingUrl);
  }

  // ─── Selector constants (ported from Vexa) ────────────────────────────────
  //
  // Arrays are ordered: most specific / most-likely-to-hit first.
  // Playwright selector forms are translated to plain DOM queries by the
  // helpers below. Pure CSS attribute/aria selectors are used as-is.

  var SELECTORS = {
    // Pre-join: Join now / Ask to join button.
    // XPath translated to firstFromList + text filter; :has-text() translated to
    // querySelectorAll + textContent filter.
    joinButton: [
      '//button[.//span[text()="Ask to join"]]',
      'button:has-text("Ask to join")',
      'button:has-text("Join now")',
      'button:has-text("Join")',
    ],

    // Pre-join or in-call microphone toggle.
    // Aria-label heuristic: "Turn off microphone" → mic is currently ON.
    //                        "Turn on microphone"  → mic is currently OFF.
    microphoneToggle: [
      '[aria-label*="Turn off microphone"]',
      'button[aria-label*="Turn off microphone"]',
      'button[aria-label*="Turn on microphone"]',
    ],

    // Pre-join or in-call camera toggle.
    // Aria-label heuristic: "Turn off camera" → cam is currently ON.
    //                        "Turn on camera"  → cam is currently OFF.
    cameraToggle: [
      '[aria-label*="Turn off camera"]',
      'button[aria-label*="Turn off camera"]',
      'button[aria-label*="Turn on camera"]',
    ],

    // Name input (not used in Stage 1 — we rely on the existing login — but
    // included so future stages can fall back to guest-name entry).
    nameInput: [
      'input[type="text"][aria-label="Your name"]',
      'input[placeholder*="name"]',
      'input[placeholder*="Name"]',
    ],

    // Primary leave button (toolbar).
    primaryLeave: [
      'button[aria-label="Leave call"]',
      'button[aria-label*="Leave"]',
      'button[aria-label*="leave"]',
      'button[aria-label*="End meeting"]',
      'button[aria-label*="end meeting"]',
      'button[aria-label*="Hang up"]',
      'button[aria-label*="hang up"]',
      '[role="toolbar"] button[aria-label*="Leave"]',
    ],

    // Secondary leave buttons — confirmation-dialog variants that appear after
    // clicking a primary leave button that opens a dialog.
    secondaryLeave: [
      'button:has-text("Leave meeting")',
      'button:has-text("Just leave the meeting")',
      'button:has-text("Leave")',
      'button:has-text("End meeting")',
      'button:has-text("Hang up")',
      'button:has-text("End call")',
      'button:has-text("Leave call")',
      '[role="dialog"] button:has-text("Leave")',
      '[role="dialog"] button:has-text("End meeting")',
      '[role="alertdialog"] button:has-text("Leave")',
    ],

    // Indicators that appear ONLY when actually admitted to a call.
    // DANGER: toolbar, mic/cam toggles, and leave button also appear in the
    // lobby — do NOT include them here.
    initialAdmissionIndicators: [
      '[data-participant-id]',
      '[data-self-name]',
      'button[aria-label*="Share screen"]',
      'button[aria-label*="Present now"]',
    ],

    // Broader admission indicators (less strict — also usable for state change
    // detection after initialAdmissionIndicators confirms we're in).
    admissionIndicators: [
      'button[aria-label*="Chat"]',
      'button[aria-label*="chat"]',
      'button[aria-label*="People"]',
      'button[aria-label*="people"]',
      'button[aria-label*="Participants"]',
      'button[aria-label*="Leave call"]',
      'button[aria-label*="Leave meeting"]',
      'button[aria-label*="Turn off microphone"]',
      'button[aria-label*="Turn on microphone"]',
      'button[aria-label*="Turn off camera"]',
      'button[aria-label*="Turn on camera"]',
      'button[aria-label*="Share screen"]',
      'button[aria-label*="Present now"]',
      '[role="toolbar"]',
      '[data-participant-id]',
      '[data-self-name]',
      '[data-audio-level]',
    ],

    // Text and aria signals that indicate we are in the waiting room (host has
    // not yet admitted us). These are NON-terminal — keep polling.
    waitingRoomIndicators: [
      'text="Asking to be let in..."',
      'text*="Asking to be let in"',
      'text="You\'ll join the call when someone lets you in"',
      'text*="You\'ll join the call when someone lets you"',
      'text="Please wait until a meeting host brings you into the call"',
      'text="Waiting for the host to let you in"',
      'text="You\'re in the waiting room"',
      'text="Asking to be let in"',
      '[aria-label*="waiting room"]',
      '[aria-label*="Asking to be let in"]',
      '[aria-label*="waiting for admission"]',
    ],

    // Signals that the meeting cannot be joined at all. Terminal — stop polling.
    // Map each selector to a reason string via REJECTION_REASON_MAP below.
    rejectionIndicators: [
      'text="Meeting not found"',
      'text="Can\'t join the meeting"',
      'text="Unable to join"',
      'text="Access denied"',
      'text="Meeting has ended"',
      'text="This meeting has ended"',
      'text="Invalid meeting"',
      'text="Meeting link expired"',
      '[role="dialog"]:has-text("Meeting not found")',
      '[role="alertdialog"]:has-text("Meeting not found")',
      '[role="dialog"]:has-text("Meeting has ended")',
      '[role="alertdialog"]:has-text("Meeting has ended")',
      'button:has-text("Try again")',
      'button:has-text("Retry")',
      'button:has-text("Go back")',
    ],

    // Participant tile containers.
    participantContainers: [
      '[data-participant-id]',
      '[data-self-name]',
      '.participant-tile',
      '.video-tile',
      '[jsname="BOHaEe"]',
    ],

    // Top-level meeting container.
    meetingContainer: [
      '[jsname="BOHaEe"]',
      '[role="main"]',
      'body',
    ],
  };

  // Rejection selector → stable reason string.
  // Ordered to match SELECTORS.rejectionIndicators above (same index).
  var REJECTION_REASON_MAP = [
    'meeting-not-found',   // text="Meeting not found"
    'access-denied',       // text="Can't join the meeting"
    'unable-to-join',      // text="Unable to join"
    'access-denied',       // text="Access denied"
    'meeting-ended',       // text="Meeting has ended"
    'meeting-ended',       // text="This meeting has ended"
    'invalid-link',        // text="Invalid meeting"
    'invalid-link',        // text="Meeting link expired"
    'meeting-not-found',   // [role="dialog"]:has-text("Meeting not found")
    'meeting-not-found',   // [role="alertdialog"]:has-text("Meeting not found")
    'meeting-ended',       // [role="dialog"]:has-text("Meeting has ended")
    'meeting-ended',       // [role="alertdialog"]:has-text("Meeting has ended")
    'unable-to-join',      // button:has-text("Try again")
    'unable-to-join',      // button:has-text("Retry")
    'unable-to-join',      // button:has-text("Go back")
  ];

  // ─── Selector helper toolkit ──────────────────────────────────────────────
  //
  // Translates Vexa's Playwright selector syntax to plain DOM queries.
  // Supported forms:
  //   "button[aria-label='X']"        → plain querySelector
  //   "button:has-text(\"Join now\")" → CSS prefix + textContent filter
  //   "//button[.//span[text()='X']]" → XPath via document.evaluate
  //   "text=\"X\"" / "text*=\"X\""   → text-walker scan (exact / contains)

  /**
   * Parse a :has-text("...") selector.
   * Returns { cssPrefix, text } or null if the selector is not :has-text form.
   * The cssPrefix is the part before :has-text (may be empty string).
   */
  function parseHasText(selector) {
    var m = /^(.*?):has-text\("(.+?)"\)\s*$/.exec(selector);
    if (!m) return null;
    return { cssPrefix: m[1].trim(), text: m[2] };
  }

  /**
   * Parse a Playwright text= or text*= selector.
   * Returns { exact: boolean, text: string } or null.
   * Handles both single and double quoted text values.
   */
  function parseTextSelector(selector) {
    // text="value" or text='value'
    var mExact = /^text=["'](.+?)["']$/.exec(selector);
    if (mExact) return { exact: true, text: mExact[1] };
    // text*="value" or text*='value'
    var mContains = /^text\*=["'](.+?)["']$/.exec(selector);
    if (mContains) return { exact: false, text: mContains[1] };
    return null;
  }

  /**
   * Walk all elements under root and return those whose trimmed textContent
   * matches the predicate.
   */
  function walkText(root, predicate) {
    var results = [];
    try {
      var all = root.querySelectorAll('*');
      for (var i = 0; i < all.length; i++) {
        var el = all[i];
        // Only leaf-ish nodes to avoid matching outer wrappers spuriously.
        var t = (el.textContent || '').trim();
        if (t && predicate(t)) results.push(el);
      }
    } catch (_) {}
    return results;
  }

  /**
   * queryByCssOrText(doc, selector) → Element | null
   *
   * Supported selector forms:
   *   - Plain CSS              → doc.querySelector
   *   - "button:has-text(...)" → CSS prefix + textContent filter
   *   - "//..."                → XPath via document.evaluate
   *   - "text=\"X\""          → exact textContent walk
   *   - "text*=\"X\""         → contains textContent walk
   */
  function queryByCssOrText(doc, selector) {
    var results = queryAllByCssOrText(doc, selector);
    return results.length > 0 ? results[0] : null;
  }

  /**
   * queryAllByCssOrText(doc, selector) → Element[]
   * Like queryByCssOrText but returns all matches.
   */
  function queryAllByCssOrText(doc, selector) {
    if (!selector) return [];
    try {
      // XPath form.
      if (selector.charAt(0) === '/') {
        var xpathResult = doc.evaluate(
          selector,
          doc,
          null,
          XPathResult.ORDERED_NODE_SNAPSHOT_TYPE,
          null
        );
        var xpathNodes = [];
        for (var xi = 0; xi < xpathResult.snapshotLength; xi++) {
          var node = xpathResult.snapshotItem(xi);
          if (node) xpathNodes.push(node);
        }
        return xpathNodes;
      }

      // text= / text*= form.
      var textParsed = parseTextSelector(selector);
      if (textParsed) {
        var needle = textParsed.text.toLowerCase();
        return walkText(doc.body || doc, function (t) {
          var lower = t.toLowerCase();
          return textParsed.exact ? lower === needle : lower.indexOf(needle) !== -1;
        });
      }

      // :has-text("...") form.
      var htParsed = parseHasText(selector);
      if (htParsed) {
        var needle2 = htParsed.text.toLowerCase();
        var candidates;
        if (htParsed.cssPrefix) {
          try {
            candidates = Array.prototype.slice.call(doc.querySelectorAll(htParsed.cssPrefix));
          } catch (_) {
            candidates = [];
          }
        } else {
          candidates = Array.prototype.slice.call(doc.querySelectorAll('*'));
        }
        return candidates.filter(function (el) {
          return (el.textContent || '').toLowerCase().indexOf(needle2) !== -1;
        });
      }

      // Plain CSS.
      return Array.prototype.slice.call(doc.querySelectorAll(selector));
    } catch (_) {}
    return [];
  }

  /**
   * firstFromList(doc, selectors) → Element | null
   * Walk `selectors` in order; return the first non-null visible match.
   */
  function firstFromList(doc, selectors) {
    for (var i = 0; i < selectors.length; i++) {
      var el = queryByCssOrText(doc, selectors[i]);
      if (el) return el;
    }
    return null;
  }

  // ─── Pure helpers (exported for Vitest) ──────────────────────────────────

  /**
   * Extract the meeting code (e.g. "abc-defg-hij") from a full URL string.
   * Returns null when the pathname does not match a Meet room pattern.
   */
  function extractMeetingCode(href) {
    try {
      var pathname = new URL(href).pathname;
      var m = /^\/([a-z]{3,4}-[a-z]{3,4}-[a-z]{3,4})(?:$|\/|\?)/i.exec(pathname);
      return m ? m[1] : null;
    } catch (_) {
      return null;
    }
  }

  /**
   * Locate the pre-join "Join now" / "Ask to join" button.
   * Uses SELECTORS.joinButton via firstFromList. Skips disabled buttons.
   * NOTE: "Switch here" (shown when the user is already in the meeting from
   * another device) is intentionally not handled — see PR #1163.
   */
  function findJoinButton(doc) {
    try {
      for (var i = 0; i < SELECTORS.joinButton.length; i++) {
        var candidates = queryAllByCssOrText(doc, SELECTORS.joinButton[i]);
        for (var j = 0; j < candidates.length; j++) {
          var btn = candidates[j];
          if (!btn.disabled) return btn;
        }
      }
    } catch (_) {}
    return null;
  }

  /**
   * Locate the microphone toggle button.
   * Uses SELECTORS.microphoneToggle via firstFromList.
   */
  function findMicButton(doc) {
    try {
      return firstFromList(doc, SELECTORS.microphoneToggle);
    } catch (_) {}
    return null;
  }

  /**
   * Locate the camera toggle button.
   * Uses SELECTORS.cameraToggle via firstFromList.
   */
  function findCamButton(doc) {
    try {
      return firstFromList(doc, SELECTORS.cameraToggle);
    } catch (_) {}
    return null;
  }

  /**
   * Returns true if the mic appears to be ON (unmuted).
   *
   * Aria-label heuristic (Vexa-derived):
   *   "Turn off microphone" → button's ACTION is to turn off → mic is currently ON.
   *   "Turn on microphone"  → button's ACTION is to turn on  → mic is currently OFF.
   *
   * Falls back to aria-pressed / data-is-muted for older Meet UI versions.
   * Defaults to false (assume off) when state is ambiguous — safer to skip
   * clicking an already-off button than to accidentally turn it on.
   */
  function isMicOn(btn) {
    if (!btn) return false;
    try {
      var label = (btn.getAttribute('aria-label') || '').toLowerCase();
      if (label.indexOf('turn off microphone') !== -1) return true;
      if (label.indexOf('turn on microphone') !== -1) return false;
      // Legacy fallbacks.
      if (btn.getAttribute('aria-pressed') === 'true') return true;
      if (btn.getAttribute('data-is-muted') === 'false') return true;
    } catch (_) {}
    return false;
  }

  /**
   * Returns true if the camera appears to be ON.
   *
   * Aria-label heuristic (same pattern as isMicOn):
   *   "Turn off camera" → cam is currently ON.
   *   "Turn on camera"  → cam is currently OFF.
   */
  function isCamOn(btn) {
    if (!btn) return false;
    try {
      var label = (btn.getAttribute('aria-label') || '').toLowerCase();
      if (label.indexOf('turn off camera') !== -1) return true;
      if (label.indexOf('turn on camera') !== -1) return false;
      // Legacy fallbacks.
      if (btn.getAttribute('aria-pressed') === 'true') return true;
      if (btn.getAttribute('data-is-muted') === 'false') return true;
    } catch (_) {}
    return false;
  }

  /**
   * Returns true when the document shows in-call participant signals.
   *
   * Uses SELECTORS.initialAdmissionIndicators first (strict — only DOM nodes
   * that do NOT appear in the lobby), then SELECTORS.admissionIndicators
   * (broader — also covers older Meet UI variants).
   *
   * [data-self-name] appears on the user's own tile; [data-participant-id]
   * appears on every participant tile. Either is sufficient.
   */
  function isInCall(doc) {
    try {
      for (var i = 0; i < SELECTORS.initialAdmissionIndicators.length; i++) {
        if (queryByCssOrText(doc, SELECTORS.initialAdmissionIndicators[i])) return true;
      }
      for (var j = 0; j < SELECTORS.admissionIndicators.length; j++) {
        if (queryByCssOrText(doc, SELECTORS.admissionIndicators[j])) return true;
      }
    } catch (_) {}
    return false;
  }

  /**
   * Returns true when the document indicates we are in the waiting room
   * (submitted a join request, waiting for the host to admit us).
   *
   * This is NON-terminal — the polling loop keeps running.
   */
  function isInWaitingRoom(doc) {
    try {
      for (var i = 0; i < SELECTORS.waitingRoomIndicators.length; i++) {
        if (queryByCssOrText(doc, SELECTORS.waitingRoomIndicators[i])) return true;
      }
    } catch (_) {}
    return false;
  }

  /**
   * Detect screens where joining is impossible. Returns a reason string or null.
   *
   * Reason strings and their selector mappings:
   *   'meeting-not-found' — "Meeting not found", related dialogs
   *   'meeting-ended'     — "Meeting has ended", "This meeting has ended"
   *   'access-denied'     — "Can't join the meeting", "Access denied"
   *   'invalid-link'      — "Invalid meeting", "Meeting link expired"
   *   'unable-to-join'    — "Unable to join", retry buttons (ambiguous errors)
   *
   * Conservative — unknown screens return null rather than false-positive.
   */
  function isUnjoinableScreen(doc) {
    try {
      for (var i = 0; i < SELECTORS.rejectionIndicators.length; i++) {
        if (queryByCssOrText(doc, SELECTORS.rejectionIndicators[i])) {
          return REJECTION_REASON_MAP[i] || 'unable-to-join';
        }
      }
    } catch (_) {}
    return null;
  }

  /**
   * Locate the "Leave call" button.
   * Walks SELECTORS.primaryLeave first; if nothing found, walks
   * SELECTORS.secondaryLeave (catches confirmation-dialog buttons).
   */
  function findLeaveButton(doc) {
    try {
      var primary = firstFromList(doc, SELECTORS.primaryLeave);
      if (primary) return primary;
      return firstFromList(doc, SELECTORS.secondaryLeave);
    } catch (_) {}
    return null;
  }

  // ─── Main agent loop ──────────────────────────────────────────────────────

  var JOIN_TIMEOUT_MS = 120000; // 120 s — gives host time to admit from waiting room
  var POLL_INTERVAL_MS = 1000;

  var startedAt = Date.now();
  var joinedCode = null;   // non-null once we've emitted meet_agent_joined
  var failedEmitted = false;
  var pollTimer = null;
  var stopped = false;

  // State machine states for transition logging.
  var STATE_LOBBY = 'lobby';
  var STATE_WAITING = 'waiting-room';
  var STATE_IN_CALL = 'in-call';
  var currentState = STATE_LOBBY;
  var waitingRoomLoggedOnce = false;

  function emitOnce(kind, payload) {
    if (!api) return;
    try {
      api.emit(kind, payload);
    } catch (_) {}
  }

  function stopPolling() {
    stopped = true;
    if (pollTimer !== null) {
      clearInterval(pollTimer);
      pollTimer = null;
    }
  }

  function logStateTransition(from, to) {
    if (api) api.log('info', '[meet-agent] state: ' + from + ' → ' + to);
  }

  function poll() {
    if (stopped) return;

    var elapsed = Date.now() - startedAt;
    var doc = document;
    var currentHref = window.location.href;

    // If we're not on the target meeting URL, navigate there.
    var targetCode = extractMeetingCode(meetingUrl);
    var currentCode = extractMeetingCode(currentHref);

    if (targetCode && currentCode !== targetCode) {
      if (api) api.log('debug', '[meet-agent] navigating to meeting url=' + meetingUrl);
      try {
        window.location.replace(meetingUrl);
      } catch (_) {}
      return; // Wait for next poll after navigation.
    }

    // 1. Check for unjoinable screens first (terminal).
    var unjoinable = isUnjoinableScreen(doc);
    if (unjoinable && !failedEmitted) {
      failedEmitted = true;
      if (api) api.log('warn', '[meet-agent] unjoinable screen reason=' + unjoinable);
      emitOnce('meet_agent_failed', { accountId: accountId, reason: unjoinable });
      stopPolling();
      return;
    }

    var inCall = isInCall(doc);

    // 2. Transition: we were in the call and now we're not.
    if (joinedCode && !inCall) {
      var navigatedAway = currentCode && currentCode !== joinedCode;
      var leftReason = navigatedAway ? 'navigated-away' : 'leave-button-gone';
      if (api) api.log('info', '[meet-agent] left call code=' + joinedCode + ' reason=' + leftReason);
      emitOnce('meet_agent_left', { accountId: accountId, reason: leftReason });
      stopPolling();
      return;
    }

    // 3. Transition: just joined.
    if (inCall && !joinedCode) {
      joinedCode = currentCode || targetCode || 'unknown';
      if (currentState !== STATE_IN_CALL) {
        logStateTransition(currentState, STATE_IN_CALL);
        currentState = STATE_IN_CALL;
      }
      if (api) api.log('info', '[meet-agent] joined call code=' + joinedCode);
      emitOnce('meet_agent_joined', {
        accountId: accountId,
        code: joinedCode,
        joinedAt: Date.now(),
      });
      return; // Continue polling to detect leave.
    }

    // 4. Already in call — nothing to do this tick.
    if (inCall && joinedCode) return;

    // 5. Waiting room — non-terminal, keep polling. Do NOT increment failure timer.
    if (isInWaitingRoom(doc)) {
      if (currentState !== STATE_WAITING) {
        logStateTransition(currentState, STATE_WAITING);
        currentState = STATE_WAITING;
        waitingRoomLoggedOnce = false;
      }
      if (!waitingRoomLoggedOnce) {
        if (api) api.log('info', '[meet-agent] waiting for host to admit');
        waitingRoomLoggedOnce = true;
      }
      return; // Do NOT check timeout — waiting room is expected latency.
    }

    // 6. Not yet in call — try to join if we find the join button.
    if (currentState === STATE_WAITING) {
      // Transitioned back to lobby (e.g. re-shown pre-join screen).
      logStateTransition(STATE_WAITING, STATE_LOBBY);
      currentState = STATE_LOBBY;
    }

    var joinBtn = findJoinButton(doc);
    if (joinBtn) {
      // Ensure mic is off.
      var micBtn = findMicButton(doc);
      if (micBtn && isMicOn(micBtn)) {
        if (api) api.log('debug', '[meet-agent] muting mic before join');
        try { micBtn.click(); } catch (_) {}
      }
      // Ensure cam is off.
      var camBtn = findCamButton(doc);
      if (camBtn && isCamOn(camBtn)) {
        if (api) api.log('debug', '[meet-agent] disabling camera before join');
        try { camBtn.click(); } catch (_) {}
      }
      if (api) api.log('info', '[meet-agent] clicking join button');
      try { joinBtn.click(); } catch (_) {}
      return;
    }

    // 7. Timeout check — only applies before we've joined (not during waiting room).
    if (!joinedCode && elapsed >= JOIN_TIMEOUT_MS && !failedEmitted) {
      failedEmitted = true;
      if (api) api.log('warn', '[meet-agent] join timeout after ' + Math.round(elapsed / 1000) + 's');
      emitOnce('meet_agent_failed', { accountId: accountId, reason: 'timeout' });
      stopPolling();
    }
  }

  pollTimer = setInterval(poll, POLL_INTERVAL_MS);

  // Run one tick immediately (don't wait for first interval).
  try { poll(); } catch (_) {}

  // ─── Public API ───────────────────────────────────────────────────────────

  window.__openhumanMeetAgent = {
    /**
     * Best-effort: click the Leave call button.
     * The host Tauri command calls this before closing the webview, but
     * closing the webview is the authoritative teardown — this is just
     * graceful cleanup.
     */
    leave: function () {
      try {
        var btn = findLeaveButton(document);
        if (btn) {
          if (api) api.log('info', '[meet-agent] leave() clicked leave button');
          btn.click();
        } else {
          if (api) api.log('debug', '[meet-agent] leave() leave button not found (host will close webview)');
        }
      } catch (_) {}
      stopPolling();
    },

    /** Pure helpers — exposed for Vitest (no side effects). */
    pure: {
      extractMeetingCode: extractMeetingCode,
      queryByCssOrText: queryByCssOrText,
      queryAllByCssOrText: queryAllByCssOrText,
      firstFromList: firstFromList,
      findJoinButton: findJoinButton,
      findMicButton: findMicButton,
      findCamButton: findCamButton,
      isMicOn: isMicOn,
      isCamOn: isCamOn,
      isInCall: isInCall,
      isInWaitingRoom: isInWaitingRoom,
      findLeaveButton: findLeaveButton,
      isUnjoinableScreen: isUnjoinableScreen,
    },
  };
})();
