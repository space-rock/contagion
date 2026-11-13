import { hierarchy, tree } from 'd3-hierarchy';
import type { Host } from '../types';

export type LayoutNode = {
  id: string;
  x: number;
  y: number;
  gen: number;
  zero: boolean;
  parent: number;
};

export type Layout = { nodes: LayoutNode[]; index: Map<string, number> };

type Entry = { host: Host | null; children: Entry[] };

export const isSpreader = (host: Host): boolean => host.status === 'contagious' || host.status === 'recovered';

export const spreaderCount = (hosts: Host[]): number => hosts.reduce((sum, host) => sum + (isSpreader(host) ? 1 : 0), 0);

export const layoutOutbreak = (hosts: Host[], radius: number): Layout => {
  const spread = hosts.filter(isSpreader);
  if (spread.length === 0) return { nodes: [], index: new Map() };

  const entries = new Map<string, Entry>(spread.map((host) => [host.account_id, { host, children: [] }]));
  const root: Entry = { host: null, children: [] };
  for (const host of spread) {
    const entry = entries.get(host.account_id)!;
    const parent = host.parent ? entries.get(host.parent) : undefined;
    (parent ?? root).children.push(entry);
  }

  const laid = tree<Entry>()
    .size([2 * Math.PI, radius])
    .separation((a, b) => (a.parent === b.parent ? 1 : 2) / Math.max(a.depth, 1))(hierarchy(root, (entry) => entry.children));

  const nodes: LayoutNode[] = [];
  const index = new Map<string, number>();
  for (const node of laid.descendants()) {
    if (!node.data.host) continue;
    const host = node.data.host;
    index.set(host.account_id, nodes.length);
    nodes.push({
      id: host.account_id,
      x: node.y * Math.sin(node.x),
      y: -node.y * Math.cos(node.x),
      gen: host.gen,
      zero: !host.parent || !entries.has(host.parent),
      parent: -1,
    });
  }
  for (const node of laid.descendants()) {
    const parentHost = node.parent?.data.host;
    if (!node.data.host || !parentHost) continue;
    nodes[index.get(node.data.host.account_id)!].parent = index.get(parentHost.account_id) ?? -1;
  }
  return { nodes, index };
};

export const averageChildren = (hosts: Host[]): number | null => {
  const settled = hosts.filter((host) => host.status === 'recovered');
  if (settled.length === 0) return null;
  return settled.reduce((sum, host) => sum + host.children, 0) / settled.length;
};
