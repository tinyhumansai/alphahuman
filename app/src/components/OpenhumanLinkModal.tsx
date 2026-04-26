/**
 * Modal popped open when an `<openhuman-link path="...">` pill is clicked
 * inside an agent message bubble.
 *
 * The pill dispatches a `window` `CustomEvent('openhuman-link', { detail: { path } })`;
 * this component listens for it, opens the modal, and routes to a focused
 * mini-flow per path. Keeps the chat in view (no react-router navigation)
 * so the user can complete the action and return to the agent without
 * losing the conversation.
 *
 * Mounted once at AppShell root.
 */
import { useCallback, useEffect, useMemo, useState } from 'react';

import { useChannelDefinitions } from '../hooks/useChannelDefinitions';
import { showNativeNotification } from '../lib/nativeNotifications/tauriBridge';
import { openUrl } from '../utils/openUrl';
import ChannelSetupModal from './channels/ChannelSetupModal';

interface OpenhumanLinkEvent {
  path: string;
}

export const OPENHUMAN_LINK_EVENT = 'openhuman-link';

const OpenhumanLinkModal = () => {
  const [activePath, setActivePath] = useState<string | null>(null);

  useEffect(() => {
    const handler = (event: Event) => {
      const detail = (event as CustomEvent<OpenhumanLinkEvent>).detail;
      if (detail?.path) setActivePath(detail.path);
    };
    window.addEventListener(OPENHUMAN_LINK_EVENT, handler);
    return () => window.removeEventListener(OPENHUMAN_LINK_EVENT, handler);
  }, []);

  const close = useCallback(() => setActivePath(null), []);

  if (!activePath) return null;

  // Telegram (and any future channel) gets the dedicated `ChannelSetupModal`
  // already used by Skills + Settings instead of a bespoke body wrapper.
  // It manages its own portal + backdrop, so render it standalone.
  if (activePath === 'settings/messaging') {
    return <MessagingSetupBridge onClose={close} />;
  }

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4"
      onClick={close}
      role="dialog"
      aria-modal="true">
      <div
        className="w-full max-w-md rounded-2xl bg-white shadow-xl overflow-hidden"
        onClick={e => e.stopPropagation()}>
        <div className="flex items-center justify-between border-b border-stone-100 px-5 py-3">
          <h2 className="text-sm font-semibold text-stone-900">{titleForPath(activePath)}</h2>
          <button
            type="button"
            onClick={close}
            aria-label="Close"
            className="rounded p-1 text-stone-500 hover:bg-stone-100 hover:text-stone-800">
            <svg className="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 6l12 12M6 18L18 6" />
            </svg>
          </button>
        </div>
        <div className="p-5">{renderBody(activePath, close)}</div>
      </div>
    </div>
  );
};

/**
 * Resolves the Telegram channel definition and hands it to the shared
 * `ChannelSetupModal` (same component the Settings → Messaging panel
 * uses). When definitions are still loading we render a tiny placeholder
 * so the user gets feedback instead of a flashing screen.
 */
const MessagingSetupBridge = ({ onClose }: { onClose: () => void }) => {
  const { definitions, loading } = useChannelDefinitions();
  const telegram = useMemo(
    () => definitions.find(d => d.id === 'telegram') ?? null,
    [definitions]
  );

  if (loading && !telegram) {
    return (
      <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4">
        <div className="rounded-2xl bg-white px-6 py-4 text-sm text-stone-600 shadow-xl">
          Loading channel setup…
        </div>
      </div>
    );
  }

  if (!telegram) {
    return (
      <div
        className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4"
        onClick={onClose}>
        <div
          className="rounded-2xl bg-white p-6 text-sm text-stone-700 shadow-xl max-w-sm"
          onClick={e => e.stopPropagation()}>
          <p>Telegram channel definition isn't available right now. Try again from Settings → Messaging.</p>
          <div className="mt-3 flex justify-end">
            <button
              type="button"
              onClick={onClose}
              className="rounded-lg border border-stone-200 px-3 py-1.5 text-xs font-medium text-stone-700 hover:bg-stone-50">
              Close
            </button>
          </div>
        </div>
      </div>
    );
  }

  return <ChannelSetupModal definition={telegram} onClose={onClose} />;
};

function titleForPath(path: string): string {
  switch (path) {
    case 'settings/notifications':
      return 'Allow notifications';
    case 'settings/billing':
      return 'Billing & credits';
    case 'settings/messaging':
      return 'Connect a chat channel';
    case 'settings/connections':
      return 'Integrations';
    case 'community/discord':
      return 'Join the community';
    default:
      return 'Settings';
  }
}

function renderBody(path: string, close: () => void) {
  switch (path) {
    case 'settings/notifications':
      return <NotificationsBody close={close} />;
    case 'settings/billing':
      return <BillingBody close={close} />;
    case 'settings/messaging':
      // Routed via the dedicated `MessagingSetupBridge` above; this case
      // is kept to satisfy the path-completeness check but is unreachable
      // because the parent component returns the bridge before calling
      // `renderBody`.
      return null;
    case 'community/discord':
      return <DiscordBody close={close} />;
    default:
      return (
        <div className="space-y-3 text-sm text-stone-700">
          <p>This setting isn't ready in the popup yet. Open the full settings page when you're ready.</p>
          <DoneFooter close={close} />
        </div>
      );
  }
}

