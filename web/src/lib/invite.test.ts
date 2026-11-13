import { afterEach, describe, expect, it, vi } from 'vitest';
import { inviteLink, readParams, shareText, shareUrl } from './invite';

afterEach(() => {
  vi.unstubAllGlobals();
});

const at = (search: string, hash: string) =>
  vi.stubGlobal('window', { location: { search, hash, origin: 'https://flu.example', pathname: '/game/' } });

describe('invite links', () => {
  it('reads legacy query parameters', () => {
    at('?c=contagion-live.testnet&from=alice.testnet', '');
    expect(readParams().get('from')).toBe('alice.testnet');
    expect(readParams().get('c')).toBe('contagion-live.testnet');
  });

  it('reads parameters inside the hash route and lets them win', () => {
    at('?from=old.testnet&c=x.testnet', '#/outbreak?from=new.testnet&page=2');
    const params = readParams();
    expect(params.get('from')).toBe('new.testnet');
    expect(params.get('c')).toBe('x.testnet');
    expect(params.get('page')).toBe('2');
  });

  it('handles a hash without parameters', () => {
    at('', '#/activity');
    expect([...readParams().keys()]).toEqual([]);
  });

  it('builds a shareable hash link', () => {
    at('', '');
    expect(inviteLink('contagion-live.testnet', 'alice.testnet')).toBe('https://flu.example/game/#/?c=contagion-live.testnet&from=alice.testnet');
  });

  it('builds the X share text with the hackathon tags', () => {
    const text = shareText('https://flu.example/#/?from=a');
    expect(text).toContain('@NEARProtocol');
    expect(text).toContain('#NEARHackathon');
    expect(shareUrl('https://flu.example/#/?from=a').startsWith('https://x.com/intent/post?text=')).toBe(true);
  });
});
