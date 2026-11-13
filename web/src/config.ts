const list = (raw: string): string[] =>
  raw
    .split(',')
    .map((entry) => entry.trim())
    .filter(Boolean);

export const config = {
  network: 'testnet' as const,
  rpcUrls: list(import.meta.env.VITE_RPC_URLS ?? 'https://test.rpc.fastnear.com,https://rpc.testnet.near.org'),
  explorer: import.meta.env.VITE_EXPLORER ?? 'https://testnet.nearblocks.io',
  defaultContract: import.meta.env.VITE_CONTRACT_ID ?? 'contagion-live.testnet',
  siteName: 'Contagion',
  symbol: 'FLU',
};

export const STORAGE_MIN = 3_000_000_000_000_000_000_000n;
export const GAS_TGAS = 60;
