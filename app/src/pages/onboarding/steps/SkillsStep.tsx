import { useState } from 'react';

import OnboardingNextButton from '../components/OnboardingNextButton';

interface SkillsStepProps {
  onNext: (connectedSources: string[]) => void | Promise<void>;
  onBack?: () => void;
}

const SkillsStep = ({ onNext, onBack: _onBack }: SkillsStepProps) => {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleFinish = async () => {
    setError(null);
    setLoading(true);
    try {
      await onNext([]);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Something went wrong. Please try again.');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="rounded-2xl border border-stone-200 bg-white p-8 shadow-soft animate-fade-up">
      <div className="text-center mb-4">
        <h1 className="text-xl font-bold mb-2 text-stone-900">Connect Integrations Later</h1>
        <p className="text-stone-600 text-sm">
          OpenHuman no longer installs local QuickJS skills during onboarding. You can connect
          channels and Composio integrations later from the Integrations page once setup is
          complete.
        </p>
      </div>

      <div className="mb-4 rounded-2xl border border-stone-200 bg-stone-50 p-4 text-sm text-stone-600">
        Available after onboarding:
        <div className="mt-2 space-y-1 text-left text-stone-500">
          <div>Channels like Telegram and Discord</div>
          <div>Composio integrations like Gmail, Notion, and GitHub</div>
          <div>Built-in features like Voice, Screen Intelligence, and Autocomplete</div>
        </div>
      </div>

      {error && <p className="text-coral-400 text-sm mb-3 text-center">{error}</p>}

      <OnboardingNextButton onClick={handleFinish} loading={loading} loadingLabel="Loading..." />
    </div>
  );
};

export default SkillsStep;
