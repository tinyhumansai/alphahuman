import { describe, expect, it } from 'vitest';

import {
  base64ToBytes,
  decryptIntegrationTokens,
  encryptIntegrationTokens,
  hexToBase64,
  hexToBytes,
} from './integrationTokensCrypto';

// 32-byte key expressed as 64 hex chars (all-zeros for test determinism).
const TEST_KEY_HEX = '0'.repeat(64);

// A minimal valid payload satisfying the expiresAt requirement.
const VALID_PAYLOAD = JSON.stringify({
  accessToken: 'acc_test',
  refreshToken: 'ref_test',
  expiresAt: '2030-01-01T00:00:00.000Z',
});

// ─── hexToBytes ────────────────────────────────────────────────────────────

describe('hexToBytes', () => {
  it('converts a simple hex string to bytes', () => {
    const bytes = hexToBytes('deadbeef');
    expect(bytes).toEqual(new Uint8Array([0xde, 0xad, 0xbe, 0xef]));
  });

  it('handles 0x prefix', () => {
    const bytes = hexToBytes('0xdeadbeef');
    expect(bytes).toEqual(new Uint8Array([0xde, 0xad, 0xbe, 0xef]));
  });

  it('handles uppercase hex', () => {
    const bytes = hexToBytes('DEADBEEF');
    expect(bytes).toEqual(new Uint8Array([0xde, 0xad, 0xbe, 0xef]));
  });

  it('returns empty Uint8Array for empty string', () => {
    expect(hexToBytes('')).toEqual(new Uint8Array());
  });

  it('throws for odd-length hex string', () => {
    expect(() => hexToBytes('abc')).toThrow(/even length/);
  });

  it('throws for non-hex characters', () => {
    expect(() => hexToBytes('zzzz')).toThrow(/\[0-9a-fA-F\]/);
  });

  it('converts a 32-byte (64-char) all-zeros key', () => {
    const bytes = hexToBytes(TEST_KEY_HEX);
    expect(bytes).toHaveLength(32);
    expect(bytes.every(b => b === 0)).toBe(true);
  });
});

// ─── hexToBase64 ───────────────────────────────────────────────────────────

describe('hexToBase64', () => {
  it('converts deadbeef to its base64 equivalent', () => {
    // deadbeef → Uint8Array [0xde,0xad,0xbe,0xef] → base64 "3q2+7w=="
    expect(hexToBase64('deadbeef')).toBe('3q2+7w==');
  });

  it('returns empty string for an empty hex string', () => {
    expect(hexToBase64('')).toBe('');
  });
});

// ─── base64ToBytes ─────────────────────────────────────────────────────────

describe('base64ToBytes', () => {
  it('decodes standard base64', () => {
    // "3q2+7w==" → [0xde, 0xad, 0xbe, 0xef]
    const bytes = base64ToBytes('3q2+7w==');
    expect(bytes).toEqual(new Uint8Array([0xde, 0xad, 0xbe, 0xef]));
  });

  it('handles URL-safe base64 (- and _)', () => {
    // URL-safe variant of "3q2+7w==" is "3q2-7w=="
    const bytesStd = base64ToBytes('3q2+7w==');
    const bytesUrl = base64ToBytes('3q2-7w==');
    expect(bytesUrl).toEqual(bytesStd);
  });

  it('handles missing padding (pad=2)', () => {
    // "3q2+7w" (no padding) should work
    const bytes = base64ToBytes('3q2+7w');
    expect(bytes).toEqual(new Uint8Array([0xde, 0xad, 0xbe, 0xef]));
  });

  it('handles missing padding (pad=3)', () => {
    // One-byte value: 0xFF → base64 "/w==" → without padding "/w"
    const bytes = base64ToBytes('/w');
    expect(bytes).toEqual(new Uint8Array([0xff]));
  });
});

// ─── encrypt / decrypt roundtrip ───────────────────────────────────────────

