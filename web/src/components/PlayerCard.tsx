import { Check, Copy, HeartPulse, Share2, Syringe, Wallet, Zap } from 'lucide-react';
import { useState } from 'react';
import { useSearchParams } from 'react-router';
import { Mascot } from '../brand/Mascot';
import { STORAGE_MIN } from '../config';
import { usePoll } from '../hooks';
import { hueShift } from '../lib/avatar';
import { bps, duration, near, tokens, truncateAccount } from '../lib/format';
import { clockFraction, moodFor, rankFor, titleFor } from '../lib/game';
import { inviteLink, readParams, shareUrl } from '../lib/invite';
import { useActions, useMe } from '../state/actions';
import { useGame } from '../state/game';
import { useToasts } from '../state/toasts';
import { useWallet } from '../state/wallet';
import type { Config, Host, Quote } from '../types';
import { Confetti } from './Confetti';
import { Button, Card, Chip, ClockRing, Flu, Notice, Pips, ProgressBar } from './ui';

const InvitePreview = ({ from, quote, loading }: { from: string; quote: Quote | null; loading: boolean }) => (
  <div className="invite-preview">
    <Mascot mood="contagious" size={56} hue={hueShift(from)} />
    <div>
      <p className="invite-from">
        <b>{truncateAccount(from)}</b> invited you
      </p>
      {loading || !quote ? (
        <span className="sk sk-line" />
      ) : quote.can_catch ? (
        <div className="invite-chips">
          <Chip tone="brand">Generation {quote.gen}</Chip>
          <Chip tone="brand">
            <Flu raw={quote.mint} size={14} />
          </Chip>
          <Chip>{near(quote.fee)} NEAR fee</Chip>
        </div>
      ) : (
        <p className="warn-text">{quote.reason ?? 'This carrier cannot be caught from.'}</p>
      )}
    </div>
  </div>
);

