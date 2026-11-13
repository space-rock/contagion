import { Skull } from 'lucide-react';
import { Mascot } from '../brand/Mascot';
import { STORAGE_MIN } from '../config';
import { hueShift } from '../lib/avatar';
import { bps, tokens, truncateAccount } from '../lib/format';
import { useActions, useMe } from '../state/actions';
import { useGame } from '../state/game';
import { useWallet } from '../state/wallet';
import { Button, Flu } from './ui';

export const Doctors = () => {
  const { hosts, config } = useGame();
  const wallet = useWallet();
  const { storage } = useMe();
  const { act, busy } = useActions();
  const registered = storage.data !== null && storage.data !== undefined;

  if (!hosts || !config) {
    return (
      <ul className="doctors">
        {Array.from({ length: 3 }, (_, i) => (
          <li key={i}>
            <span className="sk sk-avatar sm" />
            <span className="sk sk-line" />
          </li>
        ))}
      </ul>
    );
  }

  const expired = hosts
    .filter((host) => host.status === 'contagious' && host.expired)
    .sort((a, b) => (BigInt(b.viral) > BigInt(a.viral) ? 1 : -1))
    .slice(0, 12);

  if (expired.length === 0) {
    return (
      <div className="empty">
        <Mascot mood="recovered" size={64} animate />
        <span>No expired carriers right now. Everyone is healthy or still spreading.</span>
      </div>
    );
  }

  const cull = (target: string) =>
    void act(
      `cull:${target}`,
      registered
        ? [{ method: 'cull', args: { account: target } }]
        : [
            { method: 'storage_deposit', args: { registration_only: true }, deposit: STORAGE_MIN },
            { method: 'cull', args: { account: target } },
          ],
      `${truncateAccount(target)} was culled.`,
    );

  return (
    <>
      <ul className="doctors">
        {expired.map((host) => {
          const burn = (BigInt(host.viral) * BigInt(config.burn_expired_bps)) / 10_000n;
          const cut = host.account_id === wallet.accountId ? 0n : (burn * BigInt(config.cull_cut_bps)) / 10_000n;
          return (
            <li key={host.account_id}>
              <Mascot mood="expired" size={40} hue={hueShift(host.account_id)} />
              <div className="doctor-id">
                <strong>{truncateAccount(host.account_id, 10, 10)}</strong>
                <span className="muted small">{tokens(host.viral)} FLU locked</span>
              </div>
              <div className="doctor-cut">
                <span className="stat-label">Your cut</span>
                <Flu raw={cut} size={16} />
              </div>
              <Button
                tone="danger"
                size="sm"
                icon={<Skull size={16} />}
                busy={busy === `cull:${host.account_id}`}
                onClick={() => (wallet.accountId ? cull(host.account_id) : void wallet.connect())}
              >
                {wallet.accountId ? 'Cull' : 'Connect'}
              </Button>
            </li>
          );
        })}
      </ul>
      <p className="muted small">
        Culling burns {bps(config.burn_expired_bps)} of the carrier's locked FLU and pays you {bps(config.cull_cut_bps)} of that burn.
      </p>
    </>
  );
};
