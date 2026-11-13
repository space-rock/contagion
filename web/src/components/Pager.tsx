import { ChevronLeft, ChevronRight, ChevronsLeft, ChevronsRight } from 'lucide-react';
import { pageWindow } from '../lib/game';

export const Pager = ({ page, pages, onPage }: { page: number; pages: number; onPage: (page: number) => void }) => {
  if (pages <= 1) return null;
  return (
    <nav className="pager" aria-label="Pages">
      <button onClick={() => onPage(1)} disabled={page === 1} aria-label="Newest">
        <ChevronsLeft size={16} />
      </button>
      <button onClick={() => onPage(page - 1)} disabled={page === 1} aria-label="Newer">
        <ChevronLeft size={16} />
      </button>
      {pageWindow(page, pages).map((number) => (
        <button key={number} onClick={() => onPage(number)} className={number === page ? 'on' : ''} aria-current={number === page ? 'page' : undefined}>
          {number}
        </button>
      ))}
      <button onClick={() => onPage(page + 1)} disabled={page === pages} aria-label="Older">
        <ChevronRight size={16} />
      </button>
      <button onClick={() => onPage(pages)} disabled={page === pages} aria-label="Oldest">
        <ChevronsRight size={16} />
      </button>
    </nav>
  );
};
