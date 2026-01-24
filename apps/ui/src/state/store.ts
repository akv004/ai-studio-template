import { create } from 'zustand';

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

export interface Project {
    id: string;
    name: string;
    description: string;
    createdAt: string;
    updatedAt: string;
    thumbnail?: string;
}

export interface Agent {
    id: string;
    name: string;
    status: 'running' | 'idle' | 'error' | 'offline';
    model: string;
    lastActive: string;
}

export interface TrainingRun {
    id: string;
    name: string;
    status: 'queued' | 'running' | 'completed' | 'failed';
    progress: number;
    epochs: number;
    currentEpoch: number;
    startedAt: string;
    dataset: string;
}

export interface RunPhase {
    id: string;
    name: string;
    status: 'pending' | 'running' | 'completed' | 'failed';
    startedAt?: string;
    completedAt?: string;
    logs: string[];
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

// Mock data for demonstration
const mockProjects: Project[] = [
    {
        id: '1',
        name: 'Object Detection Pipeline',
        description: 'Real-time object detection for autonomous navigation',
        createdAt: '2024-01-15T10:00:00Z',
        updatedAt: '2024-01-20T14:30:00Z',
    },
    {
        id: '2',
        name: 'Voice Assistant',
        description: 'Multi-language voice recognition and synthesis',
        createdAt: '2024-01-10T08:00:00Z',
        updatedAt: '2024-01-19T16:45:00Z',
    },
    {
        id: '3',
        name: 'Document Analyzer',
        description: 'OCR and NLP for document processing',
        createdAt: '2024-01-05T12:00:00Z',
        updatedAt: '2024-01-18T11:20:00Z',
    },
];

const mockAgents: Agent[] = [
    { id: '1', name: 'Vision Processor', status: 'running', model: 'YOLOv8', lastActive: 'Now' },
    { id: '2', name: 'Audio Transcriber', status: 'idle', model: 'Whisper', lastActive: '5m ago' },
    { id: '3', name: 'Text Generator', status: 'running', model: 'LLaMA-7B', lastActive: 'Now' },
    { id: '4', name: 'Image Enhancer', status: 'error', model: 'ESRGAN', lastActive: '1h ago' },
    { id: '5', name: 'Code Assistant', status: 'offline', model: 'CodeGen', lastActive: '2d ago' },
];

const mockTrainingRuns: TrainingRun[] = [
    {
        id: '1',
        name: 'YOLOv8 Fine-tune',
        status: 'running',
        progress: 67,
        epochs: 100,
        currentEpoch: 67,
        startedAt: '2024-01-20T08:00:00Z',
        dataset: 'custom-objects-v2',
    },
    {
        id: '2',
        name: 'Whisper Adaptation',
        status: 'queued',
        progress: 0,
        epochs: 50,
        currentEpoch: 0,
        startedAt: '',
        dataset: 'voice-samples',
    },
    {
        id: '3',
        name: 'Sentiment Classifier',
        status: 'completed',
        progress: 100,
        epochs: 30,
        currentEpoch: 30,
        startedAt: '2024-01-19T14:00:00Z',
        dataset: 'reviews-2024',
    },
];

const mockRunPhases: RunPhase[] = [
    {
        id: '1',
        name: 'Data Loading',
        status: 'completed',
        startedAt: '2024-01-20T08:00:00Z',
        completedAt: '2024-01-20T08:02:00Z',
        logs: ['Loading dataset...', 'Found 10,000 samples', 'Validation split: 20%'],
    },
    {
        id: '2',
        name: 'Preprocessing',
        status: 'completed',
        startedAt: '2024-01-20T08:02:00Z',
        completedAt: '2024-01-20T08:10:00Z',
        logs: ['Normalizing images...', 'Applying augmentations', 'Cache built'],
    },
    {
        id: '3',
        name: 'Training',
        status: 'running',
        startedAt: '2024-01-20T08:10:00Z',
        logs: ['Epoch 67/100', 'Loss: 0.0234', 'Accuracy: 94.2%'],
    },
    {
        id: '4',
        name: 'Evaluation',
        status: 'pending',
        logs: [],
    },
    {
        id: '5',
        name: 'Export',
        status: 'pending',
        logs: [],
    },
];

export const useAppStore = create<AppState>((set) => ({
    // Navigation
    activeModule: 'projects',
    setActiveModule: (module) => set({ activeModule: module }),

    // Command Palette
    isCommandPaletteOpen: false,
    openCommandPalette: () => set({ isCommandPaletteOpen: true }),
    closeCommandPalette: () => set({ isCommandPaletteOpen: false }),
    toggleCommandPalette: () => set((state) => ({ isCommandPaletteOpen: !state.isCommandPaletteOpen })),

    // Projects
    projects: mockProjects,
    selectedProjectId: null,
    selectProject: (id) => set({ selectedProjectId: id }),

    // Agents
    agents: mockAgents,

    // Training
    trainingRuns: mockTrainingRuns,

    // Runs/Timeline
    runPhases: mockRunPhases,

    // System Info
    systemInfo: null,
    setSystemInfo: (info) => set({ systemInfo: info }),
}));
