import { quadtree } from 'd3-quadtree';
import { select } from 'd3-selection';
import 'd3-transition';
import { type ZoomBehavior, type ZoomTransform, zoom, zoomIdentity } from 'd3-zoom';
import { Maximize2, Minus, Plus, Search } from 'lucide-react';
import { type PointerEvent as ReactPointerEvent, useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { Link } from 'react-router';
import { Mascot } from '../../brand/Mascot';
import { hueShift } from '../../lib/avatar';
import { tokens, truncateAccount } from '../../lib/format';
import { type HostClass, classify, moodFor, titleFor } from '../../lib/game';
import { layoutOutbreak, spreaderCount } from '../../lib/tree';
import type { Config, Host } from '../../types';
import { Chip } from '../ui';

const RADIUS = 360;
const CLASSES: HostClass[] = ['live', 'expired', 'recovered'];
const COLORS: Record<HostClass, string> = { live: '#22c55e', expired: '#f59e0b', recovered: '#38bdf8' };
const LABELS: Record<HostClass, string> = { live: 'Live', expired: 'Expired', recovered: 'Recovered' };
const TAU = Math.PI * 2;

type Filter = Record<HostClass, boolean>;
type Item = { x: number; y: number; i: number };

export const OutbreakMap = ({
  hosts,
  config,
  loading,
  focus,
}: {
  hosts: Host[] | null;
  config: Config | null;
  loading: boolean;
  focus: string | null;
}) => {
  const stage = useRef<HTMLDivElement>(null);
  const canvas = useRef<HTMLCanvasElement>(null);
  const tooltip = useRef<HTMLDivElement>(null);
  const behavior = useRef<ZoomBehavior<HTMLCanvasElement, unknown> | null>(null);
  const transform = useRef<ZoomTransform>(zoomIdentity);
  const size = useRef({ w: 800, h: 560 });
  const frame = useRef(0);
  const hoverIndex = useRef(-1);
  const pinnedIndex = useRef(-1);
  const focused = useRef<string | null>(null);

  const [filter, setFilter] = useState<Filter>({ live: true, expired: true, recovered: true });
  const [hover, setHover] = useState(-1);
  const [pinned, setPinned] = useState(-1);
  const [query, setQuery] = useState('');
  const [resized, setResized] = useState(0);

  const list = hosts ?? [];
  const spreaders = spreaderCount(list);
  const layout = useMemo(() => layoutOutbreak(list, RADIUS), [spreaders, hosts === null]);
  const byId = useMemo(() => new Map(list.map((host) => [host.account_id, host])), [hosts]);

  const codes = useMemo(() => {
    const out = new Uint8Array(layout.nodes.length);
    layout.nodes.forEach((node, i) => {
      const cls = classify(byId.get(node.id) as Host);
      out[i] = cls === 'live' ? 0 : cls === 'expired' ? 1 : 2;
    });
    return out;
  }, [layout, byId]);

  const counts = useMemo(() => {
    const out = [0, 0, 0];
    codes.forEach((code) => {
      out[code] += 1;
    });
    return out;
  }, [codes]);

  const tree = useMemo(() => {
    const items: Item[] = [];
    layout.nodes.forEach((node, i) => {
      if (filter[CLASSES[codes[i]]]) items.push({ x: node.x, y: node.y, i });
    });
    return quadtree<Item>()
      .x((d) => d.x)
      .y((d) => d.y)
      .addAll(items);
  }, [layout, codes, filter]);

  const filterRef = useRef(filter);
  const layoutRef = useRef(layout);
  const codesRef = useRef(codes);
  filterRef.current = filter;
  layoutRef.current = layout;
  codesRef.current = codes;
  hoverIndex.current = hover;
  pinnedIndex.current = pinned;

  const draw = useCallback(() => {
    frame.current = 0;
    const el = canvas.current;
    const ctx = el?.getContext('2d');
    if (!el || !ctx) return;
    const dpr = window.devicePixelRatio || 1;
    const t = transform.current;
    const { nodes } = layoutRef.current;
    const cs = codesRef.current;
    const show = [filterRef.current.live, filterRef.current.expired, filterRef.current.recovered];

    ctx.setTransform(1, 0, 0, 1, 0, 0);
    ctx.clearRect(0, 0, el.width, el.height);
    ctx.setTransform(dpr * t.k, 0, 0, dpr * t.k, dpr * t.x, dpr * t.y);

    ctx.lineWidth = 1 / t.k;
    ctx.strokeStyle = 'rgba(148, 163, 184, 0.28)';
    ctx.beginPath();
    for (let i = 0; i < nodes.length; i += 1) {
      const node = nodes[i];
      if (node.parent < 0 || !show[cs[i]]) continue;
      const parent = nodes[node.parent];
      ctx.moveTo(parent.x, parent.y);
      ctx.lineTo(node.x, node.y);
    }
    ctx.stroke();

    for (let c = 0; c < 3; c += 1) {
      if (!show[c]) continue;
      ctx.fillStyle = COLORS[CLASSES[c]];
      ctx.beginPath();
      for (let i = 0; i < nodes.length; i += 1) {
        if (cs[i] !== c) continue;
        const node = nodes[i];
        const r = (node.zero ? 7 : 4) / t.k;
        ctx.moveTo(node.x + r, node.y);
        ctx.arc(node.x, node.y, r, 0, TAU);
      }
      ctx.fill();
    }

    const mark = (index: number, width: number): void => {
      if (index < 0 || index >= nodes.length || !show[cs[index]]) return;
      const node = nodes[index];
      ctx.lineWidth = width / t.k;
      ctx.strokeStyle = '#ffffff';
      ctx.beginPath();
      ctx.arc(node.x, node.y, 9 / t.k, 0, TAU);
      ctx.stroke();
    };
    mark(pinnedIndex.current, 3);
    mark(hoverIndex.current, 2);

    const active = hoverIndex.current >= 0 ? hoverIndex.current : pinnedIndex.current;
    const card = tooltip.current;
    if (card) {
      if (active >= 0 && active < nodes.length) {
        const node = nodes[active];
        card.style.transform = `translate(${t.applyX(node.x)}px, ${t.applyY(node.y)}px)`;
        card.style.opacity = '1';
      } else {
        card.style.opacity = '0';
      }
    }
  }, []);

  const schedule = useCallback(() => {
    if (frame.current === 0) frame.current = requestAnimationFrame(draw);
  }, [draw]);

  const fit = useCallback(() => {
    const el = canvas.current;
    const z = behavior.current;
    if (!el || !z) return;
    const { w, h } = size.current;
    const k = Math.min(w, h) / (2 * (RADIUS + 30));
    select(el).call(z.transform, zoomIdentity.translate(w / 2, h / 2).scale(k));
  }, []);

  const centerOn = useCallback((index: number) => {
    const el = canvas.current;
    const z = behavior.current;
    const node = layoutRef.current.nodes[index];
    if (!el || !z || !node) return;
    const { w, h } = size.current;
    const k = 7;
    select(el)
      .transition()
      .duration(450)
      .call(z.transform, zoomIdentity.translate(w / 2 - node.x * k, h / 2 - node.y * k).scale(k));
    setPinned(index);
    setHover(-1);
  }, []);

  useEffect(() => {
    const el = canvas.current;
    const holder = stage.current;
    if (!el || !holder) return;
    const z = zoom<HTMLCanvasElement, unknown>()
      .scaleExtent([0.3, 90])
      .on('zoom', (event) => {
        transform.current = event.transform;
        schedule();
      });
    behavior.current = z;
    select(el).call(z);

    const observer = new ResizeObserver(() => {
      const w = Math.max(240, Math.round(holder.clientWidth));
      const h = Math.max(280, Math.round(holder.clientHeight));
      const dpr = window.devicePixelRatio || 1;
      size.current = { w, h };
      el.width = Math.round(w * dpr);
      el.height = Math.round(h * dpr);
      el.style.width = `${w}px`;
      el.style.height = `${h}px`;
      setResized((value) => value + 1);
      schedule();
    });
    observer.observe(holder);
    return () => {
      observer.disconnect();
      select(el).on('.zoom', null);
      cancelAnimationFrame(frame.current);
      frame.current = 0;
    };
  }, [schedule]);

  const nodeCount = layout.nodes.length;
  const fitted = useRef(false);
  useEffect(() => {
    if (nodeCount > 0 && !fitted.current && resized > 0) {
      fitted.current = true;
      fit();
    }
  }, [nodeCount, resized, fit]);

  useEffect(() => {
    schedule();
  }, [layout, codes, filter, hover, pinned, resized, schedule]);

  useEffect(() => {
    if (!focus || focused.current === focus || nodeCount === 0) return;
    const index = layout.index.get(focus);
    if (index === undefined) return;
    focused.current = focus;
    centerOn(index);
  }, [focus, layout, nodeCount, centerOn]);

  const locate = (event: ReactPointerEvent<HTMLCanvasElement>): number => {
    const el = canvas.current;
    if (!el) return -1;
    const rect = el.getBoundingClientRect();
    const t = transform.current;
    const [x, y] = t.invert([event.clientX - rect.left, event.clientY - rect.top]);
    return tree.find(x, y, 12 / t.k)?.i ?? -1;
  };

  const onMove = (event: ReactPointerEvent<HTMLCanvasElement>) => {
    if (event.buttons !== 0) return;
    const index = locate(event);
    if (index !== hoverIndex.current) setHover(index);
  };

  const onClick = (event: ReactPointerEvent<HTMLCanvasElement>) => {
    const index = locate(event);
    setPinned(index === pinnedIndex.current ? -1 : index);
    setHover(index);
  };

  const submitSearch = (event: React.FormEvent) => {
    event.preventDefault();
    const needle = query.trim().toLowerCase();
    if (!needle) return;
    const found = layout.nodes.findIndex((node, i) => filter[CLASSES[codes[i]]] && node.id.toLowerCase().includes(needle));
    if (found >= 0) centerOn(found);
  };

  const zoomBy = (factor: number) => {
    const el = canvas.current;
    const z = behavior.current;
    if (el && z) select(el).transition().duration(220).call(z.scaleBy, factor);
  };

  const shown = hover >= 0 ? hover : pinned;
  const shownNode = shown >= 0 ? layout.nodes[shown] : undefined;
  const shownHost = shownNode ? byId.get(shownNode.id) : undefined;
  const empty = !loading && nodeCount === 0;

  return (
    <div className="map">
      <div className="map-stage" ref={stage}>
        {loading && <div className="sk sk-map" />}
        {empty && (
          <div className="map-empty">
            <Mascot mood="healthy" size={88} animate />
            <p>Nobody has it yet.</p>
          </div>
        )}
        <canvas
          ref={canvas}
          className={loading || empty ? 'hidden' : ''}
          onPointerMove={onMove}
          onPointerLeave={() => setHover(-1)}
          onClick={onClick as unknown as React.MouseEventHandler<HTMLCanvasElement>}
          role="img"
          aria-label={`Outbreak tree with ${nodeCount} infected accounts`}
        />
        <div className="map-tip" ref={tooltip} style={{ opacity: 0 }}>
          {shownHost && (
            <div className="tip-card">
              <div className="tip-head">
                <Mascot mood={moodFor(shownHost)} size={44} hue={hueShift(shownHost.account_id)} />
                <div>
                  <strong>{truncateAccount(shownHost.account_id, 10, 10)}</strong>
                  <span className="muted small">{config ? titleFor(shownHost, config) : shownHost.status}</span>
                </div>
              </div>
              <div className="tip-chips">
                <Chip>Gen {shownHost.gen}</Chip>
                <Chip>{shownHost.points} pts</Chip>
                <Chip>{shownHost.children} caught</Chip>
              </div>
              <span className="small">{tokens(shownHost.viral)} locked, {tokens(shownHost.liquid)} spendable</span>
              {pinned >= 0 && shownHost.status === 'contagious' && !shownHost.expired && (
                <Link className="btn btn-primary btn-sm" to={`/?from=${encodeURIComponent(shownHost.account_id)}`}>
                  Catch it from here
                </Link>
              )}
            </div>
          )}
        </div>
        <div className="zoom-controls">
          <button onClick={() => zoomBy(1.6)} aria-label="Zoom in">
            <Plus size={18} />
          </button>
          <button onClick={() => zoomBy(1 / 1.6)} aria-label="Zoom out">
            <Minus size={18} />
          </button>
          <button onClick={fit} aria-label="Fit to screen">
            <Maximize2 size={16} />
          </button>
        </div>
      </div>

      <aside className="map-side" aria-label="Map filters">
        <h3>Show</h3>
        <div className="filters">
          {CLASSES.map((cls, i) => (
            <button
              key={cls}
              className={`filter ${filter[cls] ? 'on' : ''}`}
              onClick={() => setFilter((previous) => ({ ...previous, [cls]: !previous[cls] }))}
              aria-pressed={filter[cls]}
            >
              <i style={{ background: COLORS[cls] }} />
              <span>{LABELS[cls]}</span>
              <b>{counts[i]}</b>
            </button>
          ))}
        </div>
        <form className="map-search" onSubmit={submitSearch}>
          <Search size={16} aria-hidden="true" />
          <input value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Find an account" aria-label="Find an account" />
        </form>
        <p className="muted small">Hover for details, click to pin, drag to pan, scroll to zoom.</p>
      </aside>
    </div>
  );
};
