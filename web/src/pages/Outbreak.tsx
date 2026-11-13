import { Radar, Skull } from 'lucide-react';
import { useMemo } from 'react';
import { useSearchParams } from 'react-router';
import { Doctors } from '../components/Doctors';
import { OutbreakMap } from '../components/map/OutbreakMap';
import { Card, SectionTitle } from '../components/ui';
import { makeSyntheticHosts } from '../lib/synthetic';
import { useGame } from '../state/game';

export const Outbreak = () => {
  const game = useGame();
  const [params] = useSearchParams();
  const synthetic = import.meta.env.DEV ? Number(params.get('synthetic') ?? 0) : 0;
  const fake = useMemo(() => (synthetic > 0 ? makeSyntheticHosts(synthetic) : null), [synthetic]);
  const hosts = fake ?? game.hosts;

  return (
    <>
      <SectionTitle icon={<Radar size={22} />} title="The outbreak" hint="Each dot is an infected account, linked to whoever it caught it from." />
      <OutbreakMap hosts={hosts} config={game.config} loading={hosts === null} focus={params.get('focus')} />
      <Card className="doctors-card">
        <SectionTitle
          icon={<Skull size={20} />}
          title="Plague doctors"
          hint="Carriers whose clock ran out. Cull one to burn part of their locked FLU and keep a cut."
        />
        <Doctors />
      </Card>
    </>
  );
};
