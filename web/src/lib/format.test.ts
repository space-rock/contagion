import { describe, expect, it } from 'vitest';
import { bps, duration, longDuration, near, pct, timeAgo, tokens, truncateAccount } from './format';

describe('format', () => {
  it('formats token amounts with grouping and trimmed fractions', () => {
    expect(tokens('600000000000000000000')).toBe('600');
    expect(tokens('1234567890000000000000')).toBe('1,234.57');
    expect(tokens('1500000000000000000', 2)).toBe('1.5');
    expect(tokens('0')).toBe('0');
    expect(tokens(1_000_000n * 10n ** 18n, 0)).toBe('1,000,000');
  });

  it('rounds instead of truncating', () => {
    expect(tokens('1999000000000000000', 2)).toBe('2');
    expect(tokens('1994000000000000000', 2)).toBe('1.99');
  });

  it('formats NEAR amounts', () => {
    expect(near('30000000000000000000000')).toBe('0.03');
    expect(near('10000000000000000000000000')).toBe('10');
    expect(near('1234500000000000000000000', 3)).toBe('1.235');
  });

  it('formats durations at the right granularity', () => {
    const s = 1_000_000_000n;
    expect(duration(0n)).toBe('0s');
    expect(duration(-5n * s)).toBe('0s');
    expect(duration(45n * s)).toBe('45s');
    expect(duration(185n * s)).toBe('3m 5s');
    expect(duration(7_500n * s)).toBe('2h 5m');
    expect(duration(190_000n * s)).toBe('2d 4h');
  });

  it('truncates only long account ids, keeping both ends', () => {
    expect(truncateAccount('alice.testnet')).toBe('alice.testnet');
    expect(truncateAccount('b1.contagion-zero1.testnet')).toBe('b1.conta…o1.testnet');
    expect(truncateAccount('a'.repeat(64))).toBe(`${'a'.repeat(8)}…${'a'.repeat(10)}`);
    expect(truncateAccount('abcdefghijklmnopqrstuvwxyz', 4, 4)).toBe('abcd…wxyz');
  });

  it('describes elapsed time coarsely', () => {
    const now = new Date('2026-11-05T12:00:00Z');
    expect(timeAgo(new Date('2026-11-05T11:59:50Z'), now)).toBe('just now');
    expect(timeAgo(new Date('2026-11-05T11:50:00Z'), now)).toBe('10m ago');
    expect(timeAgo(new Date('2026-11-05T09:00:00Z'), now)).toBe('3h ago');
    expect(timeAgo(new Date('2026-11-02T12:00:00Z'), now)).toBe('3d ago');
    expect(timeAgo(new Date('2026-11-06T12:00:00Z'), now)).toBe('just now');
  });

  it('formats basis points and percentages', () => {
    expect(bps(5000)).toBe('50%');
    expect(bps(1250)).toBe('12.5%');
    expect(pct(1, 4)).toBe('25.0%');
    expect(pct(1, 0)).toBe('0%');
  });

  it('describes long durations in plain words', () => {
    const s = 1_000_000_000n;
    expect(longDuration(21n * 86_400n * s)).toBe('21 days');
    expect(longDuration(72n * 3_600n * s)).toBe('3 days');
    expect(longDuration(36n * 3_600n * s)).toBe('36 hours');
    expect(longDuration(86_400n * s)).toBe('24 hours');
    expect(longDuration(180n * s)).toBe('3 minutes');
    expect(longDuration(60n * s)).toBe('1 minute');
    expect(longDuration(30n * s)).toBe('30 seconds');
    expect(longDuration(0n)).toBe('0 seconds');
  });
});
