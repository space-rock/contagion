import { BookOpen, CircleHelp, ScrollText } from 'lucide-react';
import { Link } from 'react-router';
import { Mascot } from '../brand/Mascot';
import { RuleCards, Steps } from '../components/Rules';
import { Card, SectionTitle } from '../components/ui';
import { useGame } from '../state/game';

export const HowToPlay = () => {
  const { config, loading } = useGame();
  return (
    <>
      <SectionTitle icon={<BookOpen size={22} />} title="How to play" />
      <Steps config={config} />
      <SectionTitle icon={<ScrollText size={22} />} title="The rules" />
      <RuleCards config={config} loading={loading && config === null} />
      <Card className="faq-cta">
        <Mascot mood="healthy" size={72} animate />
        <div>
          <h3>Still have questions?</h3>
          <p className="muted">Fees, clocks, burns, vaccines, wallets and what happens when a season ends.</p>
        </div>
        <Link className="btn btn-primary btn-md" to="/faq">
          <CircleHelp size={18} />
          <span>Read the FAQ</span>
        </Link>
      </Card>
    </>
  );
};
