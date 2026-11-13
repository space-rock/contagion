import { describe, expect, it } from 'vitest';
import type { Config } from '../types';
import { buildFaq, countFaq, filterFaq } from './faq';

const config: Config = {
  base: '1000000000000000000000',
  decay_bps: 6000,
  bonus_bps: 3000,
  fanout: 5,
  incubation_ns: '1814400000000000',
  burn_expired_bps: 5000,
  burn_voluntary_bps: 1000,
  cull_cut_bps: 1000,
  vax_cost: '20000000000000000000',
  herd_bps: 3000,
  min_hosts: 1000,
  season_max_ns: '2592000000000000',
  min_fee: '10000000000000000000000',
  fee_per_unit: '50',
  reserve_bps: 2000,
};

const flat = (config: Config | null) => buildFaq(config).flatMap((group) => group.items);

describe('faq content', () => {
  it('covers every group with several questions', () => {
    const groups = buildFaq(config);
    expect(groups.length).toBeGreaterThanOrEqual(8);
    for (const group of groups) expect(group.items.length).toBeGreaterThanOrEqual(3);
    expect(countFaq(groups)).toBeGreaterThanOrEqual(40);
  });

  it('gives every question a unique id, a question and an answer', () => {
    const items = flat(config);
    expect(new Set(items.map((item) => item.id)).size).toBe(items.length);
    for (const item of items) {
      expect(item.q.trim().endsWith('?')).toBe(true);
      expect(item.a.trim().length).toBeGreaterThan(20);
    }
  });

  it('never prints placeholders, with or without a loaded config', () => {
    for (const items of [flat(config), flat(null)]) {
      for (const item of items) {
        expect(`${item.q} ${item.a}`).not.toMatch(/undefined|NaN|\[object|null/);
      }
    }
  });

  it('uses the live contract values in its answers', () => {
    const text = flat(config).map((item) => item.a).join('\n');
    expect(text).toContain('5 people');
    expect(text).toContain('21 days');
    expect(text).toContain('30 days');
    expect(text).toContain('10%');
    expect(text).toContain('50%');
    expect(text).toContain('30%');
    expect(text).toContain('20 FLU');
    expect(text).toContain('0.00005 NEAR per FLU');
    expect(text).toContain('0.01 NEAR');
  });

  it('follows a different config instead of hard-coded numbers', () => {
    const other = { ...config, fanout: 3, incubation_ns: '180000000000', burn_expired_bps: 7000 };
    const text = flat(other).map((item) => item.a).join('\n');
    expect(text).toContain('3 people');
    expect(text).toContain('3 minutes');
    expect(text).toContain('70%');
    expect(text).not.toContain('5 people');
  });

  it('keeps plain language: no standard names or decimals', () => {
    const text = flat(config).map((item) => `${item.q} ${item.a}`).join('\n');
    expect(text).not.toMatch(/NEP-\d+|decimals|yocto/i);
  });
});

describe('removed questions', () => {
  it('does not ask about audits or deployer keys', () => {
    const items = flat(config);
    expect(items.map((item) => item.id)).not.toContain('audit');
    expect(items.map((item) => item.id)).not.toContain('keys');
    const text = items.map((item) => `${item.q} ${item.a}`).join('\n').toLowerCase();
    expect(text).not.toMatch(/audit|deployer key/);
  });
});

describe('filterFaq', () => {
  const groups = buildFaq(config);

  it('returns everything for an empty query and the all group', () => {
    expect(countFaq(filterFaq(groups, '', 'all'))).toBe(countFaq(groups));
    expect(countFaq(filterFaq(groups, '   ', 'all'))).toBe(countFaq(groups));
  });

  it('finds questions by words in the question or the answer', () => {
    const hits = filterFaq(groups, 'plague', 'all').flatMap((group) => group.items.map((item) => item.id));
    expect(hits).toContain('plague-doctor');
    expect(filterFaq(groups, 'REFUND', 'all').flatMap((group) => group.items.map((item) => item.id))).toContain('fee-back');
  });

  it('requires every word to match', () => {
    const wide = countFaq(filterFaq(groups, 'season', 'all'));
    const narrow = countFaq(filterFaq(groups, 'season herd', 'all'));
    expect(narrow).toBeGreaterThan(0);
    expect(narrow).toBeLessThan(wide);
  });

  it('limits to one group and drops empty groups', () => {
    const only = filterFaq(groups, '', 'trust');
    expect(only.map((group) => group.id)).toEqual(['trust']);
    expect(filterFaq(groups, 'plague', 'trust')).toEqual([]);
  });

  it('returns nothing for gibberish', () => {
    expect(filterFaq(groups, 'zzzzqqqq', 'all')).toEqual([]);
  });
});
