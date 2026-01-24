import { Search, Command } from 'lucide-react';
import { useAppStore } from '../../state/store';

/**
 * Application Header
 * 
 * Contains:
 * - App title/branding
 * - Search/command palette trigger
 * - Window controls (handled by Tauri)
 */
export function Header() {
    const { openCommandPalette, systemInfo } = useAppStore();

    return (
        <header className="app-header">
            <div className="flex items-center gap-3">
                <div className="flex items-center gap-2">
                    <div
                        className="w-6 h-6 rounded-md flex items-center justify-center"
                        style={{
                            background: 'linear-gradient(135deg, #8b5cf6 0%, #a78bfa 100%)',
                            boxShadow: '0 0 12px rgba(139, 92, 246, 0.4)'
                        }}
                    >
                        <span className="text-white text-xs font-bold">AI</span>
                    </div>
                    <span className="font-semibold text-sm">AI Studio</span>
                </div>
            </div>

            {/* Command Palette Trigger */}
            <button
                className="flex items-center gap-2 px-3 py-1.5 rounded-md bg-[var(--bg-tertiary)] hover:bg-[var(--bg-hover)] transition-colors"
                onClick={openCommandPalette}
            >
                <Search className="w-4 h-4 text-[var(--text-muted)]" />
                <span className="text-sm text-[var(--text-muted)]">Search...</span>
                <div className="flex items-center gap-1 ml-4">
                    <kbd className="px-1.5 py-0.5 text-xs rounded bg-[var(--bg-elevated)] text-[var(--text-muted)]">
                        <Command className="w-3 h-3 inline" />
                    </kbd>
                    <kbd className="px-1.5 py-0.5 text-xs rounded bg-[var(--bg-elevated)] text-[var(--text-muted)]">
                        K
                    </kbd>
                </div>
            </button>

            {/* System Info Badge */}
            <div className="flex items-center gap-2">
                {systemInfo && (
                    <span className="text-xs text-[var(--text-muted)]">
                        {systemInfo.platform}
                    </span>
                )}
                <div className="status-pill status-success">
                    <span className="status-dot" />
                    Ready
                </div>
            </div>
        </header>
    );
}
