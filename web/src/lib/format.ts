const TOKEN_UNIT = 10n ** 18n;
const NEAR_UNIT = 10n ** 24n;

export const groupDigits = (digits: string): string => digits.replace(/\B(?=(\d{3})+(?!\d))/g, ',');

const scaled = (raw: string | bigint, unit: bigint, places: number): string => {
  const value = BigInt(raw);
  const factor = 10n ** BigInt(places);
  const step = unit / factor;
  const rounded = (value + step / 2n) / step;
  const whole = rounded / factor;
  const fraction = (rounded % factor).toString().padStart(places, '0').replace(/0+$/, '');
  return groupDigits(whole.toString()) + (fraction ? `.${fraction}` : '');
};

export const tokens = (raw: string | bigint, places = 2): string => scaled(raw, TOKEN_UNIT, places);

export const near = (raw: string | bigint, places = 4): string => scaled(raw, NEAR_UNIT, places);

export const duration = (ns: bigint): string => {
  let seconds = Number(ns / 1_000_000_000n);
  if (seconds <= 0) return '0s';
  const days = Math.floor(seconds / 86_400);
  seconds %= 86_400;
  const hours = Math.floor(seconds / 3_600);
  seconds %= 3_600;
  const minutes = Math.floor(seconds / 60);
  seconds %= 60;
  if (days > 0) return `${days}d ${hours}h`;
  if (hours > 0) return `${hours}h ${minutes}m`;
  if (minutes > 0) return `${minutes}m ${seconds}s`;
  return `${seconds}s`;
};


export const bps = (value: number): string => `${(value / 100).toFixed(value % 100 === 0 ? 0 : 1)}%`;

export const pct = (part: number, whole: number): string => (whole > 0 ? `${((part / whole) * 100).toFixed(1)}%` : '0%');

export const truncateAccount = (id: string, head = 8, tail = 10): string =>
  id.length > head + tail + 4 ? `${id.slice(0, head)}…${id.slice(-tail)}` : id;

export const timeAgo = (then: Date, now = new Date()): string => {
  const seconds = Math.max(0, Math.floor((now.getTime() - then.getTime()) / 1000));
  if (seconds < 45) return 'just now';
  if (seconds < 3_600) return `${Math.max(1, Math.round(seconds / 60))}m ago`;
  if (seconds < 86_400) return `${Math.round(seconds / 3_600)}h ago`;
  return `${Math.round(seconds / 86_400)}d ago`;
};

export const longDuration = (ns: bigint): string => {
  const seconds = Number(ns / 1_000_000_000n);
  const unit = (value: number, name: string): string => `${value} ${name}${value === 1 ? '' : 's'}`;
  if (seconds >= 2 * 86_400) return unit(Math.round(seconds / 86_400), 'day');
  if (seconds >= 2 * 3_600) return unit(Math.round(seconds / 3_600), 'hour');
  if (seconds >= 60) return unit(Math.round(seconds / 60), 'minute');
  return unit(Math.max(0, seconds), 'second');
};
