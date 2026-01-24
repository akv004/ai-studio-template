// ============================================
// SHARED TYPES - PROJECT
// Core project data structures used across UI and backend
// ============================================

/**
 * Project status
 */
export type ProjectStatus = 'active' | 'archived' | 'draft';

/**
 * Project data structure
 * Represents an AI project with its metadata
 */
export interface Project {
    id: string;
    name: string;
    description: string;
    status: ProjectStatus;
    createdAt: string;
    updatedAt: string;
    thumbnail?: string;
    tags?: string[];
}

/**
 * Create project request
 */
export interface CreateProjectRequest {
    name: string;
    description: string;
    tags?: string[];
}

/**
 * Update project request
 */
export interface UpdateProjectRequest {
    id: string;
    name?: string;
    description?: string;
    status?: ProjectStatus;
    tags?: string[];
}
