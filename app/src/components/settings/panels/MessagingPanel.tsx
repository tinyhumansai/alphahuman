import { useEffect, useMemo, useState } from 'react';

import { resolvePreferredAuthModeForChannel } from '../../../lib/channels/routing';
import { channelConnectionsApi } from '../../../services/api/channelConnectionsApi';
import {
  completeBreakingMigration,
  disconnectChannelConnection,
  setChannelConnectionStatus,
  setDefaultMessagingChannel,
  upsertChannelConnection,
} from '../../../store/channelConnectionsSlice';
import { useAppDispatch, useAppSelector } from '../../../store/hooks';
import type {
  ChannelAuthMode,
  ChannelConnectionStatus,
  ChannelType,
} from '../../../types/channels';
import { openUrl } from '../../../utils/openUrl';
import SettingsHeader from '../components/SettingsHeader';
import { useSettingsNavigation } from '../hooks/useSettingsNavigation';

const CHANNELS: Array<{ id: ChannelType; label: string; description: string }> = [
  { id: 'telegram', label: 'Telegram', description: 'Community and direct messaging automations' },
  {
    id: 'discord',
    label: 'Discord',
    description: 'Server and DM workflows with bot + OAuth support',
  },
];

const AUTH_MODES: Array<{ id: ChannelAuthMode; label: string; help: string }> = [
  {
    id: 'managed_dm',
    label: 'OpenHuman Managed DM',
    help: 'Use OpenHuman bot as primary DM channel',
  },
  { id: 'oauth', label: 'OAuth Sign-in', help: 'Connect your account through provider OAuth' },
  { id: 'bot_token', label: 'Bot Token', help: 'Use your own bot token for this channel' },
  { id: 'api_key', label: 'API Key', help: 'Use your own API key for this channel' },
];

const STATUS_STYLES: Record<ChannelConnectionStatus, { label: string; className: string }> = {
  connected: { label: 'Connected', className: 'bg-sage-500/20 text-sage-300 border-sage-500/30' },
  connecting: {
    label: 'Connecting',
    className: 'bg-amber-500/20 text-amber-300 border-amber-500/30',
  },
  disconnected: {
    label: 'Disconnected',
    className: 'bg-stone-500/20 text-stone-300 border-stone-500/30',
  },
  error: { label: 'Error', className: 'bg-coral-500/20 text-coral-300 border-coral-500/30' },
};

