import { useNavigate } from 'react-router-dom';

import ConnectionIndicator from '../components/ConnectionIndicator';
import { useUser } from '../hooks/useUser';

const Home = () => {
  const { user } = useUser();
  const navigate = useNavigate();
  const userName = user?.firstName || 'User';

  // Open in-app chat.
  const handleStartCooking = async () => {
    navigate('/chat');
  };

  return (
    <div className="min-h-full flex flex-col items-center justify-center p-4">
      <div className="max-w-md w-full">
        {/* Main card */}
        <div className="bg-white rounded-2xl shadow-soft border border-stone-200 p-6 animate-fade-up">
          {/* Welcome title */}
          <h1 className="text-3xl font-bold text-stone-900 text-center mb-6">
            Welcome {userName}!
          </h1>

          {/* Connection status */}
          <div className="flex justify-center mb-3">
            <ConnectionIndicator />
          </div>

          {/* Description */}
          <p className="text-sm text-stone-500 text-center mb-6 leading-relaxed">
            Your agent is now connected. Keep the app running to keep the connection alive. You can
            start chatting with the button below.
          </p>

          {/* CTA button */}
          <button
            onClick={handleStartCooking}
            className="w-full py-3 bg-primary-500 hover:bg-primary-600 text-white font-medium rounded-xl transition-colors duration-200">
            Message OpenHuman
          </button>
        </div>

        {/* Next steps — compact directory of where to go next */}
        <div className="mt-3 bg-white rounded-2xl shadow-soft border border-stone-200 p-4">
          <div className="text-[11px] uppercase tracking-wide text-stone-400 mb-2">Next steps</div>
          <div className="divide-y divide-stone-100">
            <button
              onClick={() => navigate('/skills')}
              className="w-full flex items-center justify-between py-2.5 text-left hover:bg-stone-50 rounded-md px-2 -mx-2 transition-colors">
              <div>
                <div className="text-sm font-medium text-stone-900">Connect your services</div>
                <div className="text-xs text-stone-500">
                  Give your assistant access to Gmail, Calendar, and more.
                </div>
              </div>
              <svg
                className="w-4 h-4 text-stone-400"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24">
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M9 5l7 7-7 7"
                />
              </svg>
            </button>
            <button
              onClick={() => navigate('/rewards')}
              className="w-full flex items-center justify-between py-2.5 text-left hover:bg-stone-50 rounded-md px-2 -mx-2 transition-colors">
              <div>
                <div className="text-sm font-medium text-stone-900">Earn rewards</div>
                <div className="text-xs text-stone-500">
                  Unlock credits by using OpenHuman and completing milestones.
                </div>
              </div>
              <svg
                className="w-4 h-4 text-stone-400"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24">
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M9 5l7 7-7 7"
                />
              </svg>
            </button>
            <button
              onClick={() => navigate('/invites')}
              className="w-full flex items-center justify-between py-2.5 text-left hover:bg-stone-50 rounded-md px-2 -mx-2 transition-colors">
              <div>
                <div className="text-sm font-medium text-stone-900">Invite a friend</div>
                <div className="text-xs text-stone-500">
                  Share an invite — both of you get credits.
                </div>
              </div>
              <svg
                className="w-4 h-4 text-stone-400"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24">
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M9 5l7 7-7 7"
                />
              </svg>
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};

export default Home;
