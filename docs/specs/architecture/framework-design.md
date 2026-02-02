# AI Studio Framework Architecture

> **Version**: 0.2.0  
> **Status**: Active Development  
> **Last Updated**: 2026-02-02

## Overview

AI Studio Framework is a production-grade, cross-platform foundation for building AI-powered desktop applications. Unlike a simple template, this framework provides:

- **Extensible Module System** — Add custom modules without modifying core
- **Provider Abstraction** — Swap AI backends (mock, OpenClaw, custom)
- **Plugin Architecture** — Third-party extensions and skills
- **Standardized APIs** — Consistent interfaces across all layers

## Core Principles

### 1. Framework vs Template

| Template | Framework |
|----------|-----------|
| Copy and modify | Extend and configure |
| One-time scaffold | Continuous foundation |
| Tight coupling | Loose coupling |
| Manual updates | Version upgrades |

### 2. Design Goals

- **Longevity**: 5-10 year foundation
- **Extensibility**: Easy to add modules/providers
- **Maintainability**: Clean separation of concerns
- **Performance**: GPU-ready, optimized rendering
- **Developer Experience**: Type-safe, well-documented

## Architecture Layers

```
┌─────────────────────────────────────────────────────────────────┐
│                    AI STUDIO FRAMEWORK                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │                    APPLICATION LAYER                       │ │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐         │ │
│  │  │ Module  │ │ Module  │ │ Module  │ │ Custom  │  ...    │ │
│  │  │(Vision) │ │(Agents) │ │(Training)│ │ Module │         │ │
│  │  └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘         │ │
│  │       └───────────┴───────────┴───────────┘               │ │
│  │                       │                                    │ │
│  │  ┌────────────────────┴────────────────────┐              │ │
│  │  │         MODULE REGISTRY                  │              │ │
│  │  └────────────────────┬────────────────────┘              │ │
│  └───────────────────────┼────────────────────────────────────┘ │
│                          │                                      │
│  ┌───────────────────────┼────────────────────────────────────┐ │
│  │                  CORE LAYER                                │ │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐   │ │
│  │  │  State   │  │ Commands │  │ Renderer │  │ Events   │   │ │
│  │  │ Manager  │  │  System  │  │  Engine  │  │   Bus    │   │ │
│  │  └──────────┘  └──────────┘  └──────────┘  └──────────┘   │ │
│  └────────────────────────────────────────────────────────────┘ │
│                          │                                      │
│  ┌───────────────────────┼────────────────────────────────────┐ │
│  │               PROVIDER LAYER                               │ │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │ │
│  │  │ AI Provider  │  │ Storage      │  │ Platform     │     │ │
│  │  │ (Mock/Claw)  │  │ Provider     │  │ Provider     │     │ │
│  │  └──────────────┘  └──────────────┘  └──────────────┘     │ │
│  └────────────────────────────────────────────────────────────┘ │
│                          │                                      │
│  ┌───────────────────────┼────────────────────────────────────┐ │
│  │              PLATFORM LAYER                                │ │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │ │
│  │  │    Tauri     │  │   Browser    │  │   Electron   │     │ │
│  │  │   (Rust)     │  │   (Web)      │  │   (Future)   │     │ │
│  │  └──────────────┘  └──────────────┘  └──────────────┘     │ │
│  └────────────────────────────────────────────────────────────┘ │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Module System

### Module Interface

Every module implements a standard interface:

```typescript
// packages/shared/types/module.ts
export interface FrameworkModule {
  id: string;
  name: string;
  version: string;
  icon: LucideIcon;
  
  // Lifecycle
  initialize(): Promise<void>;
  destroy(): Promise<void>;
  
  // UI
  Page: React.ComponentType;
  Settings?: React.ComponentType;
  
  // State
  getState(): ModuleState;
  
  // Commands
  commands?: CommandDefinition[];
}
```

### Built-in Modules

| Module | ID | Description |
|--------|-----|-------------|
| Projects | `core.projects` | Project management |
| Vision | `core.vision` | Image analysis |
| Audio | `core.audio` | Audio processing |
| Agents | `core.agents` | AI agent interaction |
| Training | `core.training` | Model training |
| Runs | `core.runs` | Pipeline execution |
| Settings | `core.settings` | Configuration |

### Custom Module Registration

```typescript
// In your application
import { registerModule } from '@ai-studio/core';
import { MyCustomModule } from './modules/my-custom';

registerModule(MyCustomModule);
```

## Provider System

### AI Provider Interface

```typescript
// packages/shared/types/ai-provider.ts
export interface AIProvider {
  id: string;
  name: string;
  
  // Connection
  connect(config: ProviderConfig): Promise<void>;
  disconnect(): Promise<void>;
  isConnected(): boolean;
  
  // Inference
  chat(messages: Message[]): AsyncIterable<ChatChunk>;
  complete(prompt: string): Promise<string>;
  embed(text: string): Promise<number[]>;
  
