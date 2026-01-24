// ============================================
// SHARED TYPES - TRAINING
// Training run and dataset structures
// ============================================

/**
 * Training run status
 */
export type TrainingStatus = 'queued' | 'running' | 'completed' | 'failed' | 'cancelled';

/**
 * Dataset format
 */
export type DatasetFormat = 'json' | 'csv' | 'parquet' | 'images' | 'audio';

/**
 * Training run definition
 */
export interface TrainingRun {
    id: string;
    name: string;
    status: TrainingStatus;
    progress: number;
    epochs: number;
    currentEpoch: number;
    startedAt: string;
    completedAt?: string;
    dataset: string;
    model: string;
    hyperparameters: TrainingHyperparameters;
    metrics?: TrainingMetrics;
}

/**
 * Training hyperparameters
 */
export interface TrainingHyperparameters {
    learningRate: number;
    batchSize: number;
    epochs: number;
    optimizer: string;
    scheduler?: string;
    weightDecay?: number;
}

/**
 * Training metrics
 */
export interface TrainingMetrics {
    loss: number;
    accuracy?: number;
    valLoss?: number;
    valAccuracy?: number;
    customMetrics?: Record<string, number>;
}

/**
 * Dataset definition
 */
export interface Dataset {
    id: string;
    name: string;
    format: DatasetFormat;
    size: number;
    sampleCount: number;
    createdAt: string;
    path: string;
    augmentations?: string[];
}
