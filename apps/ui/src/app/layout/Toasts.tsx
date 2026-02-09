import { X, CheckCircle, AlertCircle, Info } from 'lucide-react';
import { useAppStore, type Toast } from '../../state/store';

const icons: Record<Toast['type'], React.ElementType> = {
    success: CheckCircle,
    error: AlertCircle,
    info: Info,
};

const colors: Record<Toast['type'], string> = {
    success: 'border-green-500/40 bg-green-500/10 text-green-400',
    error: 'border-red-500/40 bg-red-500/10 text-red-400',
    info: 'border-blue-500/40 bg-blue-500/10 text-blue-400',
};

/**
 * Toast notification container — renders in top-right corner.
 * Per ui-design.md spec: 200ms slide-in from top-right, auto-dismiss.
 */
export function Toasts() {
    const toasts = useAppStore((s) => s.toasts);
    const removeToast = useAppStore((s) => s.removeToast);

    if (toasts.length === 0) return null;

    return (
        <div className="fixed top-3 right-3 z-50 flex flex-col gap-2 max-w-sm">
            {toasts.map((toast) => {
                const Icon = icons[toast.type];
                return (
                    <div
                        key={toast.id}
                        className={`flex items-start gap-2 px-3 py-2.5 rounded-lg border text-sm shadow-lg animate-slide-in-right ${colors[toast.type]}`}
                    >
                        <Icon className="w-4 h-4 mt-0.5 shrink-0" />
                        <span className="flex-1 break-words">{toast.message}</span>
                        <button
                            className="shrink-0 p-0.5 opacity-60 hover:opacity-100 transition-opacity"
                            onClick={() => removeToast(toast.id)}
                        >
                            <X className="w-3.5 h-3.5" />
                        </button>
                    </div>
                );
            })}
        </div>
    );
}
