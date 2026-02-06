// ============================================
// MOCK FIXTURES
// Separated from store for cleaner architecture
// ============================================

import type { Project, Agent, TrainingRun, PhaseStatus, LogEntry } from '@ai-studio/shared';

// UI-specific RunPhase (with LogEntry[] instead of simple logs)
export interface RunPhase {
    id: string;
    name: string;
    status: PhaseStatus;
    startedAt?: string;
    completedAt?: string;
    logs: LogEntry[];
}

// ============================================
// PROJECTS
// ============================================

export const mockProjects: Project[] = [
    {
        id: '1',
        name: 'Object Detection Pipeline',
        description: 'Real-time object detection for autonomous navigation',
        status: 'active',
        createdAt: '2024-01-15T10:00:00Z',
        updatedAt: '2024-01-20T14:30:00Z',
    },
    {
        id: '2',
        name: 'Voice Assistant',
        description: 'Multi-language voice recognition and synthesis',
        status: 'active',
        createdAt: '2024-01-10T08:00:00Z',
        updatedAt: '2024-01-19T16:45:00Z',
    },
    {
        id: '3',
        name: 'Document Analyzer',
        description: 'OCR and NLP for document processing',
        status: 'draft',
        createdAt: '2024-01-05T12:00:00Z',
        updatedAt: '2024-01-18T11:20:00Z',
    },
];

// ============================================
// AGENTS
// ============================================

export const mockAgents: Agent[] = [
    { id: '1', name: 'Vision Processor', status: 'running', model: 'YOLOv8', capabilities: ['vision'], lastActive: 'Now' },
    { id: '2', name: 'Audio Transcriber', status: 'idle', model: 'Whisper', capabilities: ['audio'], lastActive: '5m ago' },
    { id: '3', name: 'Text Generator', status: 'running', model: 'LLaMA-7B', capabilities: ['text'], lastActive: 'Now' },
    { id: '4', name: 'Image Enhancer', status: 'error', model: 'ESRGAN', capabilities: ['vision'], lastActive: '1h ago' },
    { id: '5', name: 'Code Assistant', status: 'offline', model: 'CodeGen', capabilities: ['code'], lastActive: '2d ago' },
];

// ============================================
// TRAINING RUNS
// ============================================

export const mockTrainingRuns: TrainingRun[] = [
    {
        id: '1',
        name: 'YOLOv8 Fine-tune',
        status: 'running',
        progress: 67,
        epochs: 100,
        currentEpoch: 67,
        startedAt: '2024-01-20T08:00:00Z',
        dataset: 'custom-objects-v2',
        model: 'yolov8n',
        hyperparameters: { learningRate: 0.001, batchSize: 32, epochs: 100, optimizer: 'AdamW' },
        metrics: { loss: 0.0234, accuracy: 0.942 },
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
        model: 'whisper-base',
        hyperparameters: { learningRate: 0.0001, batchSize: 16, epochs: 50, optimizer: 'Adam' },
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
        model: 'bert-base',
        hyperparameters: { learningRate: 0.00005, batchSize: 64, epochs: 30, optimizer: 'AdamW' },
        metrics: { loss: 0.012, accuracy: 0.967, valAccuracy: 0.951 },
    },
];

// ============================================
// RUN PHASES
// ============================================

export const mockRunPhases: RunPhase[] = [
    {
        id: '1',
        name: 'Data Loading',
        status: 'completed',
        startedAt: '2024-01-20T08:00:00Z',
        completedAt: '2024-01-20T08:02:00Z',
        logs: [
            { timestamp: '2024-01-20T08:00:00Z', level: 'info', message: 'Loading dataset...' },
            { timestamp: '2024-01-20T08:01:00Z', level: 'info', message: 'Found 10,000 samples' },
            { timestamp: '2024-01-20T08:02:00Z', level: 'info', message: 'Validation split: 20%' },
        ],
    },
    {
        id: '2',
        name: 'Preprocessing',
        status: 'completed',
        startedAt: '2024-01-20T08:02:00Z',
        completedAt: '2024-01-20T08:10:00Z',
        logs: [
            { timestamp: '2024-01-20T08:02:30Z', level: 'info', message: 'Normalizing images...' },
            { timestamp: '2024-01-20T08:05:00Z', level: 'info', message: 'Applying augmentations' },
            { timestamp: '2024-01-20T08:10:00Z', level: 'info', message: 'Cache built' },
        ],
    },
    {
        id: '3',
        name: 'Training',
        status: 'running',
        startedAt: '2024-01-20T08:10:00Z',
        logs: [
            { timestamp: '2024-01-20T09:00:00Z', level: 'info', message: 'Epoch 67/100' },
            { timestamp: '2024-01-20T09:00:01Z', level: 'info', message: 'Loss: 0.0234' },
            { timestamp: '2024-01-20T09:00:02Z', level: 'info', message: 'Accuracy: 94.2%' },
        ],
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
