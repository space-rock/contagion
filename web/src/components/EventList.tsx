import { Flame, HeartPulse, Skull, Syringe, Trophy, Zap } from 'lucide-react';
import type { ReactNode } from 'react';
import { Link } from 'react-router';
import { Mascot } from '../brand/Mascot';
import { config } from '../config';
import { type EventKind, describeEvent, describeEventShort, eventKind, eventTime } from '../lib/events';
import { timeAgo } from '../lib/format';
import type { ChainEvent } from '../types';

const ICONS: Record<EventKind, ReactNode> = {
  catch: <Zap size={16} />,
  recover: <HeartPulse size={16} />,
  cull: <Skull size={16} />,
  vaccinate: <Syringe size={16} />,
  claim: <Trophy size={16} />,
  season_end: <Flame size={16} />,
};

const accountOf = (entry: ChainEvent): string | null => {
  const e = entry.event;
  switch (e.event) {
    case 'catch':
      return e.target;
    case 'recover':
    case 'cull':
    case 'vaccinate':
    case 'claim':
      return e.account;
    case 'season_end':
      return null;
  }
};

export const EventList = ({
  events,
  loading,
  rows = 5,
  empty = 'Nothing has happened yet.',
  compact = false,
}: {
  events: ChainEvent[] | null;
  loading: boolean;
  rows?: number;
  empty?: string;
  compact?: boolean;
}) => {
  if (loading || events === null) {
    return (
      <ul className={`events ${compact ? 'compact' : ''}`}>
        {Array.from({ length: rows }, (_, i) => (
          <li key={i} className={compact ? 'event-compact' : 'event'}>
            <span className="sk sk-avatar sm" />
            <span className="sk sk-line" />
          </li>
        ))}
      </ul>
    );
  }
  if (events.length === 0) {
    return (
      <div className="empty">
        <Mascot mood="healthy" size={64} animate />
        <span>{empty}</span>
      </div>
    );
  }
  if (compact) {
    return (
      <ul className="events compact">
        {[...events].reverse().slice(0, rows).map((entry) => {
          const kind = eventKind(entry);
          const { who, text } = describeEventShort(entry);
          return (
            <li key={entry.index} className="event-compact">
              <span className={`event-icon kind-${kind}`}>{ICONS[kind]}</span>
              <span className="event-line">
                <b>{who}</b> {text}
              </span>
              <time className="muted small" dateTime={eventTime(entry).toISOString()}>
                {timeAgo(eventTime(entry))}
              </time>
            </li>
          );
        })}
      </ul>
    );
  }
  return (
    <ul className="events">
      {[...events].reverse().map((entry) => {
        const kind = eventKind(entry);
        const { who, text } = describeEvent(entry);
        const account = accountOf(entry);
        return (
          <li key={entry.index} className="event">
            <span className={`event-icon kind-${kind}`}>{ICONS[kind]}</span>
            <div className="event-body">
              <p>
                <b>{who}</b> {text}
              </p>
              <span className="muted small">
                <time dateTime={eventTime(entry).toISOString()}>{timeAgo(eventTime(entry))}</time>
                {account && (
                  <>
                    {' · '}
                    <Link to={`/outbreak?focus=${encodeURIComponent(account)}`}>on the map</Link>
                    {' · '}
                    <a href={`${config.explorer}/address/${account}`} target="_blank" rel="noreferrer">
                      explorer
                    </a>
                  </>
                )}
              </span>
            </div>
          </li>
        );
      })}
    </ul>
  );
};
