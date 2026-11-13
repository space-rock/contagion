import { describe, expect, it } from 'vitest';
import type { Config, Host } from '../types';
import {
  classify,
  clockFraction,
  deriveExpired,
  eventPage,
  mintByGeneration,
  moodFor,
  pageWindow,
  rankFor,
  titleFor,
  topSpreaders,
  withExpiry,
} from './game';

const host = (id: string, extra: Partial<Host> = {}): Host => ({
  account_id: id,
  status: 'contagious',
  parent: 'zero.testnet',
  gen: 1,
  mint: '0',
  children: 0,
  fanout_left: 5,
  viral: '0',
  liquid: '0',
  points: 0,
  claimed: false,
  last_spread_ns: '0',
  deadline_ns: '1000',
  expired: false,
  ...extra,
});

const config = {
  base: '1000000000000000000000',
  decay_bps: 6000,
  fanout: 5,
  incubation_ns: '1000',
} as Config;

describe('expiry', () => {
  it('expires a carrier once chain time reaches the deadline', () => {
    expect(deriveExpired(host('a'), 999n, false)).toBe(false);
    expect(deriveExpired(host('a'), 1000n, false)).toBe(true);
  });

  it('expires every carrier when the season is over', () => {
    expect(deriveExpired(host('a'), 1n, true)).toBe(true);
  });

  it('never expires hosts that are not contagious', () => {
    expect(deriveExpired(host('a', { status: 'recovered' }), 99999n, true)).toBe(false);
    expect(deriveExpired(host('a', { status: 'immune' }), 99999n, false)).toBe(false);
  });

  it('keeps the server flag until chain time is known', () => {
    expect(deriveExpired(host('a', { expired: true }), null, false)).toBe(true);
    expect(deriveExpired(host('a'), null, false)).toBe(false);
  });

  it('reuses host objects whose flag did not change', () => {
    const fresh = host('a');
    const stale = host('b', { deadline_ns: '10' });
    const [a, b] = withExpiry([fresh, stale], 500n, false);
    expect(a).toBe(fresh);
    expect(b).not.toBe(stale);
    expect(b.expired).toBe(true);
  });
});

describe('classification', () => {
  it('maps hosts to map classes', () => {
    expect(classify(host('a'))).toBe('live');
    expect(classify(host('a', { expired: true }))).toBe('expired');
    expect(classify(host('a', { status: 'recovered' }))).toBe('recovered');
    expect(classify(host('a', { status: 'immune' }))).toBeNull();
    expect(classify(host('a', { status: 'uninfected' }))).toBeNull();
  });

  it('maps hosts to mascot moods', () => {
    expect(moodFor(null)).toBe('healthy');
    expect(moodFor(host('a'))).toBe('contagious');
    expect(moodFor(host('a', { expired: true }))).toBe('expired');
    expect(moodFor(host('a', { status: 'recovered' }))).toBe('recovered');
    expect(moodFor(host('a', { status: 'immune' }))).toBe('immune');
    expect(moodFor(host('a', { status: 'uninfected' }))).toBe('healthy');
  });
});

describe('ranks and titles', () => {
  it('starts as a rookie with no progress', () => {
    const { rank, next, progress } = rankFor(0);
    expect([rank.name, next?.name, progress]).toEqual(['Rookie', 'Spreader', 0]);
  });

  it('promotes exactly at the threshold', () => {
    expect(rankFor(7).rank.name).toBe('Rookie');
    expect(rankFor(8).rank.name).toBe('Spreader');
    expect(rankFor(24).rank.name).toBe('Super-spreader');
  });

  it('reports partial progress toward the next rank', () => {
    expect(rankFor(16).progress).toBeCloseTo(0.5);
  });

  it('caps at the top rank', () => {
    const { rank, next, progress } = rankFor(500);
    expect([rank.name, next, progress]).toEqual(['Legend', null, 1]);
  });

  it('titles hosts by what they are', () => {
    expect(titleFor(host('a', { parent: null }), config)).toBe('Patient zero');
    expect(titleFor(host('a', { children: 5 }), config)).toBe('Super-spreader');
    expect(titleFor(host('a', { expired: true }), config)).toBe('Overdue carrier');
    expect(titleFor(host('a'), config)).toBe('Carrier');
    expect(titleFor(host('a', { status: 'recovered' }), config)).toBe('Survivor');
    expect(titleFor(host('a', { status: 'recovered', claimed: true }), config)).toBe('Reserve claimer');
    expect(titleFor(host('a', { status: 'immune' }), config)).toBe('Immune');
    expect(titleFor(host('a', { status: 'uninfected' }), config)).toBe('Healthy');
  });
});

