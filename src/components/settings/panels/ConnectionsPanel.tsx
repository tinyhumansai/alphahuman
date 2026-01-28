import { useState, useMemo } from 'react';
import { useAppSelector } from '../../../store/hooks';
import { selectIsAuthenticated } from '../../../store/telegramSelectors';
import { useSettingsNavigation } from '../hooks/useSettingsNavigation';
import SettingsHeader from '../components/SettingsHeader';
import TelegramConnectionModal from '../../TelegramConnectionModal';

import BinanceIcon from '../../../assets/icons/binance.svg';
import NotionIcon from '../../../assets/icons/notion.svg';
import TelegramIcon from '../../../assets/icons/telegram.svg';
import MetamaskIcon from '../../../assets/icons/metamask.svg';
import GoogleIcon from '../../../assets/icons/GoogleIcon';

interface ConnectOption {
  id: string;
  name: string;
  description: string;
  icon: React.ReactElement;
  comingSoon?: boolean;
}

// Reused from ConnectStep.tsx - helper to check saved session
const hasSavedSession = (): boolean => {
  try {
    return !!localStorage.getItem('telegram_session');
  } catch {
    return false;
  }
};

const ConnectionsPanel = () => {
  const { navigateBack } = useSettingsNavigation();
  const [isTelegramModalOpen, setIsTelegramModalOpen] = useState(false);

  // Redux state
  const isTelegramAuthenticated = useAppSelector(selectIsAuthenticated);
  const sessionString = useAppSelector((state) => state.telegram.sessionString);

  // Check if Telegram account is connected (authenticated or has saved session)
  const isTelegramConnected = useMemo(() => {
    return isTelegramAuthenticated || !!sessionString || hasSavedSession();
  }, [isTelegramAuthenticated, sessionString]);

  // Connection options - reused from ConnectStep.tsx
  const connectOptions: ConnectOption[] = [
    {
      id: 'telegram',
      name: 'Telegram',
      description: 'Organize chats, automate messages and get insights.',
      icon: <img src={TelegramIcon} alt="Telegram" className="w-5 h-5" />,
    },
    {
      id: 'google',
      name: 'Google',
      description: 'Manage emails, contacts and calendar events',
      icon: <GoogleIcon />,
      comingSoon: true,
    },
    {
      id: 'notion',
      name: 'Notion',
      description: 'Manage tasks, documents and everything else in your Notion',
      icon: <img src={NotionIcon} alt="Notion" className="w-5 h-5" />,
      comingSoon: true,
    },
    {
      id: 'wallet',
      name: 'Web3 Wallet',
      description: 'Trade the trenches in a safe and secure way.',
      icon: <img src={MetamaskIcon} alt="Metamask" className="w-5 h-5" />,
      comingSoon: true,
    },
    {
      id: 'exchange',
      name: 'Crypto Trading Exchanges',
      description: 'Connect and make trades with deep insights.',
      icon: <img src={BinanceIcon} alt="Binance" className="w-5 h-5" />,
      comingSoon: true,
    },
  ];

  // Check if an account is connected
  const isAccountConnected = (accountId: string): boolean => {
    if (accountId === 'telegram') {
      return isTelegramConnected;
    }
    // Add other account checks here when implemented
    return false;
  };

  const handleConnect = (provider: string) => {
    if (provider === 'telegram') {
      if (isTelegramConnected) {
        // TODO: Show disconnect confirmation
        console.log('Disconnect Telegram');
      } else {
        setIsTelegramModalOpen(true);
      }
      return;
    }

    if (connectOptions.find(opt => opt.id === provider)?.comingSoon) {
      console.log(`${provider} coming soon`);
      return;
    }

    console.log(`Connecting to ${provider}`);
  };

  const handleTelegramComplete = () => {
    setIsTelegramModalOpen(false);
  };

  return (
    <>
      <div className="overflow-hidden h-full flex flex-col">
        <SettingsHeader
          title="Connections"
          showBackButton={true}
          onBack={navigateBack}
        />

        <div className="flex-1 overflow-y-auto">
          <div className="p-4 space-y-6">
            {/* Connection Options */}
            <div>
              {connectOptions.map((option, index) => {
                const isConnected = isAccountConnected(option.id);
                return (
                  <button
                    key={option.id}
                    onClick={() => handleConnect(option.id)}
                    disabled={option.comingSoon}
                    className={`w-full flex items-center justify-between p-3 bg-black/50 ${
                      index === connectOptions.length - 1 ? '' : 'border-b border-stone-700'
                    } hover:bg-stone-800/30 transition-all duration-200 text-left ${
                      index === 0 ? 'first:rounded-t-3xl' : ''
                    } ${
                      index === connectOptions.length - 1 ? 'last:rounded-b-3xl' : ''
                    } focus:outline-none focus:ring-0 focus:border-inherit relative ${
                      option.comingSoon ? 'opacity-60 cursor-not-allowed' : ''
                    }`}
                  >
                    {/* Connection status dot - top right corner */}
                    {isConnected && !option.comingSoon && (
                      <div className="absolute -top-1 -right-1 w-3 h-3 bg-sage-500 rounded-full border-2 border-black/50"></div>
                    )}

                    <div className="w-5 h-5 opacity-60 flex-shrink-0 mr-3 text-white">
                      {option.icon}
                    </div>
                    <div className="flex-1">
                      <div className="font-medium text-sm mb-1 text-white">
                        {option.name}
                      </div>
                      <p className="opacity-70 text-xs">
                        {option.description}
                      </p>
                    </div>

                    <div className="flex items-center space-x-3">
                      {/* Coming Soon badge */}
                      {option.comingSoon && (
                        <span className="px-2 py-1 text-xs font-medium rounded-full border bg-stone-500/20 text-stone-400 border-stone-500/30">
                          Coming Soon
                        </span>
                      )}

                      {/* Disconnect button for connected services */}
                      {!option.comingSoon && isConnected && (
                        <span className="px-2 py-1 text-xs font-medium rounded-full border bg-red-500/20 text-red-400 border-red-500/30">
                          Disconnect
                        </span>
                      )}

                      {/* Connect button for non-connected services */}
                      {!option.comingSoon && !isConnected && (
                        <span className="px-2 py-1 text-xs font-medium rounded-full border bg-blue-500/20 text-blue-400 border-blue-500/30">
                          Connect
                        </span>
                      )}
                    </div>
                  </button>
                );
              })}
            </div>

            {/* Security notice */}
            <div className="p-4 bg-blue-500/10 border border-blue-500/20 rounded-xl">
              <div className="flex items-start space-x-2">
                <svg className="w-5 h-5 text-blue-400 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                <div>
                  <p className="font-medium text-blue-300 text-sm">🔒 Privacy & Security</p>
                  <p className="text-blue-200 text-xs mt-1">
                    All data and credentials are stored locally with zero-data retention policy.
                    Your information is encrypted and never shared with third parties.
                  </p>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Telegram Connection Modal */}
      <TelegramConnectionModal
        isOpen={isTelegramModalOpen}
        onClose={() => setIsTelegramModalOpen(false)}
        onComplete={handleTelegramComplete}
      />
    </>
  );
};

export default ConnectionsPanel;
