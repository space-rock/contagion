import { Coin, Mascot } from '../brand/Mascot';
import { MOODS } from '../brand/art';

const SIZES = [16, 24, 32, 48, 64, 96];

const Sheet = ({ tone }: { tone: 'dark' | 'light' }) => (
  <section className={`brand-sheet ${tone}`}>
    <div className="brand-row">
      {MOODS.map((mood) => (
        <figure key={mood}>
          <Mascot mood={mood} size={150} />
          <figcaption>{mood}</figcaption>
        </figure>
      ))}
    </div>
    <div className="brand-row sizes">
      {SIZES.map((size) => (
        <Mascot key={`m${size}`} mood="contagious" size={size} />
      ))}
      <span className="sep" />
      {SIZES.map((size) => (
        <Coin key={`c${size}`} size={size} />
      ))}
    </div>
  </section>
);

export const Brand = () => (
  <div className="brand-page">
    <Sheet tone="dark" />
    <Sheet tone="light" />
  </div>
);

export const BrandOg = () => (
  <div className="og">
    <div className="aurora" aria-hidden="true" />
    <div className="og-copy">
      <div className="og-brand">
        <Mascot mood="contagious" size={72} />
        <span className="wordmark">Contagion</span>
      </div>
      <h1>
        You can't buy <span className="grad-text">$FLU</span>.
        <br />
        You have to catch it.
      </h1>
      <p>A NEAR memecoin that spreads like a flu.</p>
    </div>
    <div className="og-art">
      <Mascot mood="celebrating" size={360} />
      <Coin size={120} className="og-coin" />
    </div>
  </div>
);

export const BrandIcon = () => (
  <div className="icon-tile">
    <Mascot mood="contagious" size={150} />
  </div>
);
