import { describe, expect, it } from 'vitest';
import { makeSyntheticHosts } from './synthetic';
import { averageChildren, isSpreader, layoutOutbreak, spreaderCount } from './tree';
import type { Host } from '../types';

const host = (id: string, parent: string | null, extra: Partial<Host> = {}): Host => ({
  account_id: id,
  status: 'contagious',
  parent,
  gen: parent ? 1 : 0,
  mint: '0',
  children: 0,
  fanout_left: 0,
  viral: '0',
  liquid: '0',
  points: 0,
  claimed: false,
  last_spread_ns: '0',
  deadline_ns: '0',
  expired: false,
  ...extra,
});

describe('layoutOutbreak', () => {
  it('returns nothing without infected hosts', () => {
    expect(layoutOutbreak([host('a', null, { status: 'uninfected' })], 100).nodes).toEqual([]);
  });

  it('places every infected host once and points each to its parent', () => {
    const hosts = [host('zero', null), host('a', 'zero'), host('b', 'zero'), host('c', 'a'), host('imm', null, { status: 'immune' })];
    const { nodes, index } = layoutOutbreak(hosts, 200);
    expect(nodes.map((n) => n.id).sort()).toEqual(['a', 'b', 'c', 'zero']);
    expect(nodes[index.get('zero')!].parent).toBe(-1);
    expect(nodes[index.get('zero')!].zero).toBe(true);
    expect(nodes[nodes[index.get('c')!].parent].id).toBe('a');
    expect(nodes[index.get('c')!].zero).toBe(false);
  });

  it('keeps nodes inside the radius', () => {
    const hosts = [host('zero', null), ...Array.from({ length: 30 }, (_, i) => host(`n${i}`, 'zero'))];
    for (const node of layoutOutbreak(hosts, 150).nodes) {
      expect(Math.hypot(node.x, node.y)).toBeLessThanOrEqual(150.0001);
    }
  });

  it('treats a host whose parent is missing as a root', () => {
    const { nodes } = layoutOutbreak([host('orphan', 'ghost')], 100);
    expect(nodes).toHaveLength(1);
    expect(nodes[0].zero).toBe(true);
    expect(nodes[0].parent).toBe(-1);
  });

  it('lays out ten thousand accounts quickly enough to rebuild on growth', () => {
    const hosts = makeSyntheticHosts(10_000);
    const started = performance.now();
    const { nodes } = layoutOutbreak(hosts, 360);
    const elapsed = performance.now() - started;
    expect(nodes).toHaveLength(10_000);
    expect(elapsed).toBeLessThan(2_000);
  });
});

describe('counts', () => {
  it('counts only spreading hosts', () => {
    const hosts = [host('a', null), host('b', 'a', { status: 'recovered' }), host('c', 'a', { status: 'immune' }), host('d', null, { status: 'uninfected' })];
    expect(spreaderCount(hosts)).toBe(2);
    expect(hosts.map(isSpreader)).toEqual([true, true, false, false]);
  });

  it('averages children over recovered hosts only', () => {
    expect(averageChildren([host('a', null)])).toBeNull();
    const hosts = [
      host('a', null, { status: 'recovered', children: 3 }),
      host('b', null, { status: 'recovered', children: 1 }),
      host('c', null, { status: 'contagious', children: 5 }),
    ];
    expect(averageChildren(hosts)).toBe(2);
  });
});
