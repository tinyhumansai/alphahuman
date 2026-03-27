import { useAppSelector } from '../store/hooks';

interface ConnectionIndicatorProps {
  status?: 'connected' | 'disconnected' | 'connecting';
  description?: string;
  className?: string;
}

const ConnectionIndicator = ({
  status: overrideStatus,
  description = 'Your device is now connected to the OpenHuman AI. Keep the app running to keep the connection alive. You can message your assistant with the button below.',
  className = '',
}: ConnectionIndicatorProps) => {
  const hasToken = useAppSelector(state => Boolean(state.auth.token));
  const status = overrideStatus || (hasToken ? 'connected' : 'disconnected');
  const statusConfig = {
    connected: {
      color: 'bg-sage-500',
      textColor: 'text-sage-500',
      text: 'Connected to OpenHuman AI 🚀',
    },
    disconnected: { color: 'bg-coral-500', textColor: 'text-coral-500', text: 'Disconnected' },
    connecting: { color: 'bg-amber-500', textColor: 'text-amber-500', text: 'Connecting' },
  };

  const config = statusConfig[status];

  return (
    <div className={`mb-6 ${className}`}>
      <div className="flex items-center justify-center space-x-2 mb-3">
        <div
          className={`w-2 h-2 ${config.color} rounded-full ${status === 'connected' ? 'animate-pulse' : ''}`}></div>
        <span className={`text-sm ${config.textColor}`}>{config.text}</span>
      </div>
      {description && (
        <p className="text-xs opacity-60 text-center leading-relaxed">{description}</p>
      )}
    </div>
  );
};

export default ConnectionIndicator;
