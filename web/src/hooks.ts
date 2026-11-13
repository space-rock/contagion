import { useCallback, useEffect, useRef, useState } from 'react';

export type Polled<T> = { data: T | null; error: string | null; loading: boolean; refresh: () => Promise<void> };

type PollOptions = { enabled?: boolean; keepPrevious?: boolean };

export const usePoll = <T>(load: () => Promise<T>, key: string, intervalMs: number, options: PollOptions = {}): Polled<T> => {
  const { enabled = true, keepPrevious = false } = options;
  const [state, setState] = useState<{ data: T | null; error: string | null; loading: boolean }>({
    data: null,
    error: null,
    loading: enabled,
  });
  const loader = useRef(load);
  loader.current = load;
  const generation = useRef(0);

  const refresh = useCallback(async () => {
    const mine = generation.current;
    try {
      const data = await loader.current();
      if (mine === generation.current) setState({ data, error: null, loading: false });
    } catch (error) {
      if (mine === generation.current) {
        setState((previous) => ({
          ...previous,
          error: error instanceof Error ? error.message : String(error),
          loading: false,
        }));
      }
    }
  }, []);

  useEffect(() => {
    generation.current += 1;
    if (!enabled) {
      setState({ data: null, error: null, loading: false });
      return;
    }
    setState((previous) => (keepPrevious && previous.data !== null ? previous : { data: null, error: null, loading: true }));
    void refresh();
    const timer = setInterval(() => void refresh(), intervalMs);
    return () => clearInterval(timer);
  }, [key, intervalMs, enabled, keepPrevious, refresh]);

  return { ...state, refresh };
};

export const useChainNow = (chainNowNs: string | undefined, tickMs = 1000): bigint | null => {
  const anchor = useRef<{ chain: bigint; local: bigint } | null>(null);
  const [now, setNow] = useState<bigint | null>(null);

  useEffect(() => {
    if (!chainNowNs) return;
    anchor.current = { chain: BigInt(chainNowNs), local: BigInt(Date.now()) * 1_000_000n };
    const tick = () => {
      const a = anchor.current;
      if (a) setNow(a.chain + (BigInt(Date.now()) * 1_000_000n - a.local));
    };
    tick();
    const timer = setInterval(tick, tickMs);
    return () => clearInterval(timer);
  }, [chainNowNs, tickMs]);

  return now;
};
