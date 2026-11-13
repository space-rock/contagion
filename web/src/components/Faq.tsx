import { Search } from 'lucide-react';
import { useMemo, useState } from 'react';
import { Mascot } from '../brand/Mascot';
import { buildFaq, countFaq, filterFaq } from '../lib/faq';
import type { Config } from '../types';

export const Faq = ({ config }: { config: Config | null }) => {
  const [query, setQuery] = useState('');
  const [group, setGroup] = useState('all');
  const groups = useMemo(() => buildFaq(config), [config]);
  const shown = useMemo(() => filterFaq(groups, query, group), [groups, query, group]);
  const total = countFaq(shown);

  return (
    <section className="faq">
      <div className="faq-tools">
        <label className="faq-search">
          <Search size={18} aria-hidden="true" />
          <input value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Search the questions" aria-label="Search the questions" />
        </label>
        <div className="faq-chips" role="group" aria-label="Question groups">
          <button className={group === 'all' ? 'on' : ''} onClick={() => setGroup('all')} aria-pressed={group === 'all'}>
            All
          </button>
          {groups.map((g) => (
            <button key={g.id} className={group === g.id ? 'on' : ''} onClick={() => setGroup(g.id)} aria-pressed={group === g.id}>
              {g.title}
            </button>
          ))}
        </div>
      </div>
      <p className="muted small" aria-live="polite">
        {total} {total === 1 ? 'question' : 'questions'}
      </p>
      {shown.map((g) => (
        <div key={g.id} className="faq-group">
          <h3>{g.title}</h3>
          {g.items.map((item) => (
            <details key={item.id} className="faq-item">
              <summary>{item.q}</summary>
              <p>{item.a}</p>
            </details>
          ))}
        </div>
      ))}
      {total === 0 && (
        <div className="empty">
          <Mascot mood="healthy" size={64} animate />
          <span>No question matches. Try fewer words.</span>
        </div>
      )}
    </section>
  );
};
