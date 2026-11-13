import { useCallback, useState } from 'react';
import { usePoll } from '../hooks';
import { useGame } from './game';
import { useToasts } from './toasts';
import type { Call } from '../lib/calls';
import { useWallet } from './wallet';

export const useActions = () => {
  const wallet = useWallet();
  const toasts = useToasts();
  const game = useGame();
  const [busy, setBusy] = useState<string | null>(null);

  const act = useCallback(
    async (key: string, calls: Call[], done: string, after?: () => Promise<void>): Promise<boolean> => {
      setBusy(key);
      try {
        await wallet.send(calls);
        toasts.push('success', done);
        await Promise.all([game.refresh(), after?.()]);
        setTimeout(() => void Promise.all([game.refresh(), after?.()]), 4_000);
        return true;
      } catch (error) {
        toasts.push('error', error instanceof Error ? error.message : String(error));
        return false;
      } finally {
        setBusy(null);
      }
    },
    [wallet, toasts, game],
  );

  return { busy, act };
};

export const useMe = () => {
  const { chain, contractId } = useGame();
  const { accountId } = useWallet();
  const signedIn = accountId !== null;
  const host = usePoll(() => chain.host(accountId!), `${contractId}:${accountId}:host`, 10_000, {
    enabled: signedIn,
    keepPrevious: true,
  });
  const storage = usePoll(() => chain.storage(accountId!), `${contractId}:${accountId}:storage`, 30_000, {
    enabled: signedIn,
    keepPrevious: true,
  });
  return { accountId, host, storage };
};
