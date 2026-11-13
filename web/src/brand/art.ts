export type Mood = 'contagious' | 'expired' | 'recovered' | 'immune' | 'healthy' | 'celebrating';
export type Detail = 'full' | 'small';

type Palette = { light: string; mid: string; dark: string; spike: string; ink: string };

export const MOODS: Mood[] = ['contagious', 'celebrating', 'expired', 'recovered', 'immune', 'healthy'];

const PALETTES: Record<Mood, Palette> = {
  contagious: { light: '#d9f99d', mid: '#22c55e', dark: '#0e7490', spike: '#16a34a', ink: '#052e16' },
  celebrating: { light: '#d9f99d', mid: '#22c55e', dark: '#0e7490', spike: '#16a34a', ink: '#052e16' },
  expired: { light: '#fef08a', mid: '#eab308', dark: '#92400e', spike: '#ca8a04', ink: '#422006' },
  recovered: { light: '#cffafe', mid: '#38bdf8', dark: '#4338ca', spike: '#0ea5e9', ink: '#0c1f4a' },
  immune: { light: '#fae8ff', mid: '#c084fc', dark: '#be185d', spike: '#a855f7', ink: '#3b0764' },
  healthy: { light: '#ecfdf5', mid: '#6ee7b7', dark: '#0d9488', spike: '#34d399', ink: '#064e3b' },
};

const round = (value: number): number => Math.round(value * 10) / 10;

const spikePaths = (count: number, spike: string, small: boolean): string => {
  const lines: string[] = [];
  const dots: string[] = [];
  for (let i = 0; i < count; i += 1) {
    const angle = (i * 2 * Math.PI) / count - Math.PI / 2;
    const cos = Math.cos(angle);
    const sin = Math.sin(angle);
    lines.push(`M${round(64 + 33 * cos)} ${round(64 + 33 * sin)}L${round(64 + 49 * cos)} ${round(64 + 49 * sin)}`);
    dots.push(`M${round(64 + 55 * cos)} ${round(64 + 55 * sin)}h0`);
  }
  const lineWidth = small ? 10 : 8;
  const dotWidth = small ? 17 : 14;
  return (
    `<path d="${lines.join('')}" stroke="${spike}" stroke-width="${lineWidth}" stroke-linecap="round" fill="none"/>` +
    `<path d="${dots.join('')}" stroke="${spike}" stroke-width="${dotWidth}" stroke-linecap="round" fill="none"/>`
  );
};

const eyes = (ink: string, small: boolean): string => {
  const rx = small ? 11 : 9.5;
  const ry = small ? 13 : 11.5;
  const pupil = small ? 6.5 : 5.5;
  const glint = small ? 2.4 : 2;
  const eye = (cx: number): string =>
    `<ellipse cx="${cx}" cy="60" rx="${rx}" ry="${ry}" fill="#fff"/>` +
    `<circle cx="${cx + 2}" cy="62" r="${pupil}" fill="${ink}"/>` +
    `<circle cx="${cx + 4}" cy="59" r="${glint}" fill="#fff"/>`;
  return `<g class="flu-eyes">${eye(52)}${eye(76)}</g>`;
};

const happyEyes = (ink: string): string =>
  `<g class="flu-eyes"><path d="M42 62Q52 51 62 62M66 62Q76 51 86 62" stroke="${ink}" stroke-width="4" stroke-linecap="round" fill="none"/></g>`;

const dizzyEyes = (ink: string): string => {
  const spiral = (cx: number): string =>
    `<ellipse cx="${cx}" cy="60" rx="10" ry="11.5" fill="#fff"/>` +
    `<path d="M${cx} 60m-1.5 0a1.5 1.5 0 1 1 3 0a3.5 3.5 0 1 1-7 0a5.5 5.5 0 1 1 11 0" stroke="${ink}" stroke-width="2" fill="none" stroke-linecap="round"/>`;
  return `<g class="flu-eyes">${spiral(52)}${spiral(76)}</g>`;
};

const shades = (ink: string): string =>
  `<rect x="39" y="52" width="23" height="16" rx="7" fill="${ink}"/>` +
  `<rect x="66" y="52" width="23" height="16" rx="7" fill="${ink}"/>` +
  `<path d="M62 58h4" stroke="${ink}" stroke-width="3"/>` +
  `<path d="M44 56l7 0M71 56l7 0" stroke="#fff" stroke-opacity=".55" stroke-width="2.5" stroke-linecap="round"/>`;

