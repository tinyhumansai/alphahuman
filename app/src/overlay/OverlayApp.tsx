import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window';
import { useEffect, useMemo, useState } from 'react';

import RotatingTetrahedronCanvas from '../components/RotatingTetrahedronCanvas';

const OVERLAY_WIDTH = 240;
const OVERLAY_HEIGHT = 220;

type OverlayStatus = 'idle' | 'active' | 'pulse';

interface OverlayBubble {
  id: string;
  text: string;
  tone: 'neutral' | 'accent' | 'success';
  compact?: boolean;
}

function bubbleToneClass(tone: OverlayBubble['tone']) {
  switch (tone) {
    case 'accent':
      return 'border-sky-700 bg-sky-500 text-sky-950';
    case 'success':
      return 'border-emerald-700 bg-emerald-500 text-emerald-950';
    default:
      return 'border-slate-800 bg-slate-700 text-white';
  }
}

function OverlayBubbleChip({ bubble }: { bubble: OverlayBubble }) {
  return (
    <div
      className={`max-w-[184px] rounded-[18px] border px-3 py-2 text-left transition-all duration-200 ${bubbleToneClass(bubble.tone)} ${bubble.compact ? 'text-[10px] leading-4' : 'text-[11px] leading-[1.35]'}`}>
      {bubble.text}
    </div>
  );
}

export default function OverlayApp() {
  const appWindow = getCurrentWindow();
  const [status, setStatus] = useState<OverlayStatus>('idle');
  const [tapCount, setTapCount] = useState(0);

  useEffect(() => {
    const size = new LogicalSize(OVERLAY_WIDTH, OVERLAY_HEIGHT);
    void appWindow.setSize(size).catch(error => {
      console.warn('[overlay] failed to resize overlay window', error);
    });
    void appWindow.setMinSize(size).catch(error => {
      console.warn('[overlay] failed to set overlay min size', error);
    });
    void appWindow.setMaxSize(size).catch(error => {
      console.warn('[overlay] failed to set overlay max size', error);
    });
  }, [appWindow]);

  const bubbles = useMemo<OverlayBubble[]>(() => {
    const items: OverlayBubble[] = [];

    if (status === 'active') {
      items.push({ id: 'status', text: 'Orb engaged.', tone: 'accent', compact: true });
    } else {
      items.push({ id: 'status', text: 'Orb idle.', tone: 'neutral', compact: true });
    }

    items.push({
      id: 'interaction',
      text: tapCount > 0 ? `Tapped ${tapCount} times.` : 'Click to animate.',
      tone: 'neutral',
      compact: true,
    });

    items.push({
      id: 'toggle',
      text: status === 'active' ? 'State: active' : 'State: inactive',
      tone: status === 'active' ? 'accent' : 'neutral',
      compact: true,
    });

    return items;
  }, [status, tapCount]);

  const orbClassName = useMemo(() => {
    if (status === 'active') {
      return 'border-sky-950 bg-sky-600';
    }
    return 'border-slate-950 bg-slate-800';
  }, [status]);
  const tetrahedronInverted = status === 'active';

  return (
    <div className="flex h-screen w-screen items-end justify-end bg-transparent px-0 py-0">
      <div className="relative flex select-none flex-col items-end gap-3">
        <div className="mr-1 flex max-w-[190px] flex-col items-end gap-2">
          {bubbles.map((bubble, index) => (
            <div
              key={bubble.id}
              className="animate-[overlay-bubble-in_220ms_ease-out]"
              style={{ marginRight: `${index * 8}px`, animationDelay: `${index * 40}ms` }}>
              <OverlayBubbleChip bubble={bubble} />
            </div>
          ))}
        </div>

        <div className="relative">
          <button
            type="button"
            aria-label="Activate overlay orb"
            onClick={() => {
              setTapCount(count => count + 1);
              setStatus(current => (current === 'idle' ? 'active' : 'idle'));
            }}
            className={`group relative flex h-[50px] w-[50px] items-center justify-center overflow-hidden rounded-full border transition-all duration-200 ${orbClassName}`}
            title="Click to toggle active state.">
            <div className="pointer-events-none h-full w-full opacity-95 transition-transform duration-300 group-hover:scale-105">
              <RotatingTetrahedronCanvas inverted={tetrahedronInverted} />
            </div>
          </button>
        </div>
      </div>
    </div>
  );
}
