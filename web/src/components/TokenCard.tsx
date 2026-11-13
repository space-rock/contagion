import { Check, Copy, ExternalLink } from 'lucide-react';
import { useState } from 'react';
import { Coin } from '../brand/Mascot';
import { config } from '../config';
import { tokens } from '../lib/format';
import { useGame } from '../state/game';
import { useToasts } from '../state/toasts';
import { Card } from './ui';

export const TokenCard = () => {
  const game = useGame();
  const toasts = useToasts();
  const [copied, setCopied] = useState(false);
  const loading = game.stats === null;

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(game.contractId);
    } catch {
      toasts.push('error', 'Copying was blocked. Select the address and copy it by hand.');
      return;
    }
    setCopied(true);
    toasts.push('info', 'Contract address copied');
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <Card className="token-card">
      <div className="token-coin">
        <Coin size={116} spin />
      </div>
      <div className="token-meta">
        <h3>
          <span className="grad-text">${config.symbol}</span>
        </h3>
        <dl>
          <div>
            <dt>Supply</dt>
            <dd>{loading ? <span className="sk sk-inline" /> : `${tokens(game.stats!.globals.supply, 0)} FLU`}</dd>
          </div>
          <div>
            <dt>Burned</dt>
            <dd>{loading ? <span className="sk sk-inline" /> : `${tokens(game.stats!.globals.burned, 0)} FLU`}</dd>
          </div>
        </dl>
        <div className="token-address">
          <code>{game.contractId}</code>
          <button className="icon-btn" onClick={() => void copy()} aria-label="Copy contract address">
            {copied ? <Check size={16} /> : <Copy size={16} />}
          </button>
          <a className="icon-btn" href={`${config.explorer}/address/${game.contractId}`} target="_blank" rel="noreferrer" aria-label="Open in explorer">
            <ExternalLink size={16} />
          </a>
        </div>
      </div>
    </Card>
  );
};
