import {
  currentMonitor,
  getCurrentWindow,
  LogicalPosition,
  LogicalSize,
} from '@tauri-apps/api/window';
import { useEffect, useMemo, useRef, useState } from 'react';

import RotatingTetrahedronCanvas from '../components/RotatingTetrahedronCanvas';

const OVERLAY_IDLE_WIDTH = 50;
const OVERLAY_IDLE_HEIGHT = 50;
const OVERLAY_ACTIVE_WIDTH = 224;
const OVERLAY_ACTIVE_HEIGHT = 208;
const OVERLAY_IDLE_MARGIN = 10;
const OVERLAY_ACTIVE_MARGIN = 20;
const OVERLAY_IDLE_OPACITY = 0.6;
const SCENARIO_THREE_TEXT = '"Noted. Need milk."';

type OverlayStatus = 'idle' | 'active';
type OverlayScenario = 1 | 2 | 3;

interface OverlayBubble {
  id: string;
  text: string;
  tone: 'neutral' | 'accent' | 'success';
  compact?: boolean;
}

function bubbleToneClass(tone: OverlayBubble['tone']) {
  switch (tone) {
    case 'accent':
      return 'bg-blue-700 text-white';
    case 'success':
      return 'bg-emerald-500 text-emerald-950';
    default:
      return 'bg-slate-700 text-white';
  }
}

function OverlayBubbleChip({ bubble }: { bubble: OverlayBubble }) {
  const [displayedText, setDisplayedText] = useState('');
  const indexRef = useRef(0);

  useEffect(() => {
    if (!bubble.text) {
      return () => {
        indexRef.current = 0;
        setDisplayedText('');
      };
    }

    const timeoutId = window.setInterval(
      () => {
        indexRef.current += 1;
        setDisplayedText(bubble.text.slice(0, indexRef.current));
        if (indexRef.current >= bubble.text.length) {
          window.clearInterval(timeoutId);
        }
      },
      bubble.compact ? 28 : 32
    );

    return () => {
      window.clearInterval(timeoutId);
      indexRef.current = 0;
      setDisplayedText('');
    };
  }, [bubble.compact, bubble.id, bubble.text]);

  return (
    <div
      className={`max-w-[184px] rounded-[18px] px-3 py-2 text-right transition-all duration-200 ${bubbleToneClass(bubble.tone)} ${bubble.compact ? 'text-[12px] leading-[1.35]' : 'text-[13px] leading-[1.45]'}`}>
      {displayedText || ' '}
    </div>
  );
}

export default function OverlayApp() {
  const [scenario, setScenario] = useState<OverlayScenario>(1);
  const [isHovered, setIsHovered] = useState(false);

  useEffect(() => {
    const timeoutId = window.setTimeout(() => {
      setScenario(current => {
        if (current === 1) return 2;
        if (current === 2) return 3;
        return 1;
      });
    }, 5000);

    return () => {
      window.clearTimeout(timeoutId);
    };
  }, [scenario]);

  const status: OverlayStatus = scenario === 1 ? 'idle' : 'active';

  useEffect(() => {
    const appWindow = getCurrentWindow();
    const isActive = status === 'active';
    const width = isActive ? OVERLAY_ACTIVE_WIDTH : OVERLAY_IDLE_WIDTH;
    const height = isActive ? OVERLAY_ACTIVE_HEIGHT : OVERLAY_IDLE_HEIGHT;
    const margin = isActive ? OVERLAY_ACTIVE_MARGIN : OVERLAY_IDLE_MARGIN;
    const size = new LogicalSize(width, height);

    const updateWindowFrame = async () => {
      try {
        await appWindow.setSize(size);
      } catch (error) {
        console.warn('[overlay] failed to resize overlay window', error);
      }

      try {
        await appWindow.setMinSize(size);
      } catch (error) {
        console.warn('[overlay] failed to set overlay min size', error);
      }

      try {
        await appWindow.setMaxSize(size);
      } catch (error) {
        console.warn('[overlay] failed to set overlay max size', error);
      }

      try {
        const monitor = await currentMonitor();
        if (!monitor) {
          console.warn('[overlay] could not resolve current monitor for positioning');
          return;
        }

        const x = monitor.workArea.position.x + monitor.workArea.size.width - width - margin;
        const y = monitor.workArea.position.y + monitor.workArea.size.height - height - margin;
        await appWindow.setPosition(new LogicalPosition(x, y));
      } catch (error) {
        console.warn('[overlay] failed to pin overlay bottom-right after resize', error);
      }
    };

    void updateWindowFrame();
  }, [status]);

  const bubbles = useMemo<OverlayBubble[]>(() => {
    if (scenario === 1) {
      return [];
    }

    if (scenario === 2) {
      return [
        {
          id: 'assistant',
          text: '"Hey I think your coffee is getting cold. Want me to get you a new one?"',
          tone: 'accent',
        },
      ];
    }

    return [{ id: 'stt', text: SCENARIO_THREE_TEXT, tone: 'accent' }];
  }, [scenario]);

  const orbClassName = useMemo(() => {
    if (status === 'active') {
      return 'border-blue-950 bg-blue-700';
    }
    return 'border-slate-950 bg-slate-800';
  }, [status]);
  const tetrahedronInverted = status === 'active';
  const orbSizeClassName = status === 'active' ? 'h-[52px] w-[52px]' : 'h-[40px] w-[40px]';
  const orbCanvasClassName = status === 'active' ? 'h-[92%] w-[92%]' : 'h-[88%] w-[88%]';
  const orbStyle =
    status === 'idle' ? { opacity: isHovered ? 1 : OVERLAY_IDLE_OPACITY } : undefined;

  return (
    <div className="flex h-screen w-screen items-end justify-end bg-transparent px-0 py-0">
      <div
        className={`relative flex select-none flex-col items-end ${status === 'active' ? 'gap-3' : 'gap-0'}`}>
        <div
          className={`flex flex-col items-end gap-2 transition-all duration-200 ${status === 'active' ? 'max-w-[184px] opacity-100' : 'max-w-0 opacity-0'}`}>
          {bubbles.map(bubble => (
            <div key={bubble.id} className="animate-[overlay-bubble-in_220ms_ease-out]">
              <OverlayBubbleChip bubble={bubble} />
            </div>
          ))}
        </div>

        <div className="relative">
          <button
            type="button"
            aria-label="Activate overlay orb"
            onClick={() => {
              setScenario(2);
            }}
            onMouseEnter={() => {
              setIsHovered(true);
            }}
            onMouseLeave={() => {
              setIsHovered(false);
            }}
            className={`group relative flex cursor-pointer items-center justify-center overflow-hidden rounded-full border transition-all duration-200 ${orbClassName} ${orbSizeClassName}`}
            style={orbStyle}
            title="Click to start the demo.">
            <div
              className={`pointer-events-none opacity-95 transition-transform duration-300 group-hover:scale-105 ${orbCanvasClassName}`}>
              <RotatingTetrahedronCanvas inverted={tetrahedronInverted} />
            </div>
          </button>
        </div>
      </div>
    </div>
  );
}
