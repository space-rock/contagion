import { GAS_TGAS } from '../config';

export type Call = { method: string; args?: object; deposit?: bigint };

export const buildActions = (calls: Call[]) =>
  calls.map((call) => ({
    type: 'FunctionCall' as const,
    params: {
      methodName: call.method,
      args: call.args ?? {},
      gas: (BigInt(GAS_TGAS) * 10n ** 12n).toString(),
      deposit: (call.deposit ?? 0n).toString(),
    },
  }));
