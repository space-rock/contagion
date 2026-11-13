import { AlertTriangle, CheckCircle2, Info, X } from 'lucide-react';
import { useToasts } from '../state/toasts';

const ICONS = { success: CheckCircle2, info: Info, error: AlertTriangle } as const;

export const Toasts = () => {
  const { toasts, dismiss } = useToasts();
  return (
    <div className="toasts" role="status" aria-live="polite">
      {toasts.map((toast) => {
        const Icon = ICONS[toast.tone];
        return (
          <div key={toast.id} className={`toast toast-${toast.tone}`}>
            <Icon size={18} aria-hidden="true" />
            <span>{toast.text}</span>
            <button className="toast-close" onClick={() => dismiss(toast.id)} aria-label="Dismiss">
              <X size={16} />
            </button>
          </div>
        );
      })}
    </div>
  );
};
