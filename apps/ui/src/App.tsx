
import { AppShell } from './app/layout/AppShell';
import { CommandPalette } from './app/layout/CommandPalette';
import { ToolApprovalModal } from './app/layout/ToolApprovalModal';
import { Toasts } from './app/layout/Toasts';
import { useKeyboardShortcuts } from './commands';
import { useAppStore } from './state/store';
import { useEffect, useMemo, useState } from 'react';

// Page imports — 6 pillars + onboarding
import { AgentsPage } from './app/pages/AgentsPage';
import { SessionsPage } from './app/pages/SessionsPage';
import { RunsPage } from './app/pages/RunsPage';
import { InspectorPage } from './app/pages/InspectorPage';
import { NodeEditorPage } from './app/pages/NodeEditorPage';
import { SettingsPage } from './app/pages/SettingsPage';
import { WelcomePage } from './app/pages/WelcomePage';

/**
 * Main Application Component
 *
 * Renders the app shell with dynamic page content based on active module.
 * Implements global keyboard shortcuts and command palette.
 */
function App() {
  const { activeModule, isCommandPaletteOpen, agents, agentsLoading, fetchAgents, settings, fetchSettings } = useAppStore();
  const [toolApprovalQueue, setToolApprovalQueue] = useState<
    Array<{ id: string; method: string; path: string; body?: unknown }>
  >([]);
  const [showOnboarding, setShowOnboarding] = useState(false);
  const [initialLoadDone, setInitialLoadDone] = useState(false);

  const activeToolApproval = useMemo(() => toolApprovalQueue[0] ?? null, [toolApprovalQueue]);

  // Initialize keyboard shortcuts
  useKeyboardShortcuts();

  const pushEvent = useAppStore((s) => s.pushEvent);

  // Load agents + settings on mount to detect first-run
  useEffect(() => {
    Promise.all([fetchAgents(), fetchSettings()]).then(() => {
      setInitialLoadDone(true);
    });
  }, [fetchAgents, fetchSettings]);

  // Detect first-run: no agents AND onboarding not yet completed
  useEffect(() => {
    if (!initialLoadDone || agentsLoading) return;
    const onboardingDone = settings['onboarding.completed'] === 'true' || settings['onboarding.completed'] === '"true"';
    setShowOnboarding(agents.length === 0 && !onboardingDone);
  }, [initialLoadDone, agents, agentsLoading, settings]);

  // Listen for tool approval requests + live agent events from the desktop backend (Tauri only).
  useEffect(() => {
    let unlistenApproval: undefined | (() => void);
    let unlistenEvents: undefined | (() => void);

    (async () => {
      try {
        const { listen } = await import('@tauri-apps/api/event');
        unlistenApproval = await listen<{
          id: string;
          method: string;
          path: string;
          body?: unknown;
        }>('tool_approval_requested', (event) => {
          setToolApprovalQueue((q) => [...q, event.payload]);
        });

        // Live event stream from sidecar → Tauri → UI
        unlistenEvents = await listen<{
          event_id: string;
          type: string;
          ts: string;
          session_id: string;
          source: string;
          seq: number;
          payload: Record<string, unknown>;
          cost_usd: number | null;
        }>('agent_event', (tauriEvent) => {
          const e = tauriEvent.payload;
          pushEvent({
            eventId: e.event_id,
            type: e.type,
            ts: e.ts,
            sessionId: e.session_id,
            source: e.source,
            seq: e.seq,
            payload: e.payload,
            costUsd: e.cost_usd,
          });
        });
      } catch {
        // Not running under Tauri (or events unavailable).
      }
    })();

    return () => {
      try {
        unlistenApproval?.();
        unlistenEvents?.();
      } catch {
        // ignore
      }
    };
  }, [pushEvent]);

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
    if (showOnboarding) {
      return <WelcomePage onComplete={() => {
        setShowOnboarding(false);
        fetchAgents();
      }} />;
    }

    switch (activeModule) {
      case 'agents':
        return <AgentsPage />;
      case 'sessions':
        return <SessionsPage />;
      case 'runs':
        return <RunsPage />;
      case 'inspector':
        return <InspectorPage />;
      case 'workflows':
        return <NodeEditorPage />;
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

      {/* Toast Notifications */}
      <Toasts />
    </>
  );
}

export default App;
