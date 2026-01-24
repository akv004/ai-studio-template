import { ReactNode } from 'react';
import { Sidebar } from './Sidebar';
import { Header } from './Header';

interface AppShellProps {
    children: ReactNode;
}

/**
 * Application Shell Layout
 * 
 * Professional desktop application layout with:
 * - Draggable title bar (macOS-style)
 * - Fixed sidebar navigation
 * - Main content area
 */
export function AppShell({ children }: AppShellProps) {
    return (
        <div className="app-shell">
            <Header />
            <div className="app-body">
                <Sidebar />
                <main className="app-main">
                    <div className="app-content">
                        {children}
                    </div>
                </main>
            </div>
        </div>
    );
}
