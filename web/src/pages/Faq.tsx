import { CircleHelp } from 'lucide-react';
import { Faq } from '../components/Faq';
import { SectionTitle } from '../components/ui';
import { useGame } from '../state/game';

export const FaqPage = () => {
  const { config } = useGame();
  return (
    <>
      <SectionTitle icon={<CircleHelp size={22} />} title="FAQ" />
      <Faq config={config} />
    </>
  );
};
