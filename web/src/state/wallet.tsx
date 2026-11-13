import { NearConnector } from '@hot-labs/near-connect';
import { type ReactNode, createContext, useCallback, useContext, useEffect, useMemo, useRef, useState } from 'react';
import { config } from '../config';
import { type Call } from '../lib/calls';
import { buildActions } from '../lib/calls';
import { readParams } from '../lib/invite';

type Wallet = {
  accountId: string | null;
  ready: boolean;
  connect: () => Promise<void>;
  disconnect: () => Promise<void>;
  send: (calls: Call[]) => Promise<void>;
};

const WalletContext = createContext<Wallet | null>(null);

export const WalletProvider = ({ contractId, children }: { contractId: string; children: ReactNode }) => {
  const connector = useRef<NearConnector | null>(null);
  const preview = import.meta.env.DEV ? readParams().get('as') : null;
  const [accountId, setAccountId] = useState<string | null>(preview);
  const [ready, setReady] = useState(false);

  useEffect(() => {
    const instance = new NearConnector({
      network: config.network,
      features: { [config.network]: true },
      autoConnect: true,
    });
    connector.current = instance;
    const signedIn = (event: { accounts: { accountId: string }[] }) => setAccountId(event.accounts[0]?.accountId ?? null);
    const signedOut = () => setAccountId(preview);
    instance.on('wallet:signIn', signedIn);
    instance.on('wallet:signOut', signedOut);
    instance
      .getConnectedWallet()
      .then(({ accounts }) => setAccountId(preview ?? accounts[0]?.accountId ?? null))
      .catch(() => setAccountId(preview))
      .finally(() => setReady(true));
    return () => {
      instance.off('wallet:signIn', signedIn);
      instance.off('wallet:signOut', signedOut);
    };
  }, []);

  const connect = useCallback(async () => {
    await connector.current?.connect().catch(() => undefined);
  }, []);

  const disconnect = useCallback(async () => {
    await connector.current?.disconnect().catch(() => undefined);
    setAccountId(preview);
  }, []);

  const send = useCallback(
    async (calls: Call[]) => {
      if (preview) throw new Error('Preview mode: this view-as session cannot sign transactions.');
      const instance = connector.current;
      if (!instance) throw new Error('the wallet is not ready yet');
      const wallet = await instance.wallet();
      await wallet.signAndSendTransaction({ receiverId: contractId, actions: buildActions(calls) });
    },
    [contractId, preview],
  );

  const value = useMemo(() => ({ accountId, ready, connect, disconnect, send }), [accountId, ready, connect, disconnect, send]);
  return <WalletContext.Provider value={value}>{children}</WalletContext.Provider>;
};

export const useWallet = (): Wallet => {
  const value = useContext(WalletContext);
  if (!value) throw new Error('useWallet needs a WalletProvider');
  return value;
};
