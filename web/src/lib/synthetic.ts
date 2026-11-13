import type { Host } from '../types';

const lcg = (seed: number) => {
  let state = seed >>> 0;
  return () => {
    state = (Math.imul(state, 1664525) + 1013904223) >>> 0;
    return state / 0x1_0000_0000;
  };
};

export const makeSyntheticHosts = (count: number, seed = 1, nowNs = 2_000_000_000_000_000_000n): Host[] => {
  const random = lcg(seed);
  const hosts: Host[] = [];
  const queue: number[] = [];
  const children = new Map<number, number>();
  const roots = Math.min(3, count);

  const push = (parent: number | null): void => {
    const index = hosts.length;
    const roll = random();
    const status = roll < 0.55 ? 'contagious' : 'recovered';
    const deadline = nowNs + BigInt(Math.floor((random() - 0.3) * 3.6e15));
    hosts.push({
      account_id: `h${index}.synthetic.testnet`,
      status,
      parent: parent === null ? null : hosts[parent].account_id,
      gen: parent === null ? 0 : hosts[parent].gen + 1,
      mint: '0',
      children: 0,
      fanout_left: 0,
      viral: String(Math.floor(random() * 500) * 10 ** 18),
      liquid: '0',
      points: Math.floor(random() * 40),
      claimed: false,
      last_spread_ns: '0',
      deadline_ns: deadline.toString(),
      expired: status === 'contagious' && deadline <= nowNs,
    });
    queue.push(index);
  };

  for (let i = 0; i < roots; i += 1) push(null);
  let cursor = 0;
  while (hosts.length < count && cursor < queue.length) {
    const parent = queue[cursor];
    const slots = 1 + Math.floor(random() * 5);
    for (let s = 0; s < slots && hosts.length < count; s += 1) {
      push(parent);
      children.set(parent, (children.get(parent) ?? 0) + 1);
    }
    cursor += 1;
  }
  children.forEach((value, parent) => {
    hosts[parent].children = value;
  });
  return hosts;
};
