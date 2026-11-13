import type { Mood } from '../brand/art';
import type { Config, Host } from '../types';

export type HostClass = 'live' | 'expired' | 'recovered';

export const deriveExpired = (host: Host, nowNs: bigint | null, seasonOver: boolean): boolean => {
  if (host.status !== 'contagious') return false;
  if (seasonOver) return true;
  if (nowNs === null) return host.expired;
  return nowNs >= BigInt(host.deadline_ns);
};

export const withExpiry = (hosts: Host[], nowNs: bigint | null, seasonOver: boolean): Host[] =>
  hosts.map((host) => {
    const expired = deriveExpired(host, nowNs, seasonOver);
    return expired === host.expired ? host : { ...host, expired };
  });

export const classify = (host: Host): HostClass | null => {
  if (host.status === 'contagious') return host.expired ? 'expired' : 'live';
  if (host.status === 'recovered') return 'recovered';
  return null;
};

export const moodFor = (host: Host | null): Mood => {
  if (!host) return 'healthy';
  switch (host.status) {
    case 'contagious':
      return host.expired ? 'expired' : 'contagious';
    case 'recovered':
      return 'recovered';
    case 'immune':
      return 'immune';
    case 'uninfected':
      return 'healthy';
  }
};

export type Rank = { min: number; name: string };

export const RANKS: Rank[] = [
  { min: 0, name: 'Rookie' },
  { min: 8, name: 'Spreader' },
  { min: 24, name: 'Super-spreader' },
  { min: 60, name: 'Legend' },
];

export const rankFor = (points: number): { rank: Rank; next: Rank | null; progress: number } => {
  let index = 0;
  RANKS.forEach((rank, i) => {
    if (points >= rank.min) index = i;
  });
  const rank = RANKS[index];
  const next = RANKS[index + 1] ?? null;
  const progress = next ? (points - rank.min) / (next.min - rank.min) : 1;
  return { rank, next, progress: Math.min(1, Math.max(0, progress)) };
};

export const titleFor = (host: Host, config: Config): string => {
  switch (host.status) {
    case 'immune':
      return 'Immune';
    case 'recovered':
      return host.claimed ? 'Reserve claimer' : 'Survivor';
    case 'contagious':
      if (host.parent === null) return 'Patient zero';
      if (host.children >= config.fanout) return 'Super-spreader';
      return host.expired ? 'Overdue carrier' : 'Carrier';
    case 'uninfected':
      return 'Healthy';
  }
};

export const topSpreaders = (hosts: Host[], count: number): Host[] =>
  hosts
    .filter((host) => host.points > 0)
    .sort((a, b) => b.points - a.points || b.children - a.children || a.account_id.localeCompare(b.account_id))
    .slice(0, count);

export const clockFraction = (host: Host, config: Config, nowNs: bigint | null): number | null => {
  if (host.status !== 'contagious' || nowNs === null) return null;
  const total = Number(BigInt(config.incubation_ns));
  const left = Number(BigInt(host.deadline_ns) - nowNs);
  return Math.min(1, Math.max(0, left / total));
};

export type EventPage = { pages: number; page: number; from: number; limit: number };

export const eventPage = (total: number, page: number, size = 10): EventPage => {
  const pages = Math.max(1, Math.ceil(total / size));
  const current = Math.min(pages, Math.max(1, Math.floor(page) || 1));
  const end = total - (current - 1) * size;
  const from = Math.max(0, end - size);
  return { pages, page: current, from, limit: Math.max(0, end - from) };
};

export const pageWindow = (page: number, pages: number, width = 5): number[] => {
  const size = Math.min(width, pages);
  const start = Math.min(Math.max(1, page - Math.floor(size / 2)), pages - size + 1);
  return Array.from({ length: size }, (_, i) => start + i);
};

export const mintByGeneration = (config: Config, generations: number): number[] => {
  const decay = config.decay_bps / 10_000;
  const base = Number(BigInt(config.base) / 10n ** 15n) / 1000;
  return Array.from({ length: generations }, (_, i) => base * Math.pow(decay, i + 1));
};
