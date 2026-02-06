import { sidecarRequest } from '../lib/sidecar';

/**
 * Base fetch wrapper with common configuration
 */
export async function fetchApi<T>(
    endpoint: string,
    options: RequestInit = {}
): Promise<T> {
    const method = (options.method || 'GET').toUpperCase();

    let body: unknown = undefined;
    if (typeof options.body !== 'undefined') {
        if (typeof options.body === 'string') {
            try {
                body = JSON.parse(options.body);
            } catch {
                body = options.body;
            }
        } else {
            body = options.body;
        }
    }

    return sidecarRequest<T>(method, endpoint, body);
}

// ============================================
// PROVIDER TYPES (from sidecar /providers)
// ============================================

export interface ProviderInfo {
    name: string;
    models: string[];
    default: boolean;
}

export interface ProvidersResponse {
    providers: ProviderInfo[];
}

// ============================================
// PROVIDER SERVICE
// ============================================

export async function listProviders(): Promise<ProvidersResponse> {
    return fetchApi<ProvidersResponse>('/providers');
}

export async function checkProviderAvailable(providerId: string): Promise<boolean> {
    const { providers } = await listProviders();
    return providers.some(p => p.name === providerId);
}

// ============================================
// HEALTH SERVICE
// ============================================

export interface HealthResponse {
    status: string;
    version: string;
}

export interface StatusResponse {
    status: string;
    providers: Record<string, { healthy: boolean; error?: string }>;
    active_conversations: number;
}

export async function getHealth(): Promise<HealthResponse> {
    return fetchApi<HealthResponse>('/health');
}

export async function getStatus(): Promise<StatusResponse> {
    return fetchApi<StatusResponse>('/status');
}