describe('encryptIntegrationTokens + decryptIntegrationTokens', () => {
  it('roundtrips a valid payload', async () => {
    const encrypted = await encryptIntegrationTokens(VALID_PAYLOAD, TEST_KEY_HEX);
    expect(typeof encrypted).toBe('string');
    expect(encrypted.length).toBeGreaterThan(0);

    const decrypted = await decryptIntegrationTokens(encrypted, TEST_KEY_HEX);
    expect(decrypted).toBe(VALID_PAYLOAD);
  });

  it('is deterministic (same plaintext + key → same ciphertext)', async () => {
    const enc1 = await encryptIntegrationTokens(VALID_PAYLOAD, TEST_KEY_HEX);
    const enc2 = await encryptIntegrationTokens(VALID_PAYLOAD, TEST_KEY_HEX);
    expect(enc1).toBe(enc2);
  });

  it('different keys produce different ciphertexts', async () => {
    const keyA = TEST_KEY_HEX;
    const keyB = 'f'.repeat(64);
    const encA = await encryptIntegrationTokens(VALID_PAYLOAD, keyA);
    const encB = await encryptIntegrationTokens(VALID_PAYLOAD, keyB);
    expect(encA).not.toBe(encB);
  });

  it('different plaintexts produce different ciphertexts', async () => {
    const payload2 = JSON.stringify({
      accessToken: 'other_acc',
      refreshToken: 'other_ref',
      expiresAt: '2031-06-15T12:00:00.000Z',
    });
    const enc1 = await encryptIntegrationTokens(VALID_PAYLOAD, TEST_KEY_HEX);
    const enc2 = await encryptIntegrationTokens(payload2, TEST_KEY_HEX);
    expect(enc1).not.toBe(enc2);
  });

  it('roundtrips a payload with special characters', async () => {
    const special = JSON.stringify({
      accessToken: 'tok_!@#$%^&*()_+',
      refreshToken: 'ref_<>?/"\'\\',
      expiresAt: '2030-12-31T23:59:59.999Z',
    });
    const encrypted = await encryptIntegrationTokens(special, TEST_KEY_HEX);
    const decrypted = await decryptIntegrationTokens(encrypted, TEST_KEY_HEX);
    expect(decrypted).toBe(special);
  });
});

// ─── encryptIntegrationTokens — failure modes ──────────────────────────────

describe('encryptIntegrationTokens — failure modes', () => {
  it('throws for a key shorter than 32 bytes', async () => {
    await expect(encryptIntegrationTokens(VALID_PAYLOAD, 'aabb')).rejects.toThrow(
      /32-byte AES-GCM key/
    );
  });

  it('throws for a key longer than 32 bytes', async () => {
    await expect(encryptIntegrationTokens(VALID_PAYLOAD, 'a'.repeat(66))).rejects.toThrow(
      /32-byte AES-GCM key/
    );
  });

  it('throws when plaintext is not JSON', async () => {
    await expect(encryptIntegrationTokens('not-json', TEST_KEY_HEX)).rejects.toThrow(/JSON/);
  });

  it('throws when JSON payload is missing expiresAt', async () => {
    const noExpiry = JSON.stringify({ accessToken: 'a', refreshToken: 'b' });
    await expect(encryptIntegrationTokens(noExpiry, TEST_KEY_HEX)).rejects.toThrow(/expiresAt/);
  });

  it('throws when expiresAt is empty string', async () => {
    const emptyExpiry = JSON.stringify({ accessToken: 'a', refreshToken: 'b', expiresAt: '   ' });
    await expect(encryptIntegrationTokens(emptyExpiry, TEST_KEY_HEX)).rejects.toThrow(/expiresAt/);
  });
});

// ─── decryptIntegrationTokens — failure modes ──────────────────────────────

describe('decryptIntegrationTokens — failure modes', () => {
  it('throws for a key shorter than 32 bytes', async () => {
    const encrypted = await encryptIntegrationTokens(VALID_PAYLOAD, TEST_KEY_HEX);
    await expect(decryptIntegrationTokens(encrypted, 'aabb')).rejects.toThrow(
      /32-byte AES-GCM key/
    );
  });

  it('throws for a payload that is too short', async () => {
    // base64 of 10 bytes — less than the required 32-byte minimum header.
    const tooShort = btoa(String.fromCharCode(...new Array(10).fill(0)));
    await expect(decryptIntegrationTokens(tooShort, TEST_KEY_HEX)).rejects.toThrow(/too short/);
  });

  it('throws when decrypting with the wrong key', async () => {
    const encrypted = await encryptIntegrationTokens(VALID_PAYLOAD, TEST_KEY_HEX);
    const wrongKey = 'f'.repeat(64);
    await expect(decryptIntegrationTokens(encrypted, wrongKey)).rejects.toThrow();
  });

  it('throws when ciphertext is corrupted', async () => {
    const encrypted = await encryptIntegrationTokens(VALID_PAYLOAD, TEST_KEY_HEX);
    // Flip the last character to corrupt the auth tag.
    const corrupted = encrypted.slice(0, -4) + 'ZZZZ';
    await expect(decryptIntegrationTokens(corrupted, TEST_KEY_HEX)).rejects.toThrow();
  });
});
