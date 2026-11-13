import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { renderToStaticMarkup } from 'react-dom/server';
import { createElement } from 'react';
import { describe, expect, it } from 'vitest';
import { MOODS, coinSvg, mascotInner, mascotSvg } from './art';
import { Coin, Mascot } from './Mascot';

const brand = join(dirname(fileURLToPath(import.meta.url)), '../../../brand');
const read = (name: string) => readFileSync(join(brand, name), 'utf8');

describe('mascot art', () => {
  it('draws every mood with its own gradient and a face', () => {
    for (const mood of MOODS) {
      const svg = mascotSvg(mood);
      expect(svg.startsWith('<svg')).toBe(true);
      expect(svg).toContain(`fill="url(#g-${mood})"`);
      expect(svg).toContain('<circle cx="64" cy="64" r="36"');
    }
  });

  it('makes every mood look different', () => {
    const faces = new Set(MOODS.map((mood) => mascotSvg(mood)));
    expect(faces.size).toBe(MOODS.length);
  });

  it('uses fewer spikes in the small variant so it stays readable', () => {
    const count = (svg: string) => (svg.match(/M[\d.]+ [\d.]+L/g) ?? []).length;
    expect(count(mascotInner('contagious', 'full', 'x'))).toBe(12);
    expect(count(mascotInner('contagious', 'small', 'x'))).toBe(8);
  });

  it('namespaces gradient ids so several mascots can share a page', () => {
    expect(mascotInner('contagious', 'full', 'one')).toContain('id="one"');
    expect(mascotInner('contagious', 'full', 'two')).toContain('id="two"');
  });
});

describe('coin art', () => {
  it('stays small enough to live in the contract', () => {
    expect(new TextEncoder().encode(coinSvg()).length).toBeLessThanOrEqual(3_000);
  });

  it('has no external references', () => {
    expect(coinSvg()).not.toMatch(/https?:\/\/(?!www\.w3\.org)/);
    expect(coinSvg()).not.toContain('<image');
  });
});

describe('exported brand files', () => {
  it('match the builder, so the art cannot drift', () => {
    for (const mood of MOODS) expect(read(`flu-${mood}.svg`)).toBe(`${mascotSvg(mood, 'full')}\n`);
    expect(read('flu-small.svg')).toBe(`${mascotSvg('contagious', 'small')}\n`);
    expect(read('coin.svg')).toBe(`${coinSvg()}\n`);
  });
});

describe('components', () => {
  it('render each mood to markup', () => {
    for (const mood of MOODS) {
      const html = renderToStaticMarkup(createElement(Mascot, { mood, size: 96 }));
      expect(html).toContain('viewBox="0 0 128 128"');
      expect(html).toContain(`aria-label="Flu mascot, ${mood}"`);
    }
  });

  it('switch to the small drawing at small sizes and apply a hue shift', () => {
    const small = renderToStaticMarkup(createElement(Mascot, { size: 24 }));
    const large = renderToStaticMarkup(createElement(Mascot, { size: 120 }));
    expect(small.length).toBeLessThan(large.length);
    expect(renderToStaticMarkup(createElement(Mascot, { hue: 12 }))).toContain('hue-rotate(12deg)');
  });

  it('render the coin', () => {
    expect(renderToStaticMarkup(createElement(Coin, { size: 32, spin: true }))).toContain('coin-spin');
  });
});
