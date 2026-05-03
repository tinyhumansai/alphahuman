import { describe, expect, it, vi } from 'vitest';

import { callCoreRpc } from '../../../services/coreRpcClient';
import { synthesizeSpeech, visemesFromAlignment } from './ttsClient';

vi.mock('../../../services/coreRpcClient', () => ({ callCoreRpc: vi.fn() }));

describe('synthesizeSpeech (core RPC)', () => {
  it('routes through openhuman.voice_reply_synthesize and forwards options', async () => {
    const mock = callCoreRpc as ReturnType<typeof vi.fn>;
    mock.mockResolvedValueOnce({
      audio_base64: 'AAA=',
      audio_mime: 'audio/mpeg',
      visemes: [{ viseme: 'aa', start_ms: 0, end_ms: 100 }],
    });
    const r = await synthesizeSpeech('hello', { voiceId: 'v1', modelId: 'm1' });
    expect(mock).toHaveBeenCalledWith({
      method: 'openhuman.voice_reply_synthesize',
      params: { text: 'hello', voice_id: 'v1', model_id: 'm1' },
    });
    expect(r.audio_base64).toBe('AAA=');
    expect(r.visemes).toHaveLength(1);
  });

  it('omits options that were not provided', async () => {
    const mock = callCoreRpc as ReturnType<typeof vi.fn>;
    mock.mockResolvedValueOnce({ audio_base64: 'BBB=', audio_mime: 'audio/mpeg', visemes: [] });
    await synthesizeSpeech('hi');
    expect(mock).toHaveBeenCalledWith({
      method: 'openhuman.voice_reply_synthesize',
      params: { text: 'hi' },
    });
  });

  it('propagates RPC errors so the caller can degrade cleanly', async () => {
    const mock = callCoreRpc as ReturnType<typeof vi.fn>;
    mock.mockRejectedValueOnce(new Error('voice unavailable'));
    await expect(synthesizeSpeech('hi')).rejects.toThrow('voice unavailable');
  });
});

describe('visemesFromAlignment', () => {
  it('returns empty for empty input', () => {
    expect(visemesFromAlignment([])).toEqual([]);
  });

  it('buckets alignment chars into ~80ms windows', () => {
    const alignment = [
      { char: 'h', start_ms: 0, end_ms: 30 },
      { char: 'e', start_ms: 30, end_ms: 60 },
      { char: 'l', start_ms: 90, end_ms: 120 },
      { char: 'o', start_ms: 200, end_ms: 240 },
    ];
    const frames = visemesFromAlignment(alignment);
    expect(frames.length).toBeGreaterThan(0);
    const last = frames[frames.length - 1];
    expect(last.viseme).toBe('O');
  });

  it.each([
    ['a', 'aa'],
    ['e', 'E'],
    ['i', 'I'],
    ['y', 'I'],
    ['o', 'O'],
    ['u', 'U'],
    ['w', 'U'],
    ['m', 'PP'],
    ['b', 'PP'],
    ['p', 'PP'],
    ['f', 'FF'],
    ['v', 'FF'],
    ['s', 'SS'],
    ['z', 'SS'],
    ['r', 'RR'],
    ['n', 'nn'],
    ['l', 'DD'],
    ['d', 'DD'],
    ['t', 'DD'],
    ['k', 'kk'],
    ['g', 'kk'],
    ['h', 'CH'],
    ['c', 'CH'],
    ['j', 'CH'],
    ['x', 'sil'],
  ])('maps trailing letter %s in a window to %s', (ch, code) => {
    // Each char goes into its own 80ms+ window so the bucket flushes per char.
    const alignment = [
      { char: 'a', start_ms: 0, end_ms: 40 },
      { char: ch, start_ms: 100, end_ms: 140 },
    ];
    const frames = visemesFromAlignment(alignment);
    expect(frames[frames.length - 1].viseme).toBe(code);
  });
});
