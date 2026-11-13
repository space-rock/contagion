import { type ReactNode, createContext, useCallback, useContext, useEffect, useMemo, useReducer, useRef } from 'react';

export type ToastTone = 'success' | 'info' | 'error';
export type Toast = { id: number; tone: ToastTone; text: string };
export type ToastAction = { type: 'add'; toast: Toast } | { type: 'dismiss'; id: number } | { type: 'clear' };

export const MAX_TOASTS = 3;

export const toastReducer = (state: Toast[], action: ToastAction): Toast[] => {
  switch (action.type) {
    case 'add':
      return [...state.slice(-(MAX_TOASTS - 1)), action.toast];
    case 'dismiss':
      return state.filter((toast) => toast.id !== action.id);
    case 'clear':
      return [];
  }
};

const LIFETIME: Record<ToastTone, number> = { success: 4500, info: 4500, error: 9000 };

type Toasts = {
  toasts: Toast[];
  push: (tone: ToastTone, text: string) => void;
  dismiss: (id: number) => void;
  clear: () => void;
};

const ToastContext = createContext<Toasts | null>(null);

export const ToastProvider = ({ children }: { children: ReactNode }) => {
  const [toasts, dispatch] = useReducer(toastReducer, []);
  const nextId = useRef(1);
  const timers = useRef(new Map<number, ReturnType<typeof setTimeout>>());

  const dismiss = useCallback((id: number) => {
    const timer = timers.current.get(id);
    if (timer) clearTimeout(timer);
    timers.current.delete(id);
    dispatch({ type: 'dismiss', id });
  }, []);

  const push = useCallback(
    (tone: ToastTone, text: string) => {
      const id = nextId.current;
      nextId.current += 1;
      dispatch({ type: 'add', toast: { id, tone, text } });
      timers.current.set(
        id,
        setTimeout(() => dismiss(id), LIFETIME[tone]),
      );
    },
    [dismiss],
  );

  const clear = useCallback(() => {
    timers.current.forEach((timer) => clearTimeout(timer));
    timers.current.clear();
    dispatch({ type: 'clear' });
  }, []);

  useEffect(() => () => timers.current.forEach((timer) => clearTimeout(timer)), []);

  const value = useMemo(() => ({ toasts, push, dismiss, clear }), [toasts, push, dismiss, clear]);
  return <ToastContext.Provider value={value}>{children}</ToastContext.Provider>;
};

export const useToasts = (): Toasts => {
  const value = useContext(ToastContext);
  if (!value) throw new Error('useToasts needs a ToastProvider');
  return value;
};

export const useClearToastsOnChange = (key: unknown): void => {
  const { clear } = useToasts();
  useEffect(() => {
    clear();
  }, [key, clear]);
};
