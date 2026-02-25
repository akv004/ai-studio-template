export interface Trigger {
  id: string;
  workflowId: string;
  triggerType: 'webhook' | 'cron' | 'file_watch';
  config: WebhookConfig;
  enabled: boolean;
  lastFired: string | null;
  fireCount: number;
  createdAt: string;
  updatedAt: string;
}

export interface WebhookConfig {
  path: string;
  methods?: string[];
  authMode?: 'none' | 'token' | 'hmac';
  authToken?: string;
  hmacSecret?: string;
  responseMode?: 'immediate' | 'wait';
  timeoutSecs?: number;
  maxPerMinute?: number;
}

export interface TriggerLogEntry {
  id: string;
  triggerId: string;
  runId: string | null;
  firedAt: string;
  status: 'fired' | 'completed' | 'error';
  metadata: Record<string, unknown>;
}

export interface WebhookServerStatus {
  running: boolean;
  port: number;
  activeHooks: number;
}
