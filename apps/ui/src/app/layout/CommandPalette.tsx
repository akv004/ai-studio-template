import { useState, useEffect, useRef } from 'react';
import { useAppStore } from '../../state/store';
import { useCommands, type Command } from '../../commands';
import {
    FolderOpen,
    Eye,
    AudioLines,
    Bot,
    Dumbbell,
    Play,
    Settings,
    Search,
    Plus,
    Zap
} from 'lucide-react';

const iconMap: Record<string, React.ElementType> = {
    'nav-projects': FolderOpen,
    'nav-vision': Eye,
    'nav-audio': AudioLines,
    'nav-agents': Bot,
    'nav-training': Dumbbell,
    'nav-runs': Play,
    'nav-settings': Settings,
    'new-project': Plus,
    'start-training': Zap,
};

/**
 * Command Palette
 * 
 * Keyboard-driven command interface (⌘K)
 * Features:
 * - Fuzzy search
 * - Keyboard navigation
 * - Action execution
 */
export function CommandPalette() {
    const { closeCommandPalette } = useAppStore();
    const commands = useCommands();
    const [query, setQuery] = useState('');
    const [selectedIndex, setSelectedIndex] = useState(0);
    const inputRef = useRef<HTMLInputElement>(null);

    // Filter commands based on query
    const filteredCommands = commands.filter(cmd =>
        cmd.label.toLowerCase().includes(query.toLowerCase())
    );

    // Focus input on mount
    useEffect(() => {
        inputRef.current?.focus();
    }, []);

    // Reset selection when query changes
    useEffect(() => {
        setSelectedIndex(0);
    }, [query]);

    // Keyboard navigation
    const handleKeyDown = (e: React.KeyboardEvent) => {
        switch (e.key) {
            case 'ArrowDown':
                e.preventDefault();
                setSelectedIndex(i => Math.min(i + 1, filteredCommands.length - 1));
                break;
            case 'ArrowUp':
                e.preventDefault();
                setSelectedIndex(i => Math.max(i - 1, 0));
                break;
            case 'Enter':
                e.preventDefault();
                if (filteredCommands[selectedIndex]) {
                    executeCommand(filteredCommands[selectedIndex]);
                }
                break;
            case 'Escape':
                e.preventDefault();
                closeCommandPalette();
                break;
        }
    };

    const executeCommand = (command: Command) => {
        command.action();
        closeCommandPalette();
    };

    return (
        <div
            className="command-palette-overlay animate-fade-in"
            onClick={(e) => {
                if (e.target === e.currentTarget) closeCommandPalette();
            }}
        >
            <div className="command-palette animate-slide-up">
                {/* Search Input */}
                <div className="flex items-center gap-3 px-4 py-3 border-b border-[var(--border-subtle)]">
                    <Search className="w-5 h-5 text-[var(--text-muted)]" />
                    <input
                        ref={inputRef}
                        type="text"
                        className="command-input flex-1 p-0 border-0"
                        placeholder="Type a command or search..."
                        value={query}
                        onChange={(e) => setQuery(e.target.value)}
                        onKeyDown={handleKeyDown}
                    />
                </div>

                {/* Command List */}
                <div className="command-list">
                    {filteredCommands.length === 0 ? (
                        <div className="p-4 text-center text-[var(--text-muted)]">
                            No commands found
                        </div>
                    ) : (
                        filteredCommands.map((cmd, index) => {
                            const Icon = iconMap[cmd.id] || Zap;
                            return (
                                <div
                                    key={cmd.id}
                                    className={`command-item ${index === selectedIndex ? 'selected' : ''}`}
                                    onClick={() => executeCommand(cmd)}
                                    onMouseEnter={() => setSelectedIndex(index)}
                                >
                                    <Icon className="w-4 h-4" />
                                    <span>{cmd.label}</span>
                                    {cmd.shortcut && (
                                        <span className="command-shortcut">{cmd.shortcut}</span>
                                    )}
                                </div>
                            );
                        })
                    )}
                </div>
            </div>
        </div>
    );
}
