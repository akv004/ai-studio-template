import { create } from 'zustand';
import type {
    Agent, Session, Run, Message,
    Event as StudioEvent, SessionStats,
    CreateAgentRequest, SendMessageResponse,
} from '@ai-studio/shared';

// ============================================
// APP STATE STORE
// Real Tauri IPC — no more mocks
// ============================================

export type ModuleId =
    | 'agents'
    | 'sessions'
    | 'runs'
    | 'inspector'
    | 'settings';

export type { Agent, Session, Run, Message };
export type { StudioEvent, SessionStats };

// Lazy-load Tauri invoke to work in both desktop and browser dev
async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
    const { invoke: tauriInvoke } = await import('@tauri-apps/api/core');
    return tauriInvoke<T>(cmd, args);
}

interface AppState {
    // Navigation
    activeModule: ModuleId;
    setActiveModule: (module: ModuleId) => void;

    // Command Palette
    isCommandPaletteOpen: boolean;
    openCommandPalette: () => void;
    closeCommandPalette: () => void;
    toggleCommandPalette: () => void;

    // Agents
    agents: Agent[];
    agentsLoading: boolean;
    fetchAgents: () => Promise<void>;
    createAgent: (req: CreateAgentRequest) => Promise<Agent>;
    deleteAgent: (id: string) => Promise<void>;

    // Sessions
    sessions: Session[];
    sessionsLoading: boolean;
    fetchSessions: () => Promise<void>;
    createSession: (agentId: string, title?: string) => Promise<Session>;
    deleteSession: (id: string) => Promise<void>;

    // Messages (for active session)
    messages: Message[];
    messagesLoading: boolean;
    fetchMessages: (sessionId: string) => Promise<void>;
    sendMessage: (sessionId: string, content: string) => Promise<SendMessageResponse>;
    sending: boolean;

    // Events (for inspector)
    events: StudioEvent[];
    eventsLoading: boolean;
    sessionStats: SessionStats | null;
    fetchEvents: (sessionId: string) => Promise<void>;
    fetchSessionStats: (sessionId: string) => Promise<void>;

    // Runs
    runs: Run[];
    runsLoading: boolean;
    fetchRuns: () => Promise<void>;

    // System Info (from Tauri)
    systemInfo: { platform: string; version: string } | null;
    setSystemInfo: (info: { platform: string; version: string }) => void;

    // Settings (provider keys, preferences)
    settings: Record<string, string>;
    settingsLoading: boolean;
    fetchSettings: () => Promise<void>;
    saveSetting: (key: string, value: string) => Promise<void>;

    // Inspector navigation (set by Sessions "Inspect" button)
    inspectorSessionId: string | null;
    openInspector: (sessionId: string) => void;
    clearInspectorSession: () => void;

    // Error tracking
    error: string | null;
    clearError: () => void;
}

