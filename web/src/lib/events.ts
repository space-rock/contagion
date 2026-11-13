import type { ChainEvent } from '../types';
import { near, tokens, truncateAccount } from './format';

export type EventKind = 'catch' | 'recover' | 'cull' | 'vaccinate' | 'claim' | 'season_end';

export const eventKind = (entry: ChainEvent): EventKind => entry.event.event;

export const describeEvent = (entry: ChainEvent): { who: string; text: string } => {
  const e = entry.event;
  switch (e.event) {
    case 'catch':
      return {
        who: truncateAccount(e.target),
        text: `caught it from ${truncateAccount(e.carrier)} as generation ${e.gen}, minting ${tokens(e.mint)} FLU for ${near(e.fee)} NEAR`,
      };
    case 'recover':
      return {
        who: truncateAccount(e.account),
        text: `${e.expired ? 'recovered late' : 'recovered in time'} and burned ${tokens(e.burned)} FLU`,
      };
    case 'cull':
      return {
        who: truncateAccount(e.caller),
        text: `culled ${truncateAccount(e.account)}, burning ${tokens(e.burned)} FLU and keeping ${tokens(e.cut)} FLU`,
      };
    case 'vaccinate':
      return {
        who: truncateAccount(e.by),
        text: `vaccinated ${truncateAccount(e.account)} for ${tokens(e.cost)} FLU`,
      };
    case 'claim':
      return { who: truncateAccount(e.account), text: `claimed ${tokens(e.amount)} FLU from the reserve` };
    case 'season_end':
      return {
        who: 'The season',
        text: e.reason === 'herd' ? 'ended by herd immunity' : 'ended when time ran out',
      };
  }
};

export const describeEventShort = (entry: ChainEvent): { who: string; text: string } => {
  const e = entry.event;
  switch (e.event) {
    case 'catch':
      return { who: truncateAccount(e.target), text: `caught it from ${truncateAccount(e.carrier)}` };
    case 'recover':
      return { who: truncateAccount(e.account), text: e.expired ? 'recovered late' : 'recovered' };
    case 'cull':
      return { who: truncateAccount(e.caller), text: `culled ${truncateAccount(e.account)}` };
    case 'vaccinate':
      return { who: truncateAccount(e.by), text: `vaccinated ${truncateAccount(e.account)}` };
    case 'claim':
      return { who: truncateAccount(e.account), text: `claimed ${tokens(e.amount, 0)} FLU` };
    case 'season_end':
      return { who: 'The season', text: 'ended' };
  }
};

export const eventTime = (entry: ChainEvent): Date => new Date(Number(BigInt(entry.at_ns) / 1_000_000n));
