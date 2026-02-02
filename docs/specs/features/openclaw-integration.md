# Feature Spec: OpenClaw Integration

> **Status**: Proposed  
> **Priority**: High  
> **Estimated Effort**: Medium-Large  
> **Target Version**: 0.2.0

## Overview

Integrate [OpenClaw](https://github.com/openclaw/openclaw) — a personal AI assistant platform — as an optional backend for the AI Studio Framework's Agents module. This integration transforms the mock agent system into a fully functional AI assistant workstation.

## Background

### What is OpenClaw?

OpenClaw is an open-source personal AI assistant (141K+ GitHub stars) that:
- Runs locally on your devices as a control plane
- Connects to multiple messaging channels (WhatsApp, Telegram, Slack, Discord, Signal, iMessage, etc.)
- Provides real AI agent capabilities with voice, canvas, browser control, and skills
- Uses a WebSocket gateway architecture for real-time communication
- Supports multiple AI models (Anthropic Claude, OpenAI GPT, etc.)

### Why Integrate OpenClaw?

| Current State | With OpenClaw |
|--------------|---------------|
| Mock AI responses | Real AI inference |
| Isolated desktop app | Multi-channel messaging |
| Static agent status | Live agent sessions |
| No voice support | Voice Wake + Talk Mode |
| No external integrations | Skills ecosystem |

## Technical Requirements

### Prerequisites

- Node.js ≥22 (for OpenClaw)
- OpenClaw installed globally (`npm install -g openclaw@latest`)
- OpenClaw Gateway running (`openclaw gateway --port 18789`)

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     AI STUDIO FRAMEWORK                         │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    React UI Layer                        │   │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────────────┐            │   │
│  │  │Projects │ │ Vision  │ │ Agents (Enhanced)│  ...      │   │
│  │  └─────────┘ └─────────┘ └────────┬────────┘            │   │
│  │                                   │                      │   │
│  │  ┌────────────────────────────────┴─────────────────┐   │   │
│  │  │           OpenClaw Bridge Layer                   │   │   │
│  │  │  ┌──────────────┐    ┌──────────────────────┐    │   │   │
│  │  │  │ WS Client    │    │ Channel Manager      │    │   │   │
│  │  │  └──────────────┘    └──────────────────────┘    │   │   │
│  │  └──────────────────────────────────────────────────┘   │   │
│  └─────────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────────┐              ┌──────────────────────┐    │
│  │   Tauri / Rust   │◄────IPC────►│  OpenClaw Gateway    │    │
│  │   (OS Access)    │              │  ws://127.0.0.1:18789│    │
│  └──────────────────┘              └──────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
```

### Integration Points

#### 1. Gateway Connection (`packages/openclaw-bridge/`)

Create a new package for OpenClaw integration:

```typescript
// packages/openclaw-bridge/src/gateway.ts
export interface OpenClawConfig {
  gatewayUrl: string;  // default: ws://127.0.0.1:18789
  autoReconnect: boolean;
  channels: ChannelConfig[];
}

export class OpenClawGateway {
  connect(): Promise<void>;
  disconnect(): void;
  sendMessage(channel: string, content: string): Promise<void>;
  onMessage(handler: MessageHandler): void;
}
```

#### 2. Agent Provider Interface (`packages/shared/types/agent-provider.ts`)

Abstract the agent backend to support multiple providers:

```typescript
export interface AgentProvider {
  id: string;
  name: string;
  connect(): Promise<void>;
  sendMessage(sessionId: string, message: string): Promise<AgentResponse>;
  getStatus(): AgentStatus;
}

// Implementations:
// - MockAgentProvider (existing)
// - OpenClawAgentProvider (new)
```

#### 3. UI Enhancements (`apps/ui/src/app/pages/AgentsPage.tsx`)

- Add provider selector (Mock / OpenClaw)
- Display connected channels
- Show real agent status from Gateway
- Enable skill management UI

### API Mapping

| AI Studio | OpenClaw Gateway |
|-----------|-----------------|
| `agent.sendMessage()` | `POST /api/agent/send` |
| `agent.getStatus()` | `GET /api/agent/status` |
| `agent.listChannels()` | `GET /api/channels` |
| `agent.connectChannel()` | `POST /api/channels/{id}/connect` |

## Implementation Phases

### Phase 1: Gateway Bridge (Week 1-2)

- [ ] Create `packages/openclaw-bridge` package
- [ ] Implement WebSocket client for Gateway
- [ ] Add connection status to UI
- [ ] Basic message send/receive

### Phase 2: Agent Provider Abstraction (Week 3)

- [ ] Define `AgentProvider` interface
- [ ] Refactor mock agent to use interface
- [ ] Implement `OpenClawAgentProvider`
- [ ] Add provider selection to Settings

### Phase 3: Channel Management (Week 4)

- [ ] Add channel list UI
- [ ] Implement channel connection flow
- [ ] Support WhatsApp/Telegram/Discord pairing
- [ ] Channel status indicators

### Phase 4: Advanced Features (Week 5-6)

- [ ] Voice Wake integration (if macOS)
- [ ] Canvas A2UI support
- [ ] Skills browser and installer
- [ ] Conversation history sync

## Configuration

### Settings Integration

Add to Settings page:

```yaml
# OpenClaw Settings
openclaw:
  enabled: false
  gatewayUrl: "ws://127.0.0.1:18789"
  autoStart: false
  defaultModel: "claude-sonnet-4-20250514"
  channels:
    whatsapp:
      enabled: false
    telegram:
      enabled: false
    discord:
      enabled: false
```

### Environment Variables

```bash
OPENCLAW_GATEWAY_URL=ws://127.0.0.1:18789
OPENCLAW_AUTO_START=false
```

## Dependencies

### New npm packages

```json
{
  "dependencies": {
    "ws": "^8.x"  // WebSocket client (already bundled by Tauri)
  },
  "optionalDependencies": {
    "openclaw": "^latest"  // For auto-spawning gateway
  }
}
```

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| OpenClaw requires Node ≥22 | Document requirement, graceful fallback to mock |
| Gateway not running | Auto-detect and prompt user to start |
| Breaking API changes | Version lock, abstraction layer |
| Performance overhead | Lazy loading, connection pooling |

## Success Criteria

- [ ] Users can connect to OpenClaw Gateway from AI Studio
- [ ] Real AI conversations work through Agents page
- [ ] At least one messaging channel (Discord/Telegram) can be configured
- [ ] Graceful fallback when OpenClaw is unavailable
- [ ] Documentation complete

## References

- [OpenClaw Repository](https://github.com/openclaw/openclaw)
- [OpenClaw Documentation](https://docs.openclaw.ai)
- [OpenClaw Gateway API](https://docs.openclaw.ai/gateway)
- [OpenClaw Skills Platform](https://docs.openclaw.ai/tools/skills)