export const useAppStore = create<AppState>((set, get) => ({
    // Navigation
    activeModule: 'agents',
    setActiveModule: (module) => set({ activeModule: module }),

    // Command Palette
    isCommandPaletteOpen: false,
    openCommandPalette: () => set({ isCommandPaletteOpen: true }),
    closeCommandPalette: () => set({ isCommandPaletteOpen: false }),
    toggleCommandPalette: () => set((s) => ({ isCommandPaletteOpen: !s.isCommandPaletteOpen })),

    // Agents
    agents: [],
    agentsLoading: false,
    fetchAgents: async () => {
        set({ agentsLoading: true, error: null });
        try {
            const agents = await invoke<Agent[]>('list_agents');
            set({ agents, agentsLoading: false });
        } catch (e) {
            set({ agentsLoading: false, error: `Failed to load agents: ${e}` });
        }
    },
    createAgent: async (req) => {
        const agent = await invoke<Agent>('create_agent', { agent: req });
        set((s) => ({ agents: [agent, ...s.agents] }));
        return agent;
    },
    deleteAgent: async (id) => {
        await invoke<void>('delete_agent', { id });
        set((s) => ({ agents: s.agents.filter((a) => a.id !== id) }));
    },

    // Sessions
    sessions: [],
    sessionsLoading: false,
    fetchSessions: async () => {
        set({ sessionsLoading: true, error: null });
        try {
            const sessions = await invoke<Session[]>('list_sessions');
            set({ sessions, sessionsLoading: false });
        } catch (e) {
            set({ sessionsLoading: false, error: `Failed to load sessions: ${e}` });
        }
    },
    createSession: async (agentId, title) => {
        set({ error: null });
        try {
            const args: Record<string, unknown> = { agentId };
            if (title) args.title = title;
            const session = await invoke<Session>('create_session', args);
            set((s) => ({ sessions: [session, ...s.sessions] }));
            return session;
        } catch (e) {
            const msg = `Failed to create session: ${e}`;
            set({ error: msg });
            throw e;
        }
    },
    deleteSession: async (id) => {
        await invoke<void>('delete_session', { id });
        set((s) => ({
            sessions: s.sessions.filter((sess) => sess.id !== id),
            messages: [],
        }));
    },

    // Messages
    messages: [],
    messagesLoading: false,
    sending: false,
    fetchMessages: async (sessionId) => {
        set({ messagesLoading: true });
        try {
            const messages = await invoke<Message[]>('get_session_messages', { sessionId });
            set({ messages, messagesLoading: false });
        } catch (e) {
            set({ messagesLoading: false, error: `Failed to load messages: ${e}` });
        }
    },
    sendMessage: async (sessionId, content) => {
        set({ sending: true, error: null });
        try {
            const resp = await invoke<SendMessageResponse>('send_message', {
                request: { sessionId, content },
            });
            set((s) => ({
                messages: [...s.messages, resp.userMessage, resp.assistantMessage],
                sending: false,
            }));
            // Refresh sessions to update message counts
            get().fetchSessions();
            return resp;
        } catch (e) {
            set({ sending: false, error: `Send failed: ${e}` });
            throw e;
        }
    },

    // Events (Inspector)
    events: [],
    eventsLoading: false,
    sessionStats: null,
    fetchEvents: async (sessionId) => {
        set({ eventsLoading: true });
        try {
            const events = await invoke<StudioEvent[]>('get_session_events', { sessionId });
            set({ events, eventsLoading: false });
        } catch (e) {
            set({ eventsLoading: false, error: `Failed to load events: ${e}` });
        }
    },
    fetchSessionStats: async (sessionId) => {
        try {
            const stats = await invoke<SessionStats>('get_session_stats', { sessionId });
            set({ sessionStats: stats });
        } catch (e) {
            set({ error: `Failed to load stats: ${e}` });
        }
    },

    // Runs
    runs: [],
    runsLoading: false,
    fetchRuns: async () => {
        set({ runsLoading: true, error: null });
        try {
            const runs = await invoke<Run[]>('list_runs');
            set({ runs, runsLoading: false });
        } catch (e) {
            set({ runsLoading: false, error: `Failed to load runs: ${e}` });
        }
    },

    // Settings
    settings: {},
    settingsLoading: false,
    fetchSettings: async () => {
        set({ settingsLoading: true });
        try {
            const settings = await invoke<Record<string, string>>('get_all_settings');
            set({ settings, settingsLoading: false });
        } catch (e) {
            set({ settingsLoading: false, error: `Failed to load settings: ${e}` });
        }
    },
    saveSetting: async (key, value) => {
        await invoke<void>('set_setting', { key, value });
        set((s) => ({ settings: { ...s.settings, [key]: value } }));
    },

    // System Info
    systemInfo: null,
    setSystemInfo: (info) => set({ systemInfo: info }),

    // Inspector navigation
    inspectorSessionId: null,
    openInspector: (sessionId) => set({ inspectorSessionId: sessionId, activeModule: 'inspector' }),
    clearInspectorSession: () => set({ inspectorSessionId: null }),

    // Error
    error: null,
    clearError: () => set({ error: null }),
}));
