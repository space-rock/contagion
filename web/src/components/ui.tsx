import type { ButtonHTMLAttributes, ReactNode } from 'react';
import { Coin } from '../brand/Mascot';
import { tokens } from '../lib/format';

type ButtonProps = ButtonHTMLAttributes<HTMLButtonElement> & {
  tone?: 'primary' | 'ghost' | 'soft' | 'danger';
  busy?: boolean;
  icon?: ReactNode;
  size?: 'md' | 'sm';
};

export const Button = ({ tone = 'primary', busy, icon, size = 'md', children, disabled, className = '', ...rest }: ButtonProps) => (
  <button className={`btn btn-${tone} btn-${size} ${className}`} disabled={disabled || busy} {...rest}>
    {busy ? <span className="spinner" aria-hidden="true" /> : icon}
    <span>{busy ? 'Waiting for wallet' : children}</span>
  </button>
);

export const Card = ({
  children,
  className = '',
  glow = false,
}: {
  children: ReactNode;
  className?: string;
  glow?: boolean;
}) => <section className={`card ${glow ? 'card-glow' : ''} ${className}`}>{children}</section>;

export const SectionTitle = ({ icon, title, hint, action }: { icon?: ReactNode; title: string; hint?: string; action?: ReactNode }) => (
  <header className="section-title">
    <div>
      <h2>
        {icon}
        {title}
      </h2>
      {hint && <p className="muted">{hint}</p>}
    </div>
    {action}
  </header>
);

export const ProgressBar = ({
  value,
  tone = 'brand',
  label,
  loading,
}: {
  value: number;
  tone?: 'brand' | 'hot' | 'immune' | 'cool';
  label?: string;
  loading?: boolean;
}) => (
  <div className="progress" role="progressbar" aria-valuemin={0} aria-valuemax={100} aria-valuenow={Math.round(value * 100)} aria-label={label}>
    {loading ? (
      <span className="sk sk-bar" />
    ) : (
      <span className={`progress-fill tone-${tone}`} style={{ width: `${Math.min(100, Math.max(2, value * 100))}%` }} />
    )}
  </div>
);

export const Pips = ({ filled, total }: { filled: number; total: number }) => (
  <span className="pips" aria-label={`${filled} of ${total}`}>
    {Array.from({ length: total }, (_, i) => (
      <i key={i} className={i < filled ? 'on' : ''} />
    ))}
  </span>
);

export const ClockRing = ({
  fraction,
  label,
  sub,
  size = 96,
  tone,
}: {
  fraction: number | null;
  label: string;
  sub?: string;
  size?: number;
  tone?: 'brand' | 'hot' | 'cool';
}) => {
  const radius = 42;
  const circumference = 2 * Math.PI * radius;
  const value = fraction === null ? 0 : fraction;
  const color = tone ?? (value > 0.5 ? 'brand' : value > 0.2 ? 'amber' : 'hot');
  return (
    <div className="ring" style={{ width: size, height: size }}>
      <svg viewBox="0 0 100 100" width={size} height={size} aria-hidden="true">
        <circle cx="50" cy="50" r={radius} className="ring-track" />
        <circle
          cx="50"
          cy="50"
          r={radius}
          className={`ring-fill ring-${color}`}
          strokeDasharray={circumference}
          strokeDashoffset={circumference * (1 - value)}
          transform="rotate(-90 50 50)"
        />
      </svg>
      <div className="ring-label">
        <strong>{label}</strong>
        {sub && <span>{sub}</span>}
      </div>
    </div>
  );
};

export const Flu = ({ raw, places = 2, size = 18, loading }: { raw?: string | bigint; places?: number; size?: number; loading?: boolean }) => (
  <span className="flu-amount">
    <Coin size={size} />
    {loading || raw === undefined ? <span className="sk sk-inline" /> : <b>{tokens(raw, places)}</b>}
    <span className="flu-symbol">FLU</span>
  </span>
);

export const Chip = ({
  children,
  tone = 'plain',
  icon,
}: {
  children: ReactNode;
  tone?: 'plain' | 'brand' | 'hot' | 'cool' | 'immune';
  icon?: ReactNode;
}) => (
  <span className={`chip chip-${tone}`}>
    {icon}
    {children}
  </span>
);

export const Notice = ({ tone = 'info', children }: { tone?: 'info' | 'warn' | 'error'; children: ReactNode }) => (
  <p className={`notice notice-${tone}`}>{children}</p>
);
