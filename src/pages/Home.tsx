import { useNavigate } from 'react-router-dom';
import { openUrl } from '../utils/openUrl';
import { TELEGRAM_BOT_USERNAME } from '../utils/config';
import ConnectionIndicator from '../components/ConnectionIndicator';
import TelegramConnectionIndicator from '../components/TelegramConnectionIndicator';
import GmailConnectionIndicator from '../components/GmailConnectionIndicator';
import { useUser } from '../hooks/useUser';

const Home = () => {
  const navigate = useNavigate();
  const { user } = useUser();
  const userName = user?.firstName || 'User';

  // Get current date
  const getCurrentDate = () => {
    const days = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];
    const months = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];
    const now = new Date();
    return `${days[now.getDay()]}, ${months[now.getMonth()]} ${now.getDate()}`;
  };

  // Get greeting based on time
  const getGreeting = () => {
    const hour = new Date().getHours();
    if (hour < 12) return 'Good morning';
    if (hour < 18) return 'Good afternoon';
    return 'Good evening';
  };

  // Handle Telegram bot link
  const handleStartCooking = async () => {
    await openUrl(`https://t.me/${TELEGRAM_BOT_USERNAME}`);
  };

  const handleManageConnections = () => {
    navigate('/settings');
  };


  return (
    <div className="min-h-screen relative overflow-hidden">
      {/* Content overlay */}
      <div className="relative z-10 min-h-screen flex flex-col">
        {/* Main content */}
        <div className="flex-1 flex items-center justify-center p-4">
          <div className="max-w-md w-full">
            {/* Weather card */}
            <div className="glass rounded-3xl p-8 shadow-large animate-fade-up text-center">
              {/* Date */}
              <p className="text-sm mb-2 opacity-50 font-medium">
                {getCurrentDate()}
              </p>

              {/* Greeting */}
              <h1 className="text-2xl font-bold mb-4">
                {getGreeting()}, {userName}
              </h1>

              {/* Connection indicator */}
              <ConnectionIndicator />

              {/* Get Access button */}
              <button
                onClick={handleStartCooking}
                className="btn-primary w-full py-2.5 text-sm font-medium rounded-xl"
              >
                Message AlphaHuman 🔥
              </button>
            </div>

            {/* Action buttons */}
            <div className="glass rounded-3xl p-0 shadow-large animate-fade-up mt-4 overflow-hidden">
              {/* Settings */}
              <button
                onClick={handleManageConnections}
                className="w-full flex items-center justify-between p-3 bg-black/50 hover:bg-stone-800/30 transition-all duration-200 text-left rounded-3xl focus:outline-none"
              >
                <svg className="w-5 h-5 opacity-60 flex-shrink-0 mr-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                </svg>
                <div className="flex-1">
                  <div className="font-medium text-sm mb-1">Settings</div>
                  <p className="opacity-70 text-xs">Manage connections, privacy, profile, and app settings</p>
                </div>
              </button>
            </div>

            {/* Connection Indicators */}
            <TelegramConnectionIndicator className="mt-4" />
            <GmailConnectionIndicator className="mt-4" />
          </div>
        </div>
      </div>
    </div>
  );
};

export default Home;
