import { useEffect, useCallback } from 'react';
import { useAppStore, type ModuleId } from '../state/store';

// ============================================
// COMMAND DEFINITIONS
// Central registry of all available commands
// ============================================

export interface Command {
    id: string;
    label: string;
    shortcut?: string;
    icon?: string;
    action: () => void;
    category: 'navigation' | 'action' | 'view';
}

export function useCommands(): Command[] {
    const { setActiveModule, toggleCommandPalette } = useAppStore();

    const navigateTo = useCallback((module: ModuleId) => {
        setActiveModule(module);
    }, [setActiveModule]);

    return [
        // Navigation Commands — 5 pillars
        {
            id: 'nav-agents',
            label: 'Go to Agents',
            shortcut: '⌘1',
            category: 'navigation',
            action: () => navigateTo('agents'),
        },
        {
            id: 'nav-sessions',
            label: 'Go to Sessions',
            shortcut: '⌘2',
            category: 'navigation',
            action: () => navigateTo('sessions'),
        },
        {
            id: 'nav-runs',
            label: 'Go to Runs',
            shortcut: '⌘3',
            category: 'navigation',
            action: () => navigateTo('runs'),
        },
        {
            id: 'nav-inspector',
            label: 'Go to Inspector',
            shortcut: '⌘4',
            category: 'navigation',
            action: () => navigateTo('inspector'),
        },
        {
            id: 'nav-settings',
            label: 'Go to Settings',
            shortcut: '⌘,',
            category: 'navigation',
            action: () => navigateTo('settings'),
        },
        // Action Commands
        {
            id: 'new-agent',
            label: 'New Agent',
            shortcut: '⌘N',
            category: 'action',
            action: () => {
                console.log('Creating new agent...');
                navigateTo('agents');
            },
        },
        {
            id: 'new-session',
            label: 'New Session',
            shortcut: '⌘⇧N',
            category: 'action',
            action: () => {
                console.log('Creating new session...');
                navigateTo('sessions');
            },
        },
        // View Commands
        {
            id: 'toggle-command-palette',
            label: 'Toggle Command Palette',
            shortcut: '⌘K',
            category: 'view',
            action: () => toggleCommandPalette(),
        },
    ];
}

// ============================================
// KEYBOARD SHORTCUTS HOOK
// Global keyboard shortcut handling
// ============================================

export function useKeyboardShortcuts() {
    const {
        setActiveModule,
        toggleCommandPalette,
        closeCommandPalette,
        isCommandPaletteOpen
    } = useAppStore();

    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            const isMeta = e.metaKey || e.ctrlKey;

            // Command Palette toggle
            if (isMeta && e.key === 'k') {
                e.preventDefault();
                toggleCommandPalette();
                return;
            }

            // Escape to close command palette
            if (e.key === 'Escape' && isCommandPaletteOpen) {
                e.preventDefault();
                closeCommandPalette();
                return;
            }

            // Navigation shortcuts (only when command palette is closed)
            if (isMeta && !isCommandPaletteOpen) {
                const shortcuts: Record<string, ModuleId> = {
                    '1': 'agents',
                    '2': 'sessions',
                    '3': 'runs',
                    '4': 'inspector',
                    ',': 'settings',
                };

                if (shortcuts[e.key]) {
                    e.preventDefault();
                    setActiveModule(shortcuts[e.key]);
                }
            }
        };

        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [setActiveModule, toggleCommandPalette, closeCommandPalette, isCommandPaletteOpen]);
}
