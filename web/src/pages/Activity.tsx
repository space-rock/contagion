import { ScrollText } from 'lucide-react';
import { useSearchParams } from 'react-router';
import { EventList } from '../components/EventList';
import { Leaderboard } from '../components/Leaderboard';
import { Pager } from '../components/Pager';
import { Card, SectionTitle } from '../components/ui';
import { usePoll } from '../hooks';
import { eventPage } from '../lib/game';
import { useGame } from '../state/game';

const PAGE_SIZE = 10;

export const Activity = () => {
  const game = useGame();
  const [params, setParams] = useSearchParams();
  const total = game.stats?.events ?? 0;
  const { pages, page, from, limit } = eventPage(total, Number(params.get('page') ?? 1), PAGE_SIZE);
  const known = game.stats !== null;
  const events = usePoll(() => game.chain.events(from, limit), `${game.contractId}:${from}:${limit}`, 600_000, {
    enabled: known && limit > 0,
    keepPrevious: true,
  });

  const goTo = (next: number) =>
    setParams((previous) => {
      const copy = new URLSearchParams(previous);
      copy.set('page', String(next));
      return copy;
    });

  return (
    <div className="grid-activity">
      <Card>
        <SectionTitle
          icon={<ScrollText size={20} />}
          title="Activity"
          hint={known ? `${total} events, newest first.` : undefined}
        />
        <EventList events={limit === 0 ? [] : events.data} loading={!known || (limit > 0 && events.data === null)} rows={PAGE_SIZE} />
        <Pager page={page} pages={pages} onPage={goTo} />
      </Card>
      <Leaderboard count={10} link />
    </div>
  );
};
