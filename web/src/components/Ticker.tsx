import { describeEvent, eventKind } from '../lib/events';
import { useGame } from '../state/game';

export const Ticker = () => {
  const { recent } = useGame();
  if (!recent || recent.length === 0) return null;
  const items = [...recent].reverse();
  return (
    <div className="ticker" aria-label="Latest activity">
      <div className="ticker-track">
        {[0, 1].map((copy) => (
          <ul key={copy} aria-hidden={copy === 1}>
            {items.map((entry) => {
              const { who, text } = describeEvent(entry);
              return (
                <li key={`${copy}-${entry.index}`}>
                  <i className={`dot kind-${eventKind(entry)}`} />
                  <b>{who}</b> {text}
                </li>
              );
            })}
          </ul>
        ))}
      </div>
    </div>
  );
};
