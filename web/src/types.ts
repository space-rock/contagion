export type Status = 'uninfected' | 'contagious' | 'recovered' | 'immune';

export type Config = {
  base: string;
  decay_bps: number;
  bonus_bps: number;
  fanout: number;
  incubation_ns: string;
  burn_expired_bps: number;
  burn_voluntary_bps: number;
  cull_cut_bps: number;
  vax_cost: string;
  herd_bps: number;
  min_hosts: number;
  season_max_ns: string;
  min_fee: string;
  fee_per_unit: string;
  reserve_bps: number;
};

export type Host = {
  account_id: string;
  status: Status;
  parent: string | null;
  gen: number;
  mint: string;
  children: number;
  fanout_left: number;
  viral: string;
  liquid: string;
  points: number;
  claimed: boolean;
  last_spread_ns: string;
  deadline_ns: string;
  expired: boolean;
};

export type Globals = {
  start_ns: string;
  supply: string;
  burned: string;
  fees_locked: string;
  infection_minted: string;
  reserve_paid: string;
  infected: number;
  immune: number;
  max_gen: number;
  total_points: number;
  ended_at: string | null;
  end_reason: 'herd' | 'time' | null;
};

export type Stats = {
  globals: Globals;
  now_ns: string;
  season_over: boolean;
  season_ends_ns: string;
  reserve_pool: string;
  vax_cost?: string;
  accounts: number;
  events: number;
  storage_usage: number;
  balance: string;
};

export type Quote = {
  can_catch: boolean;
  reason: string | null;
  gen: number;
  mint: string;
  fee: string;
};

export type EventBody =
  | { event: 'catch'; carrier: string; target: string; gen: number; mint: string; bonus: string; fee: string }
  | { event: 'recover'; account: string; burned: string; expired: boolean }
  | { event: 'cull'; account: string; caller: string; burned: string; cut: string }
  | { event: 'vaccinate'; account: string; by: string; cost: string }
  | { event: 'claim'; account: string; amount: string }
  | { event: 'season_end'; at_ns: string; reason: 'herd' | 'time' };

export type ChainEvent = { index: number; at_ns: string; event: EventBody };

export type StorageBalance = { total: string; available: string } | null;
