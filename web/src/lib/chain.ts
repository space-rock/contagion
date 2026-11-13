import { FailoverRpcProvider, JsonRpcProvider, type SerializedReturnValue } from 'near-api-js';
import type { ChainEvent, Config, Host, Quote, Stats, StorageBalance } from '../types';

const PAGE = 200;
const PARALLEL = 4;

export type Chain = ReturnType<typeof makeChain>;

export const makeChain = (rpcUrls: string[], contractId: string) => {
  const provider = new FailoverRpcProvider(rpcUrls.map((url) => new JsonRpcProvider({ url }, { retries: 1, wait: 300, backoff: 1.5 })));

  const view = async <T>(method: string, args: Record<string, unknown> = {}): Promise<T> => {
    const result = await provider.callFunction<SerializedReturnValue>({ contractId, method, args });
    if (result === undefined) throw new Error(`${method} returned nothing`);
    return result as unknown as T;
  };

  const hosts = async (total: number): Promise<Host[]> => {
    const pages = Math.max(1, Math.ceil(total / PAGE));
    const slots: Host[][] = new Array(pages);
    let next = 0;
    const worker = async (): Promise<void> => {
      for (;;) {
        const page = next;
        next += 1;
        if (page >= pages) return;
        slots[page] = await view<Host[]>('get_hosts', { from: page * PAGE, limit: PAGE });
      }
    };
    await Promise.all(Array.from({ length: Math.min(PARALLEL, pages) }, worker));
    return slots.flat();
  };

  return {
    contractId,
    config: () => view<Config>('get_config'),
    stats: () => view<Stats>('get_stats'),
    host: (accountId: string) => view<Host | null>('get_host', { account_id: accountId }),
    quote: (carrier: string) => view<Quote>('quote_catch', { carrier }),
    storage: (accountId: string) => view<StorageBalance>('storage_balance_of', { account_id: accountId }),
    events: (from: number, limit: number) => view<ChainEvent[]>('get_events', { from, limit }),
    hosts,
  };
};
