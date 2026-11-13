import { Coin, Mascot } from '../brand/Mascot';

const PIECES = 26;

export const Confetti = () => (
  <div className="confetti" aria-hidden="true">
    {Array.from({ length: PIECES }, (_, i) => {
      const angle = (i / PIECES) * 360;
      const distance = 90 + ((i * 37) % 90);
      const style = {
        '--angle': `${angle}deg`,
        '--distance': `${distance}px`,
        '--delay': `${(i % 6) * 40}ms`,
        '--spin': `${(i % 2 === 0 ? 1 : -1) * (180 + i * 20)}deg`,
      } as React.CSSProperties;
      return (
        <span key={i} className="confetti-piece" style={style}>
          {i % 3 === 0 ? <Mascot mood="celebrating" size={22} /> : i % 3 === 1 ? <Coin size={20} /> : <i className="confetti-dot" />}
        </span>
      );
    })}
  </div>
);