// ── Notifications ────────────────────────────────────────────────────────

const NotificationsBody = ({ close }: { close: () => void }) => {
  const [status, setStatus] = useState<'idle' | 'sending' | 'sent' | 'error'>('idle');
  const [error, setError] = useState<string | null>(null);

  const handleAllow = async () => {
    setStatus('sending');
    setError(null);
    try {
      // First send triggers the OS permission prompt on macOS / Windows.
      // Once granted the notification appears and subsequent calls
      // succeed silently.
      await showNativeNotification({
        title: 'OpenHuman is good to go',
        body: 'You will get pings here when something needs your attention.',
        tag: 'welcome-notification-test',
      });
      setStatus('sent');
    } catch (e) {
      setStatus('error');
      setError(e instanceof Error ? e.message : String(e));
    }
  };

  return (
    <div className="space-y-4 text-sm text-stone-700">
      <p>
        OpenHuman uses native notifications so it can ping you when something needs your attention,
        even when the chat window is hidden. Click below to send a test, your OS will ask for
        permission the first time.
      </p>
      <button
        type="button"
        onClick={() => void handleAllow()}
        disabled={status === 'sending'}
        className="w-full rounded-xl bg-primary-500 text-white text-sm font-medium py-2.5 hover:bg-primary-600 transition-colors disabled:opacity-60">
        {status === 'sending' ? 'Asking your OS…' : 'Send test notification'}
      </button>
      {status === 'sent' && (
        <p className="text-xs text-sage-700">
          Sent. If you saw a pop-up in the corner, you're all set. If your OS asked for permission,
          allow it and then tell the agent it's done.
        </p>
      )}
      {status === 'error' && (
        <p className="text-xs text-coral-600">Couldn't send: {error}</p>
      )}
      <DoneFooter close={close} />
    </div>
  );
};

// ── Billing ──────────────────────────────────────────────────────────────

const BillingBody = ({ close }: { close: () => void }) => {
  return (
    <div className="space-y-4 text-sm text-stone-700">
      <div className="rounded-xl border border-stone-200 bg-stone-50 p-4">
        <p className="text-xs uppercase tracking-wide text-stone-500">Trial credit</p>
        <p className="mt-1 text-2xl font-semibold text-stone-900">$1.00</p>
        <p className="mt-1 text-xs text-stone-500">
          More than enough to play around. Top up or pick a plan when you want real usage.
        </p>
      </div>
      <button
        type="button"
        onClick={() => {
          void openUrl('https://tinyhumans.ai/dashboard').catch(() => {});
        }}
        className="w-full rounded-xl bg-primary-500 text-white text-sm font-medium py-2.5 hover:bg-primary-600 transition-colors">
        Open dashboard in browser
      </button>
      <DoneFooter close={close} skipLabel="Stay on trial" />
    </div>
  );
};

// ── Discord ──────────────────────────────────────────────────────────────

const DISCORD_INVITE_URL = 'https://discord.tinyhumans.ai/';

const DiscordBody = ({ close }: { close: () => void }) => {
  return (
    <div className="space-y-4 text-sm text-stone-700">
      <p>
        Hop into our Discord and link your OpenHuman account. You'll get exclusive early access to
        features, free credits to play with, a great community to nerd out with, and yes, free
        merch.
      </p>
      <ul className="space-y-1.5 text-xs text-stone-600 pl-1">
        <li className="flex items-center gap-2">
          <span className="h-1.5 w-1.5 rounded-full bg-primary-400 flex-shrink-0" />
          Exclusive feature access
        </li>
        <li className="flex items-center gap-2">
          <span className="h-1.5 w-1.5 rounded-full bg-primary-400 flex-shrink-0" />
          Free credits for active members
        </li>
        <li className="flex items-center gap-2">
          <span className="h-1.5 w-1.5 rounded-full bg-primary-400 flex-shrink-0" />
          Community of builders and operators
        </li>
        <li className="flex items-center gap-2">
          <span className="h-1.5 w-1.5 rounded-full bg-primary-400 flex-shrink-0" />
          Free merch when you stick around
        </li>
      </ul>
      <button
        type="button"
        onClick={() => {
          void openUrl(DISCORD_INVITE_URL).catch(() => {});
        }}
        className="w-full rounded-xl bg-primary-500 text-white text-sm font-medium py-2.5 hover:bg-primary-600 transition-colors">
        Open Discord invite
      </button>
      <DoneFooter close={close} skipLabel="Maybe later" />
    </div>
  );
};

// ── Shared footer ────────────────────────────────────────────────────────

const DoneFooter = ({
  close,
  skipLabel = 'Skip for now',
}: {
  close: () => void;
  skipLabel?: string;
}) => (
  <div className="flex items-center justify-between gap-3 pt-1">
    <button
      type="button"
      onClick={close}
      className="text-xs font-medium text-stone-500 hover:text-stone-800">
      {skipLabel}
    </button>
    <button
      type="button"
      onClick={close}
      className="rounded-lg border border-stone-200 bg-white px-3 py-1.5 text-xs font-medium text-stone-700 hover:bg-stone-50">
      Done
    </button>
  </div>
);

export default OpenhumanLinkModal;
