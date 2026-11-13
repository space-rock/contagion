import { CalendarClock, ShieldCheck, Users } from 'lucide-react';
import { duration } from '../lib/format';
import { averageChildren } from '../lib/tree';
import { useGame } from '../state/game';
import { Card, ProgressBar } from './ui';

export const StatStrip = () => {
  const { stats, hosts, config, now, loading } = useGame();
  const g = stats?.globals;
  const list = hosts ?? [];
  const live = list.filter((h) => h.status === 'contagious' && !h.expired).length;
  const expired = list.filter((h) => h.status === 'contagious' && h.expired).length;
  const recovered = list.filter((h) => h.status === 'recovered').length;
  const total = Math.max(1, live + expired + recovered);
  const average = averageChildren(list);

  const required = g && config ? Math.max(1, Math.ceil((config.herd_bps * g.infected) / 10_000)) : 1;
  const unlocked = g && config ? g.infected >= config.min_hosts : false;
  const herd = g && config ? (unlocked ? g.immune / required : g.infected / config.min_hosts) : 0;

  const over = stats?.season_over ?? false;
  const left = stats && now ? BigInt(stats.season_ends_ns) - now : null;
  const seasonTotal = config ? Number(BigInt(config.season_max_ns)) : null;
  const elapsed = over ? 1 : left !== null && seasonTotal ? Math.min(1, Math.max(0, 1 - Number(left) / seasonTotal)) : 0;

  return (
    <div className="stat-strip">
      <Card className="stat">
        <span className="stat-label">
          <Users size={16} /> Infected
        </span>
        {loading ? <span className="sk sk-value" /> : <strong className="stat-value">{g?.infected ?? 0}</strong>}
        <div className="split-bar" aria-hidden="true">
          <i className="s-live" style={{ flexGrow: live / total }} />
          <i className="s-expired" style={{ flexGrow: expired / total }} />
          <i className="s-recovered" style={{ flexGrow: recovered / total }} />
        </div>
        <span className="stat-sub">
          {live} live, {expired} expired, {recovered} recovered
          {average !== null && ` · ${average.toFixed(1)} caught per settled carrier`}
        </span>
      </Card>

      <Card className="stat">
        <span className="stat-label">
          <CalendarClock size={16} /> Season
        </span>
        {loading ? (
          <span className="sk sk-value" />
        ) : (
          <strong className="stat-value">{over ? 'Over' : left ? duration(left) : '...'}</strong>
        )}
        <ProgressBar value={elapsed} tone={over ? 'hot' : 'cool'} label="Season progress" loading={loading} />
        <span className="stat-sub">
          {over
            ? `Ended by ${g?.end_reason === 'herd' ? 'herd immunity' : 'the clock'}. Claims are open.`
            : 'Ends at herd immunity or when the clock runs out.'}
        </span>
      </Card>

      <Card className="stat">
        <span className="stat-label">
          <ShieldCheck size={16} /> Herd immunity
        </span>
        {loading ? <span className="sk sk-value" /> : <strong className="stat-value">{g?.immune ?? 0} <small>immune</small></strong>}
        <ProgressBar value={herd} tone="immune" label="Progress to herd immunity" loading={loading} />
        <span className="stat-sub">
          {unlocked ? `Needs ${required} immune to end the season` : `Counts once ${config?.min_hosts ?? '...'} are infected`}
        </span>
      </Card>
    </div>
  );
};
