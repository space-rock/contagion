import { describe, expect, it } from 'vitest';
import { hashString, hueShift } from './avatar';

describe('avatar', () => {
  it('hashes the same string to the same number', () => {
    expect(hashString('alice.testnet')).toBe(hashString('alice.testnet'));
    expect(hashString('alice.testnet')).not.toBe(hashString('bob.testnet'));
  });

  it('keeps the hue shift small so the brand stays recognisable', () => {
    for (let i = 0; i < 500; i += 1) {
      const shift = hueShift(`account-${i}.testnet`);
      expect(shift).toBeGreaterThanOrEqual(-24);
      expect(shift).toBeLessThanOrEqual(24);
    }
  });

  it('gives different accounts different shifts', () => {
    const shifts = new Set(Array.from({ length: 200 }, (_, i) => hueShift(`a${i}.testnet`)));
    expect(shifts.size).toBeGreaterThan(20);
  });
});
