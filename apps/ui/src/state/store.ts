import { create } from 'zustand';
import type { Project, Agent, TrainingRun } from '@ai-studio/shared';
import { mockProjects, mockAgents, mockTrainingRuns, mockRunPhases } from '../fixtures/mocks';
import type { RunPhase } from '../fixtures/mocks';

// ============================================
// APP STATE STORE
// Central state management for the AI Studio
// ============================================

export type ModuleId =
    | 'projects'
    | 'vision'
    | 'audio'
    | 'agents'
    | 'training'
    | 'runs'
    | 'settings';

// Re-export shared types for convenience
export type { Project, Agent, TrainingRun };
export type { RunPhase };

interface AppState {
    // Navigation
    activeModule: ModuleId;
    setActiveModule: (module: ModuleId) => void;

    // Command Palette
    isCommandPaletteOpen: boolean;
    openCommandPalette: () => void;
    closeCommandPalette: () => void;
    toggleCommandPalette: () => void;

    // Projects
    projects: Project[];
    selectedProjectId: string | null;
    selectProject: (id: string | null) => void;

    // Agents
    agents: Agent[];

    // Training
    trainingRuns: TrainingRun[];

    // Runs/Timeline
    runPhases: RunPhase[];

    // System Info (from Tauri)
    systemInfo: {
        platform: string;
        version: string;
    } | null;
    setSystemInfo: (info: { platform: string; version: string }) => void;
}

export const useAppStore = create<AppState>((set) => ({
    // Navigation
    activeModule: 'projects',
    setActiveModule: (module) => set({ activeModule: module }),

    // Command Palette
    isCommandPaletteOpen: false,
    openCommandPalette: () => set({ isCommandPaletteOpen: true }),
    closeCommandPalette: () => set({ isCommandPaletteOpen: false }),
    toggleCommandPalette: () => set((state) => ({ isCommandPaletteOpen: !state.isCommandPaletteOpen })),

    // Projects - use imported fixtures
    projects: mockProjects,
    selectedProjectId: null,
    selectProject: (id) => set({ selectedProjectId: id }),

    // Agents - use imported fixtures
    agents: mockAgents,

    // Training - use imported fixtures
    trainingRuns: mockTrainingRuns,

    // Runs/Timeline - use imported fixtures
    runPhases: mockRunPhases,

    // System Info
    systemInfo: null,
    setSystemInfo: (info) => set({ systemInfo: info }),
}));
