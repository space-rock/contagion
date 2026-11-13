import { describe, expect, it } from 'vitest';
import type { ChainEvent } from '../types';
import { describeEvent, describeEventShort, eventKind, eventTime } from './events';

const entry = (event: ChainEvent['event']): ChainEvent => ({ index: 0, at_ns: '1700000000000000000', event });
const FLU = (n: number) => (BigInt(n) * 10n ** 18n).toString();

describe('describeEvent', () => {
  it('describes a catch with generation, mint and fee', () => {
    const { who, text } = describeEvent(
      entry({ event: 'catch', carrier: 'zero.testnet', target: 'alice.testnet', gen: 2, mint: FLU(360), bonus: FLU(108), fee: '18000000000000000000000' }),
    );
    expect(who).toBe('alice.testnet');
    expect(text).toBe('caught it from zero.testnet as generation 2, minting 360 FLU for 0.018 NEAR');
  });

  it('distinguishes timely and late recovery', () => {
    expect(describeEvent(entry({ event: 'recover', account: 'a.testnet', burned: FLU(60), expired: false })).text).toBe('recovered in time and burned 60 FLU');
    expect(describeEvent(entry({ event: 'recover', account: 'a.testnet', burned: FLU(300), expired: true })).text).toBe('recovered late and burned 300 FLU');
  });

  it('credits the plague doctor on a cull', () => {
    const { who, text } = describeEvent(entry({ event: 'cull', account: 'a.testnet', caller: 'doc.testnet', burned: FLU(270), cut: FLU(30) }));
    expect(who).toBe('doc.testnet');
    expect(text).toBe('culled a.testnet, burning 270 FLU and keeping 30 FLU');
  });

  it('describes vaccination, claims and the season end', () => {
    expect(describeEvent(entry({ event: 'vaccinate', account: 'v.testnet', by: 'a.testnet', cost: FLU(20) })).text).toBe('vaccinated v.testnet for 20 FLU');
    expect(describeEvent(entry({ event: 'claim', account: 'a.testnet', amount: FLU(550) })).text).toBe('claimed 550 FLU from the reserve');
    expect(describeEvent(entry({ event: 'season_end', at_ns: '1', reason: 'herd' })).text).toBe('ended by herd immunity');
    expect(describeEvent(entry({ event: 'season_end', at_ns: '1', reason: 'time' })).text).toBe('ended when time ran out');
  });

  it('exposes the kind and a real date', () => {
    expect(eventKind(entry({ event: 'claim', account: 'a.testnet', amount: '1' }))).toBe('claim');
    expect(eventTime(entry({ event: 'claim', account: 'a.testnet', amount: '1' })).getUTCFullYear()).toBe(2023);
  });
});

describe('describeEventShort', () => {
  it('keeps every event to one short line', () => {
    expect(describeEventShort(entry({ event: 'catch', carrier: 'zero.testnet', target: 'alice.testnet', gen: 2, mint: FLU(360), bonus: FLU(108), fee: '1' }))).toEqual({
      who: 'alice.testnet',
      text: 'caught it from zero.testnet',
    });
    expect(describeEventShort(entry({ event: 'recover', account: 'a.testnet', burned: FLU(60), expired: false })).text).toBe('recovered');
    expect(describeEventShort(entry({ event: 'recover', account: 'a.testnet', burned: FLU(60), expired: true })).text).toBe('recovered late');
    expect(describeEventShort(entry({ event: 'cull', account: 'a.testnet', caller: 'doc.testnet', burned: FLU(1), cut: FLU(1) }))).toEqual({
      who: 'doc.testnet',
      text: 'culled a.testnet',
    });
    expect(describeEventShort(entry({ event: 'vaccinate', account: 'v.testnet', by: 'a.testnet', cost: FLU(20) })).text).toBe('vaccinated v.testnet');
    expect(describeEventShort(entry({ event: 'claim', account: 'a.testnet', amount: FLU(550) })).text).toBe('claimed 550 FLU');
    expect(describeEventShort(entry({ event: 'season_end', at_ns: '1', reason: 'herd' }))).toEqual({ who: 'The season', text: 'ended' });
  });

  it('is shorter than the full description', () => {
    const catchEvent = entry({ event: 'catch', carrier: 'zero.testnet', target: 'alice.testnet', gen: 2, mint: FLU(360), bonus: FLU(108), fee: '18000000000000000000000' });
    expect(describeEventShort(catchEvent).text.length).toBeLessThan(describeEvent(catchEvent).text.length);
  });
});
