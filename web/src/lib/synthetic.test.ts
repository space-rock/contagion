import { describe, expect, it } from 'vitest';
import { makeSyntheticHosts } from './synthetic';

describe('makeSyntheticHosts', () => {
  it('is deterministic for a seed', () => {
    expect(makeSyntheticHosts(500, 7)).toEqual(makeSyntheticHosts(500, 7));
    expect(makeSyntheticHosts(500, 7)).not.toEqual(makeSyntheticHosts(500, 8));
  });

  it('builds a consistent forest', () => {
    const hosts = makeSyntheticHosts(1_000);
    expect(hosts).toHaveLength(1_000);
    const ids = new Set(hosts.map((h) => h.account_id));
    expect(ids.size).toBe(1_000);
    expect(hosts.filter((h) => h.parent === null)).toHaveLength(3);
    const childCount = new Map<string, number>();
    for (const h of hosts) {
      if (h.parent !== null) {
        expect(ids.has(h.parent)).toBe(true);
        childCount.set(h.parent, (childCount.get(h.parent) ?? 0) + 1);
      }
    }
    for (const h of hosts) expect(h.children).toBe(childCount.get(h.account_id) ?? 0);
  });
});
