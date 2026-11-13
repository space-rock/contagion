import { describe, expect, it } from 'vitest';
import { buildActions } from './calls';

describe('buildActions', () => {
  it('builds function calls with 60 Tgas and the exact deposit', () => {
    const [action] = buildActions([{ method: 'catch', args: { carrier: 'zero.testnet' }, deposit: 30_000_000_000_000_000_000_000n }]);
    expect(action).toEqual({
      type: 'FunctionCall',
      params: {
        methodName: 'catch',
        args: { carrier: 'zero.testnet' },
        gas: '60000000000000',
        deposit: '30000000000000000000000',
      },
    });
  });

  it('defaults to empty args and no deposit', () => {
    expect(buildActions([{ method: 'recover' }])[0].params).toMatchObject({ args: {}, deposit: '0' });
  });

  it('keeps several calls in order for one transaction', () => {
    const actions = buildActions([
      { method: 'storage_deposit', args: { registration_only: true }, deposit: 3_000_000_000_000_000_000_000n },
      { method: 'cull', args: { account: 'a.testnet' } },
    ]);
    expect(actions.map((a) => a.params.methodName)).toEqual(['storage_deposit', 'cull']);
    expect(actions[0].params.deposit).toBe('3000000000000000000000');
  });

  it('serialises BigInt deposits as strings so JSON encoding cannot fail', () => {
    expect(() => JSON.stringify(buildActions([{ method: 'catch', deposit: 10n ** 24n }]))).not.toThrow();
  });
});
