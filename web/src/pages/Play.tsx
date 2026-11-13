import { ArrowRight, ScrollText, Sparkles } from 'lucide-react';
import { Link } from 'react-router';
import { Coin, Mascot } from '../brand/Mascot';
import { EventList } from '../components/EventList';
import { Highlights } from '../components/Highlights';
import { Leaderboard } from '../components/Leaderboard';
import { PlayerCard } from '../components/PlayerCard';
import { StatStrip } from '../components/StatStrip';
import { Ticker } from '../components/Ticker';
import { TokenCard } from '../components/TokenCard';
import { Card, Chip, SectionTitle } from '../components/ui';
import { useGame } from '../state/game';

const Hero = () => {
  const { stats } = useGame();
  const over = stats?.season_over ?? false;
  return (
    <section className="hero">
      <div className="hero-copy">
        <Chip tone={over ? 'hot' : 'brand'} icon={<Sparkles size={14} />}>
          {over ? 'Season over' : 'Season live on testnet'}
        </Chip>
        <h1>
          You can't buy <span className="grad-text">$FLU</span>.
          <br />
          You have to catch it.
        </h1>
        <p className="lede">A NEAR memecoin that spreads like a flu.</p>
        <div className="hero-cta">
          <Link className="btn btn-ghost btn-md" to="/how-to-play">
            <span>How it works</span>
            <ArrowRight size={18} />
          </Link>
        </div>
      </div>
      <div className="hero-art" aria-hidden="true">
        <div className="hero-glow" />
        <Mascot mood="contagious" size={280} animate className="hero-mascot" />
        <Coin size={64} className="orbit orbit-a" />
        <Coin size={44} className="orbit orbit-b" />
        <Mascot mood="celebrating" size={52} className="orbit orbit-c" />
      </div>
    </section>
  );
};

export const Play = () => {
  const { recent } = useGame();
  return (
    <>
      <Hero />
      <Highlights />
      <Ticker />
      <div className="grid-play">
        <PlayerCard />
        <TokenCard />
      </div>
      <StatStrip />
      <div className="grid-2">
        <Leaderboard count={5} link />
        <Card>
          <SectionTitle icon={<ScrollText size={20} />} title="Latest activity" />
          <EventList events={recent === null ? null : recent.slice(-5)} loading={recent === null} rows={5} compact />
          <Link className="more" to="/activity">
            See all activity
          </Link>
        </Card>
      </div>
    </>
  );
};
