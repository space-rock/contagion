import { Coins, Gift, Hourglass, Receipt, ShieldCheck, Skull, Syringe, TimerReset, Users } from 'lucide-react';
import type { ReactNode } from 'react';
import { Mascot } from '../brand/Mascot';
import type { Mood } from '../brand/art';
import { bps, duration, near, tokens } from '../lib/format';
import { mintByGeneration } from '../lib/game';
import type { Config } from '../types';
import { Card, Pips } from './ui';

export const Steps = ({ config }: { config: Config | null }) => {
  const steps: { mood: Mood; title: string; text: string }[] = [
    {
      mood: 'celebrating',
      title: 'Catch it',
      text: 'Open an invite link and pay a small fee. The contract mints your first FLU into a locked balance. The fee grows with what you mint, so every token costs the same.',
    },
    {
      mood: 'contagious',
      title: 'Spread it',
      text: config
        ? `Share your own link. Every catch earns you a ${bps(config.bonus_bps)} bonus and restarts your clock. You can pass it on to ${config.fanout} people.`
        : 'Share your own link. Every catch earns you a bonus and restarts your clock.',
    },
    {
      mood: 'expired',
      title: 'Settle up',
      text: config
        ? `Recover before your clock runs out and burn ${bps(config.burn_voluntary_bps)}. Miss it and anyone can cull you, burning ${bps(config.burn_expired_bps)}.`
        : 'Recover in time and burn a little. Miss the clock and anyone can cull you.',
    },
    {
      mood: 'immune',
      title: 'Vaccinate and win',
      text: 'Burn FLU to make accounts immune. Enough immune accounts end the season, and spread points share a reserve of new FLU.',
    },
  ];
  return (
    <ol className="steps">
      {steps.map((step, i) => (
        <li key={step.title} className="step card">
          <span className="step-number">{i + 1}</span>
          <Mascot mood={step.mood} size={92} animate />
          <h3>{step.title}</h3>
          <p>{step.text}</p>
        </li>
      ))}
    </ol>
  );
};

const RuleCard = ({ icon, label, value, detail, children }: { icon: ReactNode; label: string; value: string; detail: string; children?: ReactNode }) => (
  <Card className="rule">
    <span className="rule-icon">{icon}</span>
    <span className="stat-label">{label}</span>
    <strong className="rule-value">{value}</strong>
    <p className="muted small">{detail}</p>
    {children}
  </Card>
);

export const RuleCards = ({ config, loading }: { config: Config | null; loading: boolean }) => {
  if (loading || !config) {
    return (
      <div className="rules">
        {Array.from({ length: 6 }, (_, i) => (
          <Card key={i} className="rule">
            <span className="sk sk-line" />
            <span className="sk sk-value" />
          </Card>
        ))}
      </div>
    );
  }
  const mints = mintByGeneration(config, 8);
  const top = Math.max(...mints);
  return (
    <div className="rules">
      <Card className="rule rule-wide">
        <span className="rule-icon">
          <Coins size={20} />
        </span>
        <span className="stat-label">Mint per generation</span>
        <strong className="rule-value">{tokens((BigInt(config.base) * BigInt(config.decay_bps)) / 10_000n, 0)} FLU</strong>
        <p className="muted small">The first generation mints this much. Every generation after it gets {bps(config.decay_bps)} of the one before.</p>
        <div className="curve" aria-label="Mint by generation">
          {mints.map((mint, i) => (
            <div key={i} className="curve-bar">
              <i style={{ height: `${Math.max(6, (mint / top) * 100)}%` }} />
              <span>{Math.round(mint)}</span>
              <em>g{i + 1}</em>
            </div>
          ))}
        </div>
      </Card>
      <RuleCard icon={<Gift size={20} />} label="Carrier bonus" value={bps(config.bonus_bps)} detail="Extra FLU for the carrier on top of every mint they cause." />
      <RuleCard icon={<Users size={20} />} label="Fanout" value={`${config.fanout} people`} detail="How many accounts one carrier can pass it on to.">
        <Pips filled={config.fanout} total={config.fanout} />
      </RuleCard>
      <RuleCard icon={<Hourglass size={20} />} label="Incubation clock" value={duration(BigInt(config.incubation_ns))} detail="Restarts with every catch. Run it out and you can be culled." />
      <RuleCard icon={<TimerReset size={20} />} label="Recover in time" value={`burn ${bps(config.burn_voluntary_bps)}`} detail="Settle before the clock ends and keep the rest as spendable FLU." />
      <RuleCard icon={<Skull size={20} />} label="Late or culled" value={`burn ${bps(config.burn_expired_bps)}`} detail={`A plague doctor keeps ${bps(config.cull_cut_bps)} of that burn for calling cull.`} />
      <RuleCard icon={<Syringe size={20} />} label="Vaccine" value={`from ${tokens(config.vax_cost)} FLU`} detail="Burned to make one account immune for good. Each vaccination costs a little more than the last." />
      <RuleCard
        icon={<ShieldCheck size={20} />}
        label="Herd immunity"
        value={bps(config.herd_bps)}
        detail={`When this share of infected accounts is immune, from ${config.min_hosts} infected, the season ends.`}
      />
      <RuleCard icon={<Hourglass size={20} />} label="Season" value={duration(BigInt(config.season_max_ns))} detail="The longest a season can run if herd immunity never comes." />
      <RuleCard
        icon={<Receipt size={20} />}
        label="Catch fee"
        value={`${near(BigInt(config.fee_per_unit) * 10n ** 18n, 8)} NEAR per FLU`}
        detail={`Never less than ${near(config.min_fee)} NEAR, which pays the storage a new account needs. It stays in the contract.`}
      />
      <RuleCard icon={<Gift size={20} />} label="Reserve" value={bps(config.reserve_bps)} detail="Extra FLU, as a share of everything the outbreak minted, paid out by spread points after the season." />
    </div>
  );
};
