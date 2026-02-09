
import { AppShell } from './app/layout/AppShell';
import { CommandPalette } from './app/layout/CommandPalette';
import { ToolApprovalModal } from './app/layout/ToolApprovalModal';
import { useKeyboardShortcuts } from './commands';
import { useAppStore } from './state/store';
import { useEffect, useMemo, useState } from 'react';

// Page imports — 5 pillars
import { AgentsPage } from './app/pages/AgentsPage';
import { SessionsPage } from './app/pages/SessionsPage';
import { RunsPage } from './app/pages/RunsPage';
import { InspectorPage } from './app/pages/InspectorPage';
import { SettingsPage } from './app/pages/SettingsPage';

/**
 * Main Application Component
 *
 * Renders the app shell with dynamic page content based on active module.
 * Implements global keyboard shortcuts and command palette.
 */
function App() {
  const { activeModule, isCommandPaletteOpen } = useAppStore();
  const [toolApprovalQueue, setToolApprovalQueue] = useState<
    Array<{ id: string; method: string; path: string; body?: unknown }>
  >([]);

  const activeToolApproval = useMemo(() => toolApprovalQueue[0] ?? null, [toolApprovalQueue]);

  // Initialize keyboard shortcuts
  useKeyboardShortcuts();

  // Listen for tool approval requests from the desktop backend (Tauri only).
  useEffect(() => {
    let unlisten: undefined | (() => void);

    (async () => {
      try {
        const { listen } = await import('@tauri-apps/api/event');
        unlisten = await listen<{
          id: string;
          method: string;
          path: string;
          body?: unknown;
        }>('tool_approval_requested', (event) => {
          setToolApprovalQueue((q) => [...q, event.payload]);
        });
      } catch {
        // Not running under Tauri (or events unavailable).
      }
    })();

    return () => {
      try {
        unlisten?.();
      } catch {
        // ignore
      }
    };
  }, []);

  const decideToolApproval = async (approve: boolean) => {
    const request = toolApprovalQueue[0];
    if (!request) return;

    setToolApprovalQueue((q) => q.slice(1));

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('approve_tool_request', { id: request.id, approve });
    } catch {
      // If not running under Tauri or the request timed out, ignore.
    }
  };

  // Dynamic page rendering based on active module
  const renderPage = () => {
    switch (activeModule) {
      case 'agents':
        return <AgentsPage />;
      case 'sessions':
        return <SessionsPage />;
      case 'runs':
        return <RunsPage />;
      case 'inspector':
        return <InspectorPage />;
      case 'settings':
        return <SettingsPage />;
      default:
        return <AgentsPage />;
    }
  };

  return (
    <>
      <AppShell>
        {renderPage()}
      </AppShell>

      {/* Command Palette Overlay */}
      {isCommandPaletteOpen && <CommandPalette />}

      {/* Tool Approval Modal (Desktop only) */}
      {activeToolApproval && (
        <ToolApprovalModal
          request={activeToolApproval}
          onApprove={() => decideToolApproval(true)}
          onDeny={() => decideToolApproval(false)}
        />
      )}
    </>
  );
}

export default App;
