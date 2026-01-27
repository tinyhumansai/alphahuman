import { useState } from 'react';
import { openUrl } from '../utils/openUrl';
import { TELEGRAM_BOT_USERNAME } from '../utils/config';
import ConnectionIndicator from '../components/ConnectionIndicator';

const Home = () => {
  const [userName] = useState('Cyrus');

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
    // TODO: Navigate to connections management page
    console.log('Manage connections');
  };

  const handleDeleteAllData = () => {
    // TODO: Show confirmation dialog and delete all data
    console.log('Delete all data');
  };

  const handleViewEncryptionKey = () => {
    // TODO: Show encryption key in a secure modal
    console.log('View encryption key');
  };

  const handleLogout = () => {
    // TODO: Implement logout functionality
    console.log('Logout');
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
              {/* Manage Connections */}
              <button
                onClick={handleManageConnections}
                className="w-full flex items-start justify-between p-3 bg-black/50 border-b border-stone-700 hover:bg-stone-800/30 transition-all duration-200 text-left first:rounded-t-3xl"
              >
                <div className="flex-1">
                  <div className="font-medium text-sm mb-1">Manage Connections</div>
                  <p className="opacity-70 text-xs">Add, remove, or update your connected accounts</p>
                </div>
                <svg className="w-5 h-5 opacity-60 flex-shrink-0 ml-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                </svg>
              </button>

              {/* View Encryption Key */}
              <button
                onClick={handleViewEncryptionKey}
                className="w-full flex items-start justify-between p-3 bg-black/50 border-b border-stone-700 hover:bg-stone-800/30 transition-all duration-200 text-left"
              >
                <div className="flex-1">
                  <div className="font-medium text-sm mb-1">View Encryption Key</div>
                  <p className="opacity-70 text-xs">Access your encryption key for backup purposes</p>
                </div>
                <svg className="w-5 h-5 opacity-60 flex-shrink-0 ml-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" />
                </svg>
              </button>

              {/* Delete All Data */}
              <button
                onClick={handleDeleteAllData}
                className="w-full flex items-start justify-between p-3 bg-black/50 border-b border-coral-500/30 hover:bg-stone-800/30 transition-all duration-200 text-left"
              >
                <div className="flex-1">
                  <div className="font-medium text-sm mb-1 text-coral-400">Delete All Data</div>
                  <p className="opacity-70 text-xs">Permanently delete all your data and reset your account</p>
                </div>
                <svg className="w-5 h-5 opacity-60 flex-shrink-0 ml-3 text-coral-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                </svg>
              </button>

              {/* Logout */}
              <button
                onClick={handleLogout}
                className="w-full flex items-start justify-between p-3 bg-black/50 hover:bg-stone-800/30 transition-all duration-200 text-left last:rounded-b-3xl"
              >
                <div className="flex-1">
                  <div className="font-medium text-sm mb-1 text-amber-400">Logout</div>
                  <p className="opacity-70 text-xs">Sign out of your account</p>
                </div>
                <svg className="w-5 h-5 opacity-60 flex-shrink-0 ml-3 text-amber-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1" />
                </svg>
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default Home;
