import { useId } from 'react';
import { type Mood, coinInner, mascotInner } from './art';

const safeId = (raw: string): string => `f${raw.replace(/[^a-zA-Z0-9]/g, '')}`;

export const Mascot = ({
  mood = 'contagious',
  size = 64,
  hue = 0,
  animate = false,
  className = '',
  label,
}: {
  mood?: Mood;
  size?: number;
  hue?: number;
  animate?: boolean;
  className?: string;
  label?: string;
}) => {
  const id = safeId(useId());
  return (
    <svg
      viewBox="0 0 128 128"
      width={size}
      height={size}
      role="img"
      aria-label={label ?? `Flu mascot, ${mood}`}
      className={`flu ${animate ? 'flu-anim' : ''} ${className}`}
      style={hue ? { filter: `hue-rotate(${hue}deg)` } : undefined}
      dangerouslySetInnerHTML={{ __html: mascotInner(mood, size <= 40 ? 'small' : 'full', id) }}
    />
  );
};

export const Coin = ({ size = 32, spin = false, className = '' }: { size?: number; spin?: boolean; className?: string }) => {
  const id = safeId(useId());
  return (
    <svg
      viewBox="0 0 128 128"
      width={size}
      height={size}
      role="img"
      aria-label="FLU coin"
      className={`coin ${spin ? 'coin-spin' : ''} ${className}`}
      dangerouslySetInnerHTML={{ __html: coinInner(id) }}
    />
  );
};
