import { Trophy } from 'lucide-react';
import { Link } from 'react-router';
import { Mascot } from '../brand/Mascot';
import { hueShift } from '../lib/avatar';
import { truncateAccount } from '../lib/format';
import { moodFor, topSpreaders } from '../lib/game';
import { useGame } from '../state/game';
import { Card, SectionTitle } from './ui';

const MEDALS = ['gold', 'silver', 'bronze'];

export const Leaderboard = ({ count = 5, link = false }: { count?: number; link?: boolean }) => {
  const { hosts } = useGame();
  const rows = hosts ? topSpreaders(hosts, count) : [];
  return (
    <Card className="board">
      <SectionTitle icon={<Trophy size={20} />} title="Top spreaders" />
      <ol className="board-list">
        {hosts === null &&
          Array.from({ length: count }, (_, i) => (
            <li key={i} className="board-row">
              <span className="sk sk-avatar sm" />
              <span className="sk sk-line" />
            </li>
          ))}
        {hosts !== null && rows.length === 0 && (
          <li className="empty">
            <Mascot mood="healthy" size={56} animate />
            <span>No spreaders yet. Be the first to catch it.</span>
          </li>
        )}
        {rows.map((host, i) => (
          <li key={host.account_id} className="board-row">
            <span className={`medal ${MEDALS[i] ?? 'plain'}`}>{i + 1}</span>
            <Mascot mood={moodFor(host)} size={36} hue={hueShift(host.account_id)} />
            <Link className="board-name" to={`/outbreak?focus=${encodeURIComponent(host.account_id)}`}>
              {truncateAccount(host.account_id, 10, 10)}
            </Link>
            <span className="board-points">
              <b>{host.points}</b> pts
            </span>
            <span className="muted small">{host.children} caught</span>
          </li>
        ))}
      </ol>
      {link && (
        <Link className="more" to="/outbreak">
          See them on the map
        </Link>
      )}
    </Card>
  );
};
