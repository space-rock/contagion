import { RouterProvider } from 'react-router';
import { config } from '../config';
import { readParams } from '../lib/invite';
import { GameProvider } from '../state/game';
import { ToastProvider } from '../state/toasts';
import { WalletProvider } from '../state/wallet';
import { router } from './routes';

const contractId = readParams().get('c') ?? config.defaultContract;

export const App = () => (
  <ToastProvider>
    <WalletProvider contractId={contractId}>
      <GameProvider contractId={contractId}>
        <RouterProvider router={router} />
      </GameProvider>
    </WalletProvider>
  </ToastProvider>
);
