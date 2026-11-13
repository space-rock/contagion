import { type ReactNode, createContext, useCallback, useContext, useMemo } from 'react';
import { config } from '../config';
import { useChainNow, usePoll } from '../hooks';
import { type Chain, makeChain } from '../lib/chain';
import { withExpiry } from '../lib/game';
import type { ChainEvent, Config, Host, Stats } from '../types';

const RECENT = 8;

type Game = {
  contractId: string;
  chain: Chain;
  config: Config | null;
  stats: Stats | null;
  hosts: Host[] | null;
  recent: ChainEvent[] | null;
  now: bigint | null;
  loading: boolean;
  error: string | null;
  refresh: () => Promise<void>;
};

const GameContext = createContext<Game | null>(null);

export const GameProvider = ({ contractId, children }: { contractId: string; children: ReactNode }) => {
  const chain = useMemo(() => makeChain(config.rpcUrls, contractId), [contractId]);

  const rules = usePoll(() => chain.config(), contractId, 3_600_000);
  const stats = usePoll(() => chain.stats(), contractId, 6_000);
  const known = stats.data !== null;
  const changeKey = stats.data ? `${stats.data.events}:${stats.data.accounts}` : 'none';

  const rawHosts = usePoll(() => chain.hosts(stats.data!.accounts), `${contractId}:${changeKey}`, 300_000, {
    enabled: known,
    keepPrevious: true,
  });
  const recent = usePoll(
    async () => {
      const total = stats.data!.events;
      if (total === 0) return [];
      const from = Math.max(0, total - RECENT);
      return chain.events(from, total - from);
    },
    `${contractId}:${stats.data?.events ?? 'none'}`,
    300_000,
    { enabled: known, keepPrevious: true },
  );

  const now = useChainNow(stats.data?.now_ns, 5_000);
  const seasonOver = stats.data?.season_over ?? false;
  const hosts = useMemo(
    () => (rawHosts.data ? withExpiry(rawHosts.data, now, seasonOver) : null),
    [rawHosts.data, now, seasonOver],
  );

  const refresh = useCallback(async () => {
    await stats.refresh();
  }, [stats]);

  const value: Game = {
    contractId,
    chain,
    config: rules.data,
    stats: stats.data,
    hosts,
    recent: recent.data,
    now,
    loading: rules.loading || stats.loading || (rawHosts.loading && hosts === null),
    error: rules.error ?? stats.error ?? rawHosts.error,
    refresh,
  };

  return <GameContext.Provider value={value}>{children}</GameContext.Provider>;
};

export const useGame = (): Game => {
  const value = useContext(GameContext);
  if (!value) throw new Error('useGame needs a GameProvider');
  return value;
};