describe('leaderboard', () => {
  it('ranks by points, then children, then name, and skips zero points', () => {
    const hosts = [
      host('c', { points: 5, children: 1 }),
      host('b', { points: 9, children: 1 }),
      host('a', { points: 5, children: 3 }),
      host('d', { points: 5, children: 1 }),
      host('z', { points: 0 }),
    ];
    expect(topSpreaders(hosts, 10).map((h) => h.account_id)).toEqual(['b', 'a', 'c', 'd']);
    expect(topSpreaders(hosts, 2)).toHaveLength(2);
  });
});

describe('clock', () => {
  it('returns the remaining fraction of the incubation window', () => {
    expect(clockFraction(host('a'), config, 0n)).toBe(1);
    expect(clockFraction(host('a'), config, 500n)).toBe(0.5);
    expect(clockFraction(host('a'), config, 5000n)).toBe(0);
  });

  it('is null for hosts without a clock', () => {
    expect(clockFraction(host('a', { status: 'recovered' }), config, 0n)).toBeNull();
    expect(clockFraction(host('a'), config, null)).toBeNull();
  });
});

describe('event paging', () => {
  it('has one empty page with no events', () => {
    expect(eventPage(0, 1)).toEqual({ pages: 1, page: 1, from: 0, limit: 0 });
  });

  it('serves the newest events on page 1', () => {
    expect(eventPage(25, 1)).toEqual({ pages: 3, page: 1, from: 15, limit: 10 });
  });

  it('serves a short last page for the oldest events', () => {
    expect(eventPage(25, 3)).toEqual({ pages: 3, page: 3, from: 0, limit: 5 });
  });

  it('serves a full last page when the total divides evenly', () => {
    expect(eventPage(30, 3)).toEqual({ pages: 3, page: 3, from: 0, limit: 10 });
    expect(eventPage(30, 2)).toEqual({ pages: 3, page: 2, from: 10, limit: 10 });
  });

  it('clamps out of range and invalid pages', () => {
    expect(eventPage(25, 99).page).toBe(3);
    expect(eventPage(25, 0).page).toBe(1);
    expect(eventPage(25, -4).page).toBe(1);
    expect(eventPage(25, Number.NaN).page).toBe(1);
  });

  it('covers every event exactly once across pages', () => {
    const seen = new Set<number>();
    const total = 47;
    const { pages } = eventPage(total, 1);
    for (let page = 1; page <= pages; page += 1) {
      const { from, limit } = eventPage(total, page);
      for (let i = from; i < from + limit; i += 1) seen.add(i);
    }
    expect(seen.size).toBe(total);
  });

  it('windows page numbers around the current page', () => {
    expect(pageWindow(1, 10)).toEqual([1, 2, 3, 4, 5]);
    expect(pageWindow(5, 10)).toEqual([3, 4, 5, 6, 7]);
    expect(pageWindow(10, 10)).toEqual([6, 7, 8, 9, 10]);
    expect(pageWindow(1, 2)).toEqual([1, 2]);
  });
});

describe('mint curve', () => {
  it('decays by the configured share each generation', () => {
    const mints = mintByGeneration(config, 3);
    expect(mints[0]).toBeCloseTo(600);
    expect(mints[1]).toBeCloseTo(360);
    expect(mints[2]).toBeCloseTo(216);
  });
});