const MessagingPanel = () => {
  const { navigateBack } = useSettingsNavigation();
  const dispatch = useAppDispatch();
  const channelConnections = useAppSelector(state => state.channelConnections);

  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [busyKeys, setBusyKeys] = useState<Record<string, boolean>>({});
  const [tokenByKey, setTokenByKey] = useState<Record<string, string>>({});

  useEffect(() => {
    if (!channelConnections.migrationCompleted) {
      dispatch(completeBreakingMigration());
    }
  }, [channelConnections.migrationCompleted, dispatch]);

  useEffect(() => {
    let cancelled = false;

    const load = async () => {
      try {
        const data = await channelConnectionsApi.listConnections();
        if (cancelled) return;

        dispatch(setDefaultMessagingChannel(data.defaultMessagingChannel));
        for (const channel of CHANNELS) {
          const channelModes = data.connections[channel.id] ?? {};
          for (const authMode of AUTH_MODES) {
            const conn = channelModes[authMode.id];
            if (conn) {
              dispatch(
                upsertChannelConnection({ channel: channel.id, authMode: authMode.id, patch: conn })
              );
            }
          }
        }
      } catch (e) {
        const msg = e instanceof Error ? e.message : String(e);
        if (!cancelled) {
          setError(`Could not load channel connections: ${msg}`);
        }
      } finally {
        if (!cancelled) {
          setLoading(false);
        }
      }
    };

    void load();
    return () => {
      cancelled = true;
    };
  }, [dispatch]);

  const recommendedRoute = useMemo(() => {
    const channel = channelConnections.defaultMessagingChannel;
    const authMode = resolvePreferredAuthModeForChannel(channelConnections, channel);
    return authMode ? `${channel} via ${authMode}` : 'No active route';
  }, [channelConnections]);

  const runBusy = async (key: string, task: () => Promise<void>) => {
    setBusyKeys(prev => ({ ...prev, [key]: true }));
    setError(null);
    try {
      await task();
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setError(msg);
    } finally {
      setBusyKeys(prev => ({ ...prev, [key]: false }));
    }
  };

  const handleSetDefaultChannel = (channel: ChannelType) => {
    const key = `default:${channel}`;
    void runBusy(key, async () => {
      dispatch(setDefaultMessagingChannel(channel));
      await channelConnectionsApi.updatePreferences(channel);
    });
  };

  const handleConnect = (channel: ChannelType, authMode: ChannelAuthMode) => {
    const key = `${channel}:${authMode}`;
    void runBusy(key, async () => {
      dispatch(setChannelConnectionStatus({ channel, authMode, status: 'connecting' }));

      const tokenValue = tokenByKey[key]?.trim();
      const credentials =
        authMode === 'bot_token'
          ? { botToken: tokenValue }
          : authMode === 'api_key'
            ? { apiKey: tokenValue }
            : undefined;

      if ((authMode === 'bot_token' || authMode === 'api_key') && !tokenValue) {
        dispatch(
          setChannelConnectionStatus({
            channel,
            authMode,
            status: 'error',
            lastError: 'Credential is required for this mode',
          })
        );
        return;
      }

      const response = await channelConnectionsApi.connectChannel(channel, {
        authMode,
        credentials,
      });

      if (response.oauthUrl) {
        await openUrl(response.oauthUrl);
      }

      dispatch(
        upsertChannelConnection({
          channel,
          authMode,
          patch: response.connection ?? {
            status: 'connected',
            lastError: undefined,
            capabilities: authMode === 'managed_dm' ? ['dm'] : ['read', 'write'],
          },
        })
      );
    });
  };

  const handleDisconnect = (channel: ChannelType, authMode: ChannelAuthMode) => {
    const key = `${channel}:${authMode}`;
    void runBusy(key, async () => {
      await channelConnectionsApi.disconnectChannel(channel, authMode);
      dispatch(disconnectChannelConnection({ channel, authMode }));
    });
  };

  return (
    <div className="overflow-hidden h-full flex flex-col">
      <SettingsHeader title="Messaging" showBackButton={true} onBack={navigateBack} />

      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        <section className="rounded-xl border border-amber-500/30 bg-amber-500/10 p-4">
          <p className="text-sm font-semibold text-amber-200">Breaking change applied</p>
          <p className="text-xs text-amber-100/80 mt-1">
            Legacy Telegram/Discord connection state is reset in this version. Reconnect channels
            using one of the standardized auth modes below.
          </p>
        </section>

        <section className="rounded-xl border border-stone-800/60 bg-black/40 p-4 space-y-3">
          <h3 className="text-sm font-semibold text-white">Default Messaging Channel</h3>
          <div className="grid grid-cols-2 gap-2">
            {CHANNELS.map(channel => {
              const selected = channelConnections.defaultMessagingChannel === channel.id;
              const key = `default:${channel.id}`;
              return (
                <button
                  key={channel.id}
                  type="button"
                  onClick={() => handleSetDefaultChannel(channel.id)}
                  disabled={busyKeys[key]}
                  className={`rounded-lg border px-3 py-2 text-sm transition-colors ${
                    selected
                      ? 'border-primary-500/60 bg-primary-500/20 text-primary-200'
                      : 'border-stone-700 bg-stone-900/30 text-stone-300 hover:border-stone-500'
                  }`}>
                  {channel.label}
                </button>
              );
            })}
          </div>
          <p className="text-xs text-stone-400">
            Outbound priority: <span className="text-stone-300">managed_dm</span> first, then OAuth,
            bot token, API key. Active route:{' '}
            <span className="text-primary-300">{recommendedRoute}</span>
          </p>
        </section>

        {error && (
          <div className="rounded-lg border border-coral-500/40 bg-coral-500/10 px-4 py-3 text-sm text-coral-100">
            {error}
          </div>
        )}

        {loading && (
          <div className="rounded-xl border border-stone-800/60 bg-black/40 p-4 text-sm text-stone-400">
            Loading channel connections...
          </div>
        )}

        {!loading &&
          CHANNELS.map(channel => (
            <section
              key={channel.id}
              className="rounded-xl border border-stone-800/60 bg-black/40 p-4">
              <div className="mb-4">
                <h3 className="text-base font-semibold text-white">{channel.label}</h3>
                <p className="text-xs text-stone-400">{channel.description}</p>
              </div>

              <div className="space-y-3">
                {AUTH_MODES.map(mode => {
                  const key = `${channel.id}:${mode.id}`;
                  const connection = channelConnections.connections[channel.id][mode.id];
                  const status = connection?.status ?? 'disconnected';
                  const statusStyle = STATUS_STYLES[status];

                  return (
                    <div
                      key={mode.id}
                      className="rounded-lg border border-stone-800 bg-stone-900/20 p-3">
                      <div className="flex items-start justify-between gap-3">
                        <div>
                          <p className="text-sm font-medium text-white">{mode.label}</p>
                          <p className="text-xs text-stone-400 mt-1">{mode.help}</p>
                          {connection?.lastError && (
                            <p className="text-xs text-coral-300 mt-1">{connection.lastError}</p>
                          )}
                        </div>
                        <span
                          className={`px-2 py-1 text-[11px] border rounded-full ${statusStyle.className}`}>
                          {statusStyle.label}
                        </span>
                      </div>

                      {(mode.id === 'bot_token' || mode.id === 'api_key') && (
                        <input
                          type="password"
                          value={tokenByKey[key] ?? ''}
                          onChange={event =>
                            setTokenByKey(prev => ({ ...prev, [key]: event.target.value }))
                          }
                          placeholder={
                            mode.id === 'bot_token' ? 'Paste bot token' : 'Paste API key'
                          }
                          className="mt-3 w-full rounded-lg border border-stone-700 bg-stone-900 px-3 py-2 text-sm text-white placeholder:text-stone-500 focus:outline-none focus:border-primary-500/60"
                        />
                      )}

                      <div className="mt-3 flex gap-2">
                        <button
                          type="button"
                          disabled={busyKeys[key]}
                          onClick={() => handleConnect(channel.id, mode.id)}
                          className="rounded-lg bg-primary-500 px-3 py-1.5 text-xs font-medium text-white hover:bg-primary-600 disabled:opacity-50">
                          {status === 'connected' ? 'Reconnect' : 'Connect'}
                        </button>
                        <button
                          type="button"
                          disabled={busyKeys[key] || status === 'disconnected'}
                          onClick={() => handleDisconnect(channel.id, mode.id)}
                          className="rounded-lg border border-stone-700 px-3 py-1.5 text-xs font-medium text-stone-300 hover:border-stone-500 disabled:opacity-50">
                          Disconnect
                        </button>
                      </div>
                    </div>
                  );
                })}
              </div>
            </section>
          ))}
      </div>
    </div>
  );
};

export default MessagingPanel;