const PlayerView = ({ host, config, celebrate, refresh }: { host: Host; config: Config; celebrate: boolean; refresh: () => Promise<void> }) => {
  const game = useGame();
  const toasts = useToasts();
  const { act, busy } = useActions();
  const [copied, setCopied] = useState(false);
  const [vaccinateOpen, setVaccinateOpen] = useState(false);
  const [target, setTarget] = useState('');
  const seasonOver = game.stats?.season_over ?? false;
  const mood = celebrate ? 'celebrating' : moodFor(host);
  const { rank, next, progress } = rankFor(host.points);
  const fraction = clockFraction(host, config, game.now);
  const remaining = game.now ? BigInt(host.deadline_ns) - game.now : null;
  const link = inviteLink(game.contractId, host.account_id);
  const overdue = host.status === 'contagious' && (host.expired || seasonOver);
  const live = host.status === 'contagious' && !overdue;
  const full = host.children >= config.fanout;
  const burn = (BigInt(host.viral) * BigInt(live ? config.burn_voluntary_bps : config.burn_expired_bps)) / 10_000n;
  const known = new Set((game.hosts ?? []).map((h) => h.account_id));

  const canRecover = host.status === 'contagious';
  const canClaim = seasonOver && host.points > 0 && !host.claimed;
  const canVaccinate = !seasonOver;
  const vaxCost = game.stats?.vax_cost ?? config.vax_cost;
  const affordable = BigInt(host.liquid) >= BigInt(vaxCost);

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(link);
    } catch {
      toasts.push('error', 'Copying was blocked. Select the link and copy it by hand.');
      return;
    }
    setCopied(true);
    toasts.push('info', 'Invite link copied');
    setTimeout(() => setCopied(false), 2000);
  };

  const vaccinate = () => {
    const account = target.trim();
    if (!account) return;
    void act(
      'vaccinate',
      [{ method: 'vaccinate', args: { account }, deposit: known.has(account) ? 0n : STORAGE_MIN }],
      `${truncateAccount(account)} is immune now.`,
      refresh,
    ).then((ok) => {
      if (ok) {
        setTarget('');
        setVaccinateOpen(false);
      }
    });
  };

  return (
    <>
      {celebrate && <Confetti />}
      <div className="player-head">
        <Mascot mood={mood} size={104} hue={hueShift(host.account_id)} animate className="player-mascot" />
        <div className="player-id">
          <h3>{truncateAccount(host.account_id, 9, 9)}</h3>
          <div className="player-chips">
            <Chip tone={host.status === 'immune' ? 'immune' : overdue ? 'hot' : host.status === 'recovered' ? 'cool' : 'brand'}>{titleFor(host, config)}</Chip>
            <Chip>Generation {host.gen}</Chip>
            <Chip icon={<Zap size={14} />}>{rank.name}</Chip>
          </div>
          <div className="xp">
            <ProgressBar value={progress} label="Spread points progress" tone="brand" />
            <span className="muted small">
              {host.points} points{next ? `, ${next.min - host.points} to ${next.name}` : ', top rank'}
            </span>
          </div>
        </div>
        {host.status === 'contagious' && (
          <ClockRing
            fraction={overdue ? 0 : fraction}
            label={overdue ? 'Overdue' : remaining ? duration(remaining) : '...'}
            sub="clock"
            tone={overdue ? 'hot' : undefined}
          />
        )}
      </div>

      <div className="player-stats">
        <div>
          <span className="stat-label">Locked</span>
          <Flu raw={host.viral} size={20} />
        </div>
        <div>
          <span className="stat-label">Spendable</span>
          <Flu raw={host.liquid} size={20} />
        </div>
        <div>
          <span className="stat-label">Caught so far</span>
          <span className="pips-row">
            <Pips filled={host.children} total={config.fanout} />
            <b>
              {host.children}/{config.fanout}
            </b>
          </span>
        </div>
        <div>
          <span className="stat-label">Caught from</span>
          <b>{host.parent ? truncateAccount(host.parent) : 'Patient zero'}</b>
        </div>
      </div>

      {host.status === 'contagious' &&
        (overdue ? (
          <Notice tone="warn">
            Your clock ran out. Anyone can cull you and burn {bps(config.burn_expired_bps)} of your locked FLU. Recovering burns the same but nobody earns a cut.
          </Notice>
        ) : full ? (
          <Notice tone="warn">
            All {config.fanout} of your slots are used, so your clock cannot restart. Recover before it ends and burn {bps(config.burn_voluntary_bps)} ({tokens(burn)}{' '}
            FLU). Waiting past the clock costs {bps(config.burn_expired_bps)}.
          </Notice>
        ) : (
          <Notice>
            Pass it on before the clock ends, or recover now and burn {bps(config.burn_voluntary_bps)} ({tokens(burn)} FLU). Waiting past the clock costs{' '}
            {bps(config.burn_expired_bps)}.
          </Notice>
        ))}

      {live && !full && (
        <section className="player-section">
          <h4>Invite friends</h4>
          <div className="share">
            <input readOnly value={link} onFocus={(event) => event.currentTarget.select()} aria-label="Your invite link" />
            <Button tone="soft" size="sm" icon={copied ? <Check size={16} /> : <Copy size={16} />} onClick={() => void copy()}>
              {copied ? 'Copied' : 'Copy'}
            </Button>
            <a className="btn btn-soft btn-sm" href={shareUrl(link)} target="_blank" rel="noreferrer">
              <Share2 size={16} />
              <span>Post on X</span>
            </a>
          </div>
        </section>
      )}

      <section className="player-section moves">
        <h4>Your moves</h4>
        <div className="moves-row">
          {canRecover && (
            <Button
              tone="danger"
              size="sm"
              icon={<HeartPulse size={16} />}
              busy={busy === 'recover'}
              onClick={() => void act('recover', [{ method: 'recover' }], 'Recovered. Your tokens are spendable now.', refresh)}
            >
              Recover, burn {tokens(burn)} FLU
            </Button>
          )}
          {canClaim && (
            <Button size="sm" busy={busy === 'claim'} onClick={() => void act('claim', [{ method: 'claim' }], 'Reserve claimed.', refresh)}>
              Claim your reserve share
            </Button>
          )}
          {host.claimed && <Chip tone="brand">Reserve claimed</Chip>}
          {canVaccinate && (
            <Button tone="ghost" size="sm" icon={<Syringe size={16} />} disabled={!affordable} onClick={() => setVaccinateOpen((value) => !value)}>
              Vaccinate someone
            </Button>
          )}
        </div>
        {canVaccinate && !affordable && <span className="muted small">Vaccinating needs {tokens(vaxCost)} spendable FLU.</span>}
        {canVaccinate && affordable && vaccinateOpen && (
          <form
            className="vaccinate-form"
            onSubmit={(event) => {
              event.preventDefault();
              vaccinate();
            }}
          >
            <input value={target} onChange={(event) => setTarget(event.target.value)} placeholder="friend.testnet" aria-label="Account to vaccinate" />
            <Button size="sm" busy={busy === 'vaccinate'} type="submit" disabled={!target.trim()}>
              Burn {tokens(vaxCost)} FLU
            </Button>
          </form>
        )}
      </section>
    </>
  );
};

