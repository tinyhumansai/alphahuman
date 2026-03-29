/**
 * Daemon Health Indicator
 *
 * Compact status indicator showing daemon health with a colored dot and optional label.
 * Can be clicked to show detailed health information.
 */
import type React from 'react';

import { formatRelativeTime, useDaemonHealth } from '../../hooks/useDaemonHealth';
import type { DaemonStatus } from '../../store/daemonSlice';

interface Props {
  userId?: string;
  size?: 'sm' | 'md' | 'lg';
  showLabel?: boolean;
  onClick?: () => void;
  className?: string;
}

const DaemonHealthIndicator: React.FC<Props> = ({
  userId,
  size = 'md',
  showLabel = false,
  onClick,
  className = '',
}) => {
  const daemonHealth = useDaemonHealth(userId);

  // Size configurations
  const sizeConfig = {
    sm: { dot: 'w-2 h-2', text: 'text-xs', container: 'gap-1.5' },
    md: { dot: 'w-3 h-3', text: 'text-sm', container: 'gap-2' },
    lg: { dot: 'w-4 h-4', text: 'text-base', container: 'gap-2.5' },
  };

  const config = sizeConfig[size];

  // Status color mapping
  const getStatusColor = (status: DaemonStatus): string => {
    switch (status) {
      case 'running':
        return 'bg-green-500';
      case 'starting':
        return 'bg-yellow-500';
      case 'error':
        return 'bg-red-500';
      case 'disconnected':
      default:
        return 'bg-gray-500';
    }
  };

  // Status text mapping
  const getStatusText = (status: DaemonStatus): string => {
    switch (status) {
      case 'running':
        return 'Running';
      case 'starting':
        return 'Starting';
      case 'error':
        return 'Error';
      case 'disconnected':
      default:
        return 'Disconnected';
    }
  };

  // Tooltip content
  const getTooltipContent = (): string => {
    const { status, componentCount, healthyComponentCount, errorComponentCount, lastUpdate } =
      daemonHealth;

    let tooltip = `Status: ${getStatusText(status)}`;

    if (componentCount > 0) {
      tooltip += `\nComponents: ${healthyComponentCount}/${componentCount} healthy`;
      if (errorComponentCount > 0) {
        tooltip += ` (${errorComponentCount} errors)`;
      }
    }

    if (lastUpdate) {
      tooltip += `\nLast update: ${formatRelativeTime(lastUpdate)}`;
    }

    return tooltip;
  };

  const statusColor = getStatusColor(daemonHealth.status);
  const statusText = getStatusText(daemonHealth.status);

  const containerClasses = `
    flex items-center ${config.container}
    ${onClick ? 'cursor-pointer hover:opacity-80 transition-opacity' : ''}
    ${className}
  `.trim();

  return (
    <div className={containerClasses} onClick={onClick} title={getTooltipContent()}>
      <div className={`${config.dot} rounded-full ${statusColor} flex-shrink-0`} />
      {showLabel && (
        <span className={`${config.text} text-gray-300 font-medium`}>{statusText}</span>
      )}
    </div>
  );
};

export default DaemonHealthIndicator;
