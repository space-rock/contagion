import { Mascot } from '../brand/Mascot';
import type { Mood } from '../brand/art';

type Highlight = { mood: Mood; tone: 'brand' | 'hot' | 'immune'; title: string; text: string };

const ITEMS: Highlight[] = [
  { mood: 'healthy', tone: 'brand', title: 'No sale, no airdrop', text: 'Every holder caught it from another holder.' },
  { mood: 'contagious', tone: 'brand', title: 'Spread it', text: 'Every catch earns a bonus, and each generation mints less than the last.' },
  { mood: 'expired', tone: 'hot', title: 'Spread or burn', text: 'Miss your clock and anyone can cull you, burning your locked FLU.' },
  { mood: 'immune', tone: 'immune', title: 'Fixed rules', text: 'Set once in the contract. No owner, no admin, no changes.' },
];

export const Highlights = () => (
  <ul className="highlights">
    {ITEMS.map((item) => (
      <li key={item.title} className="highlight" data-tone={item.tone}>
        <Mascot mood={item.mood} size={68} animate />
        <h3>{item.title}</h3>
        <p>{item.text}</p>
      </li>
    ))}
  </ul>
);
