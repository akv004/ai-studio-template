type SidecarProxyResponse = {
  status: number;
  json: unknown | null;
  text: string | null;
};

const SIDECAR_BASE_URL = import.meta.env.VITE_SIDECAR_URL || 'http://localhost:8765';

function isTauri(): boolean {
  return typeof window !== 'undefined' && (
    typeof (window as any).__TAURI_INTERNALS__ !== 'undefined' ||
    typeof (window as any).__TAURI__ !== 'undefined'
  );
}

export async function sidecarRequest<T = unknown>(
  method: string,
  path: string,
  body?: unknown,
): Promise<T> {
  if (!path.startsWith('/')) path = `/${path}`;

  if (isTauri()) {
    const { invoke } = await import('@tauri-apps/api/core');
    const requestPayload: { method: string; path: string; body?: unknown } = { method, path };
    if (typeof body !== 'undefined') requestPayload.body = body;
    const resp = await invoke<SidecarProxyResponse>('sidecar_request', {
      request: requestPayload,
    });

    if (resp.status >= 200 && resp.status < 300) {
      return (resp.json ?? resp.text ?? null) as T;
    }

    const message =
      (typeof resp.text === 'string' && resp.text) ||
      (resp.json ? JSON.stringify(resp.json) : '') ||
      `Sidecar error (HTTP ${resp.status})`;
    throw new Error(message);
  }

  const hasBody = typeof body !== 'undefined';
  const res = await fetch(`${SIDECAR_BASE_URL}${path}`, {
    method,
    headers: hasBody
      ? { 'Content-Type': typeof body === 'string' ? 'text/plain' : 'application/json' }
      : undefined,
    body: hasBody ? (typeof body === 'string' ? body : JSON.stringify(body)) : undefined,
  });

  if (!res.ok) {
    throw new Error(`Sidecar error (HTTP ${res.status})`);
  }

  return (await res.json()) as T;
}

export function sidecarGet<T = unknown>(path: string): Promise<T> {
  return sidecarRequest<T>('GET', path);
}
