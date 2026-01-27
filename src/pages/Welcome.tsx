import { useNavigate } from 'react-router-dom';
import TypewriterGreeting from '../components/TypewriterGreeting';

const Welcome = () => {
  const navigate = useNavigate();
  const greetings = [
    "Hello Satoshi! 👋",
    "Got Crypto, Anon? 👀",
    "Let's cook! 🔥",
    "Let's Ape Together! 👊",
    // "Welcome to the exclusive club of crypto degenerates! 🎪🚀",
    // "Let's get you richer than a Nigerian prince's email! 👑💸",
    // "Ready to HODL like your life depends on it? 🤝💀",
    // "Welcome, future crypto millionaire (results not guaranteed)! 🎰💎",
    // "Time to make Wall Street bros jealous AF! 📈🔥",
    // "Ready to go to the moon? Pack light! 🌙🚀"
  ];

  const handleTelegramLogin = () => {
    navigate('/onboarding/step1');
  };

  return (
    <div className="min-h-screen relative flex items-center justify-center">
      {/* Main content */}
      <div className="relative z-10 max-w-md w-full mx-4">
        {/* Welcome card */}
        <div className="glass rounded-3xl p-8 text-center animate-fade-up shadow-large">
          {/* Greeting */}
          <TypewriterGreeting greetings={greetings} />

          {/* <br /> */}

          <p className="opacity-70 mb-8 leading-relaxed">
            Welcome to AlphaHuman. Your Telegram assistant here to get you 10x more done in your crypto journey.
          </p>

          <p className="opacity-70 mb-8 leading-relaxed">
            Are you ready to cook?
          </p>

          {/* Login with Telegram button */}
          <button
            onClick={handleTelegramLogin}
            className="w-full flex items-center justify-center space-x-3 bg-blue-500 hover:bg-blue-600 active:bg-blue-700 text-white font-semibold py-4 rounded-xl transition-all duration-300 hover:shadow-medium hover:scale-[1.02] active:scale-[0.98]"
          >
            <svg className="w-6 h-6" viewBox="0 0 24 24" fill="currentColor">
              <path d="M11.944 0A12 12 0 0 0 0 12a12 12 0 0 0 12 12 12 12 0 0 0 12-12A12 12 0 0 0 12 0a12 12 0 0 0-.056 0zm4.962 7.224c.1-.002.321.023.465.14a.506.506 0 0 1 .171.325c.016.093.036.306.02.472-.18 1.898-.962 6.502-1.36 8.627-.168.9-.499 1.201-.82 1.23-.696.065-1.225-.46-1.9-.902-1.056-.693-1.653-1.124-2.678-1.8-1.185-.78-.417-1.21.258-1.91.177-.184 3.247-2.977 3.307-3.23.007-.032.014-.15-.056-.212s-.174-.041-.249-.024c-.106.024-1.793 1.14-5.061 3.345-.48.33-.913.49-1.302.48-.428-.008-1.252-.241-1.865-.44-.752-.245-1.349-.374-1.297-.789.027-.216.325-.437.893-.663 3.498-1.524 5.83-2.529 6.998-3.014 3.332-1.386 4.025-1.627 4.476-1.635z" />
            </svg>
            <span>Login with Telegram</span>
          </button>
        </div>

        {/* Bottom text */}
        <p className="text-center opacity-60 text-sm mt-6">
          Made with ❤️ by poor Web3 nerds
        </p>
      </div>
    </div>
  );
};

export default Welcome;
