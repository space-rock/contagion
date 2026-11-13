import { Check, ChevronDown, Copy, ExternalLink, LogOut, Wallet } from 'lucide-react';
import { useEffect, useRef, useState } from 'react';
import { Mascot } from '../brand/Mascot';
import { config } from '../config';
import { hueShift } from '../lib/avatar';
import { truncateAccount } from '../lib/format';
import { useToasts } from '../state/toasts';
import { useWallet } from '../state/wallet';
import { Button } from './ui';

export const WalletChip = () => {
  const wallet = useWallet();
  const toasts = useToasts();
  const [open, setOpen] = useState(false);
  const [copied, setCopied] = useState(false);
  const root = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const outside = (event: PointerEvent) => {
      if (!root.current?.contains(event.target as Node)) setOpen(false);
    };
    const escape = (event: KeyboardEvent) => {
      if (event.key === 'Escape') setOpen(false);
    };
    document.addEventListener('pointerdown', outside);
    document.addEventListener('keydown', escape);
    return () => {
      document.removeEventListener('pointerdown', outside);
      document.removeEventListener('keydown', escape);
    };
  }, [open]);

  if (!wallet.accountId) {
    return (
      <Button className="wallet-btn" icon={<Wallet size={18} />} onClick={() => void wallet.connect()} disabled={!wallet.ready} aria-label="Connect wallet">
        Connect
      </Button>
    );
  }

  const id = wallet.accountId;
  const copy = async () => {
    await navigator.clipboard.writeText(id);
    setCopied(true);
    toasts.push('info', 'Address copied');
    setTimeout(() => setCopied(false), 1500);
  };

  return (
    <div className="wallet" ref={root}>
      <button className="wallet-chip" onClick={() => setOpen((value) => !value)} aria-expanded={open} aria-haspopup="menu" aria-label="Account menu">
        <Mascot mood="healthy" size={28} hue={hueShift(id)} />
        <span className="wallet-name">{truncateAccount(id, 6, 9)}</span>
        <ChevronDown size={16} className={open ? 'flip' : ''} />
      </button>
      {open && (
        <div className="menu" role="menu">
          <div className="menu-head">
            <Mascot mood="healthy" size={40} hue={hueShift(id)} />
            <div>
              <strong>{truncateAccount(id, 10, 8)}</strong>
              <span className="muted small">{config.network}</span>
            </div>
          </div>
          <button role="menuitem" onClick={() => void copy()}>
            {copied ? <Check size={16} /> : <Copy size={16} />} Copy address
          </button>
          <a role="menuitem" href={`${config.explorer}/address/${id}`} target="_blank" rel="noreferrer">
            <ExternalLink size={16} /> View on explorer
          </a>
          <button
            role="menuitem"
            className="danger"
            onClick={() => {
              setOpen(false);
              void wallet.disconnect();
            }}
          >
            <LogOut size={16} /> Sign out
          </button>
        </div>
      )}
    </div>
  );
};