  // Capabilities
  capabilities(): ProviderCapabilities;
}
```

### Built-in Providers

| Provider | ID | Description |
|----------|-----|-------------|
| Mock | `mock` | Static responses for development |
| HTTP Sidecar | `http-sidecar` | Python HTTP server |
| OpenClaw | `openclaw` | Full AI assistant platform |

### Provider Registration

```typescript
import { registerProvider } from '@ai-studio/core';
import { OpenAIProvider } from './providers/openai';

registerProvider(OpenAIProvider);
```

## State Management

### Store Architecture

```typescript
// Zustand store with slices
interface AppState {
  // Core
  currentModule: string;
  settings: Settings;
  
  // Module states
  modules: Record<string, ModuleState>;
  
  // Provider states
  providers: Record<string, ProviderState>;
}
```

### State Persistence

- Settings: Persisted to disk (JSON/SQLite)
- Module state: Persisted per-module preference
- Session state: Memory only

## Event System

### Event Bus

```typescript
// packages/core/src/events.ts
export interface EventBus {
  emit<T>(event: string, payload: T): void;
  on<T>(event: string, handler: (payload: T) => void): Unsubscribe;
  once<T>(event: string, handler: (payload: T) => void): void;
}

// Usage
eventBus.emit('agent:message', { role: 'user', content: 'Hello' });
eventBus.on('agent:response', (msg) => console.log(msg));
```

### Standard Events

| Event | Payload | Description |
|-------|---------|-------------|
| `module:activate` | `{ moduleId }` | Module switched |
| `provider:connect` | `{ providerId }` | Provider connected |
| `agent:message` | `{ message }` | New chat message |
| `training:progress` | `{ epoch, loss }` | Training update |

## Command System

### Command Definition

```typescript
// packages/shared/types/command.ts
export interface Command {
  id: string;
  label: string;
  shortcut?: string[];
  category: string;
  execute: () => void | Promise<void>;
}
```

### Global Commands

| Command | Shortcut | Action |
|---------|----------|--------|
| `command-palette` | `⌘K` | Open palette |
| `navigate:projects` | `⌘1` | Go to Projects |
| `settings:open` | `⌘,` | Open Settings |
| `project:new` | `⌘N` | New Project |

## Rendering Engine

### Canvas Abstraction

```typescript
// packages/canvas/src/renderer.ts
export interface CanvasRenderer {
  initialize(canvas: HTMLCanvasElement): void;
  render(scene: RenderScene): void;
  resize(width: number, height: number): void;
  destroy(): void;
}

// Implementations
export class Canvas2DRenderer implements CanvasRenderer { ... }
export class WebGLRenderer implements CanvasRenderer { ... }
export class WebGPURenderer implements CanvasRenderer { ... }  // Future
```

## Configuration

### Framework Config

```typescript
// ai-studio.config.ts
export default defineConfig({
  // Application
  name: 'My AI App',
  version: '1.0.0',
  
  // Modules
  modules: {
    enable: ['projects', 'vision', 'agents'],
    disable: ['training'],
  },
  
  // Providers
  providers: {
    default: 'openclaw',
    fallback: 'mock',
  },
  
  // Platform
  platform: 'tauri',
  
  // Features
  features: {
    commandPalette: true,
    darkMode: true,
    persistState: true,
  },
});
```

## Package Structure

```
ai-studio-framework/
├── packages/
│   ├── core/                 # Framework core
│   │   ├── src/
│   │   │   ├── module-registry.ts
│   │   │   ├── provider-registry.ts
│   │   │   ├── event-bus.ts
│   │   │   └── command-system.ts
│   │   └── package.json
│   │
│   ├── shared/               # Shared types & utilities
│   │   ├── types/
│   │   └── package.json
│   │
│   ├── canvas/               # Rendering engine
│   │   ├── src/
│   │   └── package.json
│   │
│   ├── ui/                   # React components
│   │   ├── src/
│   │   └── package.json
│   │
│   └── providers/            # AI provider implementations
│       ├── mock/
│       ├── http-sidecar/
│       └── openclaw/
│
├── apps/
│   ├── desktop/              # Tauri application
│   └── web/                  # Browser-only version
│
├── docs/
│   └── specs/                # Feature & architecture specs
│
└── ai-studio.config.ts       # Framework configuration
```

## Migration Path

### From Template (v0.1.x) to Framework (v0.2.x)

1. **Update dependencies**
   ```bash
   npm install @ai-studio/core@latest
   ```

2. **Create config file**
   ```bash
   npx ai-studio init
   ```

3. **Migrate modules** (if customized)
   - Extract to module interface
   - Register with framework

4. **Update imports**
   ```typescript
   // Before
   import { useAppStore } from './state/store';
   
   // After
   import { useAppStore } from '@ai-studio/core';
   ```

## Roadmap

### v0.2.0 - Framework Foundation
- [ ] Module system
- [ ] Provider abstraction
- [ ] Event bus
- [ ] Config file support

### v0.3.0 - Provider Ecosystem
- [ ] OpenClaw integration
- [ ] Plugin architecture
- [ ] Skills support

### v0.4.0 - Advanced Features
- [ ] WebGPU renderer
- [ ] Voice integration
- [ ] Multi-window support

### v1.0.0 - Stable Release
- [ ] API stability guarantee
- [ ] Full documentation
- [ ] Example applications
