/**
 * Lightweight base64 → playable HTMLAudio wrapper. We don't need WebAudio
 * graph here; the viseme scheduler reads `currentTime` directly.
 */
export interface PlaybackHandle {
  /** ms elapsed since audio started. Returns -1 after playback ends. */
  currentMs(): number;
  /** Total audio duration in ms; 0 if metadata could not be read. */
  durationMs: number;
  /** Stop playback and release the blob URL. Idempotent. */
  stop(): void;
  /** Resolves when the audio finishes naturally. Rejects if `stop()` is called. */
  ended: Promise<void>;
}

export async function playBase64Audio(
  base64: string,
  mime: string = 'audio/mpeg'
): Promise<PlaybackHandle> {
  const bytes = Uint8Array.from(atob(base64), c => c.charCodeAt(0));
  const blob = new Blob([bytes], { type: mime });
  const url = URL.createObjectURL(blob);
  const audio = new window.Audio(url);
  audio.preload = 'auto';

  let stopped = false;
  let endedNaturally = false;
  let resolveEnded!: () => void;
  let rejectEnded!: (err: Error) => void;
  const ended = new Promise<void>((res, rej) => {
    resolveEnded = res;
    rejectEnded = rej;
  });

  const cleanup = () => {
    URL.revokeObjectURL(url);
  };

  audio.addEventListener('ended', () => {
    endedNaturally = true;
    cleanup();
    resolveEnded();
  });
  audio.addEventListener('error', () => {
    cleanup();
    rejectEnded(new Error('audio playback error'));
  });

  // Wait for metadata so the procedural-viseme fallback in useHumanMascot can
  // distribute frames across the real audio duration. Bounded with a short
  // race so a missing `loadedmetadata` event never blocks playback start.
  if (audio.readyState < 1) {
    await new Promise<void>(res => {
      const done = () => res();
      audio.addEventListener('loadedmetadata', done, { once: true });
      audio.addEventListener('error', done, { once: true });
      window.setTimeout(done, 250);
    });
  }

  try {
    await audio.play();
  } catch (err) {
    cleanup();
    rejectEnded(err instanceof Error ? err : new Error(String(err)));
    throw err;
  }

  const durationMs = Number.isFinite(audio.duration) ? audio.duration * 1000 : 0;

  return {
    currentMs: () => (endedNaturally || stopped ? -1 : audio.currentTime * 1000),
    durationMs,
    stop: () => {
      if (stopped) return;
      stopped = true;
      audio.pause();
      cleanup();
      rejectEnded(new Error('stopped'));
    },
    ended,
  };
}
