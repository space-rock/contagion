import { readdirSync, readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

const root = dirname(fileURLToPath(import.meta.url));

const sources = readdirSync(root, { recursive: true, encoding: 'utf8' })
  .filter((file) => /\.(ts|tsx)$/.test(file) && !/\.test\.tsx?$/.test(file))
  .map((file) => ({ file, text: readFileSync(join(root, file), 'utf8') }));

const BANNED = [
  'straight from the contract',
  'not from this page',
  'Read straight',
  'Four steps from healthy',
  'NEP-141',
  'decimals',
  'Locked (viral)',
  'own event log',
  'Has it been audited',
  'Are the deployer keys',
  'you have to catch it, ',
];

describe('interface copy', () => {
  it('scans the interface sources', () => {
    expect(sources.length).toBeGreaterThan(20);
  });

  it('keeps meta commentary and jargon out of the interface', () => {
    for (const phrase of BANNED) {
      const hits = sources.filter(({ text }) => text.includes(phrase)).map(({ file }) => file);
      expect({ phrase, hits }).toEqual({ phrase, hits: [] });
    }
  });

  it('has no em-dashes', () => {
    const hits = sources.filter(({ text }) => text.includes('—')).map(({ file }) => file);
    expect(hits).toEqual([]);
  });

  it('keeps the slogan in the hero only', () => {
    const hits = sources.filter(({ text }) => text.includes('Nobody can sell it to you')).map(({ file }) => file);
    expect(hits).toEqual([]);
  });
});
