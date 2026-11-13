import { BookOpen, Gamepad2, Radar, ScrollText } from 'lucide-react';
import { Link, NavLink, Outlet, ScrollRestoration, useLocation } from 'react-router';
import { Mascot } from '../brand/Mascot';
import { Toasts } from '../components/Toasts';
import { WalletChip } from '../components/WalletChip';
import { Notice } from '../components/ui';
import { config } from '../config';
import { useGame } from '../state/game';
import { useClearToastsOnChange } from '../state/toasts';
import { useWallet } from '../state/wallet';

const NAV = [
  { to: '/', label: 'Play', short: 'Play', icon: Gamepad2, end: true },
  { to: '/outbreak', label: 'Outbreak', short: 'Map', icon: Radar, end: false },
  { to: '/activity', label: 'Activity', short: 'Feed', icon: ScrollText, end: false },
  { to: '/how-to-play', label: 'How to play', short: 'Guide', icon: BookOpen, end: false },
];

const NetworkDot = () => (
  <span className="network" title={`${config.network} network`}>
    <i aria-hidden="true" />
    <span className="network-label">{config.network}</span>
  </span>
);

export const Layout = () => {
  const wallet = useWallet();
  const game = useGame();
  const { pathname } = useLocation();
  useClearToastsOnChange(wallet.accountId);
  const current = (to: string, isActive: boolean): string => (isActive || (to === '/how-to-play' && pathname === '/faq') ? 'on' : '');

  const unreachable = game.error !== null && game.stats === null;

  return (
    <div className="app">
      <div className="aurora" aria-hidden="true" />
      <header className="topbar">
        <div className="wrap bar">
          <Link to="/" className="brand" aria-label="Contagion home">
            <Mascot mood="contagious" size={38} animate />
            <span className="wordmark">Contagion</span>
          </Link>
          <nav className="nav" aria-label="Main">
            {NAV.map(({ to, label, icon: Icon, end }) => (
              <NavLink key={to} to={to} end={end} title={label} aria-label={label} className={({ isActive }) => current(to, isActive)}>
                <Icon size={18} />
                <span className="nav-label">{label}</span>
              </NavLink>
            ))}
          </nav>
          <div className="bar-right">
            <NetworkDot />
            <WalletChip />
          </div>
        </div>
      </header>

      <main className="wrap main">
        {unreachable && <Notice tone="error">Cannot reach {game.contractId}: {game.error}</Notice>}
        <Outlet />
      </main>

      <footer className="foot">
        <div className="wrap foot-row">
          <span>
            <b>$FLU</b> lives at{' '}
            <a href={`${config.explorer}/address/${game.contractId}`} target="_blank" rel="noreferrer">
              {game.contractId}
            </a>
            {' · '}
            <Link to="/faq">FAQ</Link>
          </span>
          <span className="muted">Built at NEAR HACKATHON. Testnet only, no real value.</span>
        </div>
      </footer>

      <nav className="bottom-nav" aria-label="Main">
        {NAV.map(({ to, short, icon: Icon, end }) => (
          <NavLink key={to} to={to} end={end} className={({ isActive }) => current(to, isActive)}>
            <Icon size={22} />
            <span>{short}</span>
          </NavLink>
        ))}
      </nav>
      <Toasts />
      <ScrollRestoration />
    </div>
  );
};
