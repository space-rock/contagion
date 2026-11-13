import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { MOODS, coinSvg, mascotSvg } from '../src/brand/art.ts';

const root = join(dirname(fileURLToPath(import.meta.url)), '../..');
const brand = join(root, 'brand');
mkdirSync(brand, { recursive: true });
mkdirSync(join(root, 'web/public'), { recursive: true });

for (const mood of MOODS) {
  writeFileSync(join(brand, `flu-${mood}.svg`), mascotSvg(mood, 'full') + '\n');
}
writeFileSync(join(brand, 'flu-small.svg'), mascotSvg('contagious', 'small') + '\n');
writeFileSync(join(brand, 'coin.svg'), coinSvg() + '\n');
writeFileSync(join(root, 'web/public/favicon.svg'), mascotSvg('contagious', 'small') + '\n');
console.log('exported brand svgs to', brand);