export const PlayerCard = () => {
  const game = useGame();
  const wallet = useWallet();
  const { host: me } = useMe();
  const { act, busy } = useActions();
  const [params, setParams] = useSearchParams();
  const [celebrate, setCelebrate] = useState(false);
  const from = params.get('from') ?? readParams().get('from');
  const quote = usePoll(() => game.chain.quote(from!), `${game.contractId}:${from}`, 10_000, { enabled: from !== null, keepPrevious: true });
  const seasonOver = game.stats?.season_over ?? false;
  const config = game.config;
  const host = me.data;
  const infected = host !== null && host.status !== 'uninfected';
  const pickable = (game.hosts ?? []).filter((h) => h.status === 'contagious' && !h.expired && h.fanout_left > 0);

  const pick = () => {
    if (pickable.length === 0) return;
    const chosen = pickable[Math.floor(Math.random() * pickable.length)].account_id;
    setParams((previous) => {
      const next = new URLSearchParams(previous);
      next.set('from', chosen);
      return next;
    });
  };

  const catchIt = async () => {
    if (!from || !quote.data) return;
    const ok = await act('catch', [{ method: 'catch', args: { carrier: from }, deposit: BigInt(quote.data.fee) }], 'You caught it. Spread it before your clock runs out.', me.refresh);
    if (ok) {
      setCelebrate(true);
      setTimeout(() => setCelebrate(false), 3500);
    }
  };

  if (!wallet.accountId) {
    return (
      <Card glow className="player player-intro">
        <Mascot mood="healthy" size={88} animate />
        <h3>{from ? 'You were invited to catch $FLU' : 'Ready to catch $FLU?'}</h3>
        {from && <InvitePreview from={from} quote={quote.data} loading={quote.loading} />}
        <p className="muted">Connect a testnet wallet to catch $FLU.</p>
        <Button icon={<Wallet size={18} />} onClick={() => void wallet.connect()} disabled={!wallet.ready}>
          Connect wallet
        </Button>
      </Card>
    );
  }

  if (me.loading || !config) {
    return (
      <Card glow className="player">
        <div className="player-head">
          <span className="sk sk-avatar" />
          <div className="player-id">
            <span className="sk sk-line" />
            <span className="sk sk-line short" />
          </div>
        </div>
        <span className="sk sk-block" />
      </Card>
    );
  }

  return (
    <Card glow className="player">
      {infected && host ? (
        <PlayerView host={host} config={config} celebrate={celebrate} refresh={me.refresh} />
      ) : (
        <>
          <div className="player-head">
            <Mascot mood={host?.status === 'immune' ? 'immune' : 'healthy'} size={104} hue={hueShift(wallet.accountId)} animate className="player-mascot" />
            <div className="player-id">
              <h3>{truncateAccount(wallet.accountId, 9, 9)}</h3>
              <div className="player-chips">
                <Chip>{host?.status === 'immune' ? 'Immune' : 'Healthy'}</Chip>
              </div>
            </div>
          </div>
          {host && BigInt(host.liquid) > 0n && (
            <div className="player-stats">
              <div>
                <span className="stat-label">Spendable</span>
                <Flu raw={host.liquid} size={20} />
              </div>
            </div>
          )}
          {seasonOver ? (
            <Notice tone="warn">The season is over, so nothing more can be caught. Claims are open for carriers with spread points.</Notice>
          ) : (
            <>
              {from && <InvitePreview from={from} quote={quote.data} loading={quote.loading} />}
              {host?.status === 'immune' ? (
                <Notice>You are vaccinated, so you can never catch it.</Notice>
              ) : (
                <div className="player-actions">
                  {from && quote.data?.can_catch && (
                    <Button icon={<Zap size={18} />} busy={busy === 'catch'} onClick={() => void catchIt()}>
                      Catch it for {near(quote.data.fee)} NEAR
                    </Button>
                  )}
                  <Button tone={from && quote.data?.can_catch ? 'ghost' : 'primary'} onClick={pick} disabled={pickable.length === 0}>
                    Find a live carrier
                  </Button>
                </div>
              )}
            </>
          )}
        </>
      )}
    </Card>
  );
};
