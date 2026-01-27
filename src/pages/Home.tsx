import { useState } from 'react';
import { openUrl } from '@tauri-apps/plugin-opener';
import { TELEGRAM_BOT_USERNAME } from '../utils/config';

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

              {/* <br /> */}


              {/* Get Access button */}
              <button
                onClick={handleStartCooking}
                className="btn-primary w-full py-4 text-lg font-semibold rounded-xl flex items-center justify-center space-x-2 hover:shadow-large transition-all duration-300 hover:scale-[1.02] active:scale-[0.98]"
              >
                <span>Start Cooking 🧑‍🍳</span>
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default Home;
