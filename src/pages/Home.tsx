import ConnectionIndicator from '../components/ConnectionIndicator';
import SkillsGrid from '../components/SkillsGrid';
import { useUser } from '../hooks/useUser';
import { TELEGRAM_BOT_USERNAME } from '../utils/config';
import { openUrl } from '../utils/openUrl';

const Home = () => {
  const { user } = useUser();
  const userName = user?.firstName || 'User';

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
    <div className="min-h-full relative">
      {/* Content overlay */}
      <div className="relative z-10 min-h-full flex flex-col">
        {/* Main content */}
        <div className="flex-1 flex items-center justify-center p-4">
          <div className="max-w-md w-full">
            {/* Weather card */}
            <div className="glass rounded-3xl p-4 shadow-large animate-fade-up text-center">
              {/* Greeting */}
              <h1 className="text-2xl font-bold mb-4">
                {getGreeting()}, {userName}
              </h1>

              {/* Connection indicators */}
              <ConnectionIndicator />

              {/* Get Access button */}
              <button
                onClick={handleStartCooking}
                className="btn-primary w-full py-2.5 text-sm font-medium rounded-xl">
                Message AlphaHuman 🔥
              </button>
            </div>

            {/* Skills Grid */}
            <SkillsGrid />
          </div>
        </div>
      </div>
    </div>
  );
};

export default Home;