const face = (mood: Mood, ink: string, small: boolean): string => {
  const blush = small
    ? ''
    : '<circle cx="40" cy="73" r="5" fill="#fb7185" fill-opacity=".42"/><circle cx="88" cy="73" r="5" fill="#fb7185" fill-opacity=".42"/>';
  switch (mood) {
    case 'contagious':
      return `${eyes(ink, small)}<path d="M47 75Q64 95 81 75" stroke="${ink}" stroke-width="4" stroke-linecap="round" fill="none"/>${blush}`;
    case 'celebrating':
      return (
        `${happyEyes(ink)}<path d="M48 74Q64 102 80 74Z" fill="${ink}"/><ellipse cx="64" cy="88" rx="7" ry="4" fill="#fb7185"/>${blush}` +
        '<path d="M100 22l2.5 6 6 2.5-6 2.5-2.5 6-2.5-6-6-2.5 6-2.5zM24 30l2 4.5 4.5 2-4.5 2-2 4.5-2-4.5-4.5-2 4.5-2z" fill="#fde68a"/>'
      );
    case 'expired':
      return (
        `${dizzyEyes(ink)}<path d="M49 82q4.5-6 8 0t8 0 8 0 8 0" stroke="${ink}" stroke-width="3.2" stroke-linecap="round" fill="none"/>` +
        '<path d="M96 34q-6 9 0 12.5q6-3.5 0-12.5z" fill="#7dd3fc"/>'
      );
    case 'recovered':
      return (
        `${happyEyes(ink)}<path d="M52 76Q64 86 76 76" stroke="${ink}" stroke-width="3.6" stroke-linecap="round" fill="none"/>${blush}` +
        '<path d="M99 24l2.2 5.3 5.3 2.2-5.3 2.2-2.2 5.3-2.2-5.3-5.3-2.2 5.3-2.2zM27 34l1.5 3.5 3.5 1.5-3.5 1.5-1.5 3.5-1.5-3.5-3.5-1.5 3.5-1.5z" fill="#fff"/>'
      );
    case 'immune':
      return (
        `${shades(ink)}<path d="M54 78Q66 88 79 75" stroke="${ink}" stroke-width="3.8" stroke-linecap="round" fill="none"/>${blush}` +
        `<g transform="translate(90 88)"><path d="M0-11L10-7.5V1C10 8 5 12 0 14C-5 12-10 8-10 1V-7.5Z" fill="#fff" stroke="${ink}" stroke-width="2.2" stroke-linejoin="round"/><path d="M-4.5 0l3.2 3.4 6-7" stroke="#7c3aed" stroke-width="2.8" stroke-linecap="round" stroke-linejoin="round" fill="none"/></g>`
      );
    case 'healthy':
      return (
        `${eyes(ink, small)}<path d="M40 53L58 46M88 53L70 46" stroke="${ink}" stroke-width="3" stroke-linecap="round"/>` +
        `<ellipse cx="64" cy="82" rx="4.5" ry="5.5" fill="${ink}"/>`
      );
  }
};

export const mascotInner = (mood: Mood, detail: Detail, id: string): string => {
  const p = PALETTES[mood];
  const small = detail === 'small';
  return (
    `<defs><radialGradient id="${id}" cx=".35" cy=".3" r=".85"><stop offset="0" stop-color="${p.light}"/><stop offset=".5" stop-color="${p.mid}"/><stop offset="1" stop-color="${p.dark}"/></radialGradient></defs>` +
    spikePaths(small ? 8 : 12, p.spike, small) +
    `<circle cx="64" cy="64" r="36" fill="url(#${id})" stroke="${p.ink}" stroke-opacity=".55" stroke-width="3"/>` +
    '<ellipse cx="50" cy="45" rx="13" ry="7" fill="#fff" fill-opacity=".3" transform="rotate(-30 50 45)"/>' +
    face(mood, p.ink, small)
  );
};

export const mascotSvg = (mood: Mood, detail: Detail = 'full', size = 128): string =>
  `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 128 128" width="${size}" height="${size}">${mascotInner(mood, detail, `g-${mood}`)}</svg>`;

export const coinInner = (id: string): string =>
  `<defs><linearGradient id="${id}r" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#d9f99d"/><stop offset=".5" stop-color="#22c55e"/><stop offset="1" stop-color="#06b6d4"/></linearGradient><radialGradient id="${id}d" cx=".5" cy=".4" r=".7"><stop offset="0" stop-color="#065f46"/><stop offset="1" stop-color="#022c22"/></radialGradient></defs>` +
  '<circle cx="64" cy="64" r="62" fill="url(#' + id + 'r)"/>' +
  '<circle cx="64" cy="64" r="59" fill="none" stroke="#022c22" stroke-opacity=".4" stroke-width="5" stroke-dasharray="4 7.46"/>' +
  '<circle cx="64" cy="64" r="51" fill="url(#' + id + 'd)" stroke="#bbf7d0" stroke-opacity=".4" stroke-width="2"/>' +
  `<g transform="translate(64 65) scale(.74) translate(-64 -64)">${mascotInner('contagious', 'small', `${id}m`)}</g>`;

export const coinSvg = (size = 128): string =>
  `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 128 128" width="${size}" height="${size}">${coinInner('c')}</svg>`;
