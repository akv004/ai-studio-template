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
        // Navigation Commands
        {
            id: 'nav-projects',
            label: 'Go to Projects',
            shortcut: '⌘1',
            category: 'navigation',
            action: () => navigateTo('projects'),
        },
        {
            id: 'nav-vision',
            label: 'Go to Vision',
            shortcut: '⌘2',
            category: 'navigation',
            action: () => navigateTo('vision'),
        },
        {
            id: 'nav-audio',
            label: 'Go to Audio',
            shortcut: '⌘3',
            category: 'navigation',
            action: () => navigateTo('audio'),
        },
        {
            id: 'nav-agents',
            label: 'Go to Agents',
            shortcut: '⌘4',
            category: 'navigation',
            action: () => navigateTo('agents'),
        },
        {
            id: 'nav-training',
            label: 'Go to Training',
            shortcut: '⌘5',
            category: 'navigation',
            action: () => navigateTo('training'),
        },
        {
            id: 'nav-runs',
            label: 'Go to Runs',
            shortcut: '⌘6',
            category: 'navigation',
            action: () => navigateTo('runs'),
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
            id: 'new-project',
            label: 'New Project',
            shortcut: '⌘N',
            category: 'action',
            action: () => {
                console.log('Creating new project...');
                navigateTo('projects');
            },
        },
        {
            id: 'start-training',
            label: 'Start Training Run',
            shortcut: '⌘⇧T',
            category: 'action',
            action: () => {
                console.log('Starting training...');
                navigateTo('training');
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
                    '1': 'projects',
                    '2': 'vision',
                    '3': 'audio',
                    '4': 'agents',
                    '5': 'training',
                    '6': 'runs',
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
