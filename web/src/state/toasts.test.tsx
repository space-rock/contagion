import { act, renderHook } from '@testing-library/react';
import { type ReactNode } from 'react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { MAX_TOASTS, ToastProvider, toastReducer, useClearToastsOnChange, useToasts } from './toasts';

const toast = (id: number) => ({ id, tone: 'info' as const, text: `t${id}` });

describe('toastReducer', () => {
  it('adds toasts and keeps only the newest few', () => {
    let state = toastReducer([], { type: 'add', toast: toast(1) });
    for (let id = 2; id <= MAX_TOASTS + 2; id += 1) state = toastReducer(state, { type: 'add', toast: toast(id) });
    expect(state).toHaveLength(MAX_TOASTS);
    expect(state.at(-1)?.id).toBe(MAX_TOASTS + 2);
  });

  it('dismisses one toast and clears all', () => {
    const state = [toast(1), toast(2)];
    expect(toastReducer(state, { type: 'dismiss', id: 1 }).map((t) => t.id)).toEqual([2]);
    expect(toastReducer(state, { type: 'clear' })).toEqual([]);
  });
});

describe('toast provider', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  const wrapper = ({ children }: { children: ReactNode }) => <ToastProvider>{children}</ToastProvider>;

  it('dismisses success toasts on its own and keeps errors longer', () => {
    const { result } = renderHook(() => useToasts(), { wrapper });
    act(() => {
      result.current.push('success', 'caught');
      result.current.push('error', 'failed');
    });
    expect(result.current.toasts).toHaveLength(2);
    act(() => {
      vi.advanceTimersByTime(5_000);
    });
    expect(result.current.toasts.map((t) => t.text)).toEqual(['failed']);
    act(() => {
      vi.advanceTimersByTime(5_000);
    });
    expect(result.current.toasts).toEqual([]);
  });

  it('clears every toast when the signed-in account changes or signs out', () => {
    const { result, rerender } = renderHook(
      ({ account }: { account: string | null }) => {
        useClearToastsOnChange(account);
        return useToasts();
      },
      { wrapper, initialProps: { account: 'alice.testnet' as string | null } },
    );
    act(() => result.current.push('success', 'You caught it'));
    expect(result.current.toasts).toHaveLength(1);

    rerender({ account: 'alice.testnet' });
    expect(result.current.toasts).toHaveLength(1);

    rerender({ account: null });
    expect(result.current.toasts).toEqual([]);

    act(() => result.current.push('success', 'again'));
    rerender({ account: 'bob.testnet' });
    expect(result.current.toasts).toEqual([]);
  });
});
