# AI Studio Template

A production-grade, cross-platform AI Desktop Application template built with **Tauri + React + TypeScript**.

> 🎯 **Purpose**: Architecture + scaffolding for a 5-10 year foundation. Not a fully working product, but a professional starting point.

## Features

- ✅ **Cross-platform**: macOS, Windows, Linux
- ✅ **Professional GUI**: Node graphs, timelines, media panels
- ✅ **GPU-ready**: Canvas/WebGL now, WebGPU architecture ready
- ✅ **Clean separation**: UI, OS access, AI, and rendering layers
- ✅ **Extensible**: Vision, Audio, Agents, Training, Projects
- ✅ **Mock-first**: All data mocked for rapid prototyping

---

## 🚀 Installation Guide

### Prerequisites

| Tool | Version | Purpose |
|------|---------|---------|
| **Node.js** | 18+ | JavaScript runtime |
| **npm** | 9+ | Package manager |
| **Rust** | Latest stable | Native desktop shell (Tauri) |
| **Python** | 3.10+ | AI sidecar (optional) |

### Step 1: Install Node.js

If not installed, download from [nodejs.org](https://nodejs.org/) or use:

```bash
# macOS (Homebrew)
brew install node

# Windows (Chocolatey)
choco install nodejs
```

### Step 2: Install Rust (Required for Desktop App)

Rust is needed to compile the native Tauri shell:

```bash
# macOS / Linux
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# When prompted, select: 1) Proceed with standard installation
```

After installation, **restart your terminal** or run:
```bash
source ~/.cargo/env
```

Verify installation:
```bash
rustc --version
# Should show: rustc 1.XX.X
```

### Step 3: Install Tauri CLI

```bash
cargo install tauri-cli
```

> ⏱️ First time takes ~2-3 minutes (compiles 700+ crates)

### Step 4: Clone & Install Dependencies

```bash
git clone <your-repo-url>
cd ai-studio-template
npm install
```

---

## 🏃 Running the Application

### Option A: Browser Only (Quick Development)

Best for UI development without Rust:

```bash
npm run dev
# Opens at http://localhost:1420
```

### Option B: Native Desktop App (Full Experience)

Runs as a real macOS/Windows/Linux application:

```bash
npm run tauri:dev
```

> ⏱️ First build takes ~3-5 minutes (compiles Rust dependencies)  
> Subsequent runs are instant.

### Option C: Run Python Sidecar (Mock AI Server)

```bash
npm run sidecar
# Or directly:
python apps/sidecar/server.py
```

---

## 📁 Project Structure

```
ai-studio-template/
├── apps/
│   ├── desktop/              # Tauri + Rust backend
│   │   └── src-tauri/
│   │       ├── src/
│   │       │   ├── main.rs       # Entry point
│   │       │   ├── lib.rs        # Core logic
│   │       │   ├── commands.rs   # IPC handlers
│   │       │   └── system.rs     # OS info
│   │       └── tauri.conf.json
│   │
│   ├── ui/                   # React + TypeScript + Vite
│   │   ├── src/
│   │   │   ├── app/
│   │   │   │   ├── layout/       # Shell, Header, Sidebar
│   │   │   │   └── pages/        # Module pages
│   │   │   ├── canvas/           # Rendering abstraction
│   │   │   ├── state/            # Zustand store
│   │   │   └── commands/         # Keyboard shortcuts
│   │   └── vite.config.ts
│   │
│   └── sidecar/              # Python AI mock server
│       ├── server.py
│       └── mock_responses/
│
├── packages/
│   └── shared/               # Shared types & schemas
│
├── data/
│   └── sample-projects/      # Mock project data
│
└── package.json              # Monorepo workspace config
```

---

## 🎨 UI Modules

| Module | Description | Key Features |
|--------|-------------|--------------|
| **Projects** | Project management | List, create, open, JSON persistence |
| **Vision** | Image analysis | Preview canvas, detection overlays |
| **Audio** | Audio processing | Waveform display, transcription |
| **Agents** | AI agent management | Status monitoring, chat interface |
| **Training** | Model training | Dataset management, progress tracking |
| **Runs** | Pipeline execution | Phase timeline, live logs |
| **Settings** | Configuration | Models, paths, hotkeys, appearance |

---

## ⌨️ Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `⌘K` / `Ctrl+K` | Open Command Palette |
| `⌘1-6` | Navigate to modules |
| `⌘,` | Open Settings |
| `⌘N` | New Project |

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        AI STUDIO                                │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    React UI Layer                        │   │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐        │   │
│  │  │Projects │ │ Vision  │ │ Agents  │ │Training │  ...   │   │
│  │  └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘        │   │
│  │       │           │           │           │              │   │
│  │  ┌────┴───────────┴───────────┴───────────┴────┐        │   │
│  │  │           State Management (Zustand)         │        │   │
│  │  └─────────────────────┬───────────────────────┘        │   │
│  │                        │                                 │   │
│  │  ┌─────────────────────┴───────────────────────┐        │   │
│  │  │        Canvas Rendering Layer               │        │   │
│  │  │  ┌──────────────┐    ┌──────────────────┐   │        │   │
│  │  │  │CanvasRenderer│    │WebGPURenderer.stub│  │        │   │
│  │  │  └──────────────┘    └──────────────────┘   │        │   │
│  │  └─────────────────────────────────────────────┘        │   │
│  └─────────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────────┐              ┌──────────────────┐        │
│  │   Tauri / Rust   │◄────IPC────►│  Python Sidecar  │        │
│  │   (OS Access)    │              │  (AI Inference)  │        │
│  └──────────────────┘              └──────────────────┘        │
└─────────────────────────────────────────────────────────────────┘
```

---

## 🛠️ Tech Stack

| Layer | Technology | Purpose |
|-------|------------|---------|
| Desktop Shell | Tauri 2.0 + Rust | Native OS access, windowing |
| UI Framework | React 19 + TypeScript | Component-based UI |
| Build Tool | Vite 7 | Fast HMR, optimized builds |
| Styling | Tailwind CSS 4 | Utility-first CSS |
| State | Zustand | Lightweight state management |
| Rendering | Canvas 2D (abstracted) | Node graphs, timelines |
| AI Interface | Python HTTP Server | Future ML integration |

---

## 🔧 Troubleshooting

### "cargo: command not found"
```bash
source ~/.cargo/env
# Or restart your terminal
```

### "Port 1420 is already in use"
```bash
# Kill the process using the port
lsof -ti:1420 | xargs kill -9
```

### Rust compilation taking too long
First-time compilation is slow (~3-5 min). Subsequent builds are fast.

---

## 📦 Building for Production

```bash
# Build production bundle
npm run tauri:build
```

This creates platform-specific installers in `apps/desktop/src-tauri/target/release/bundle/`:
- **macOS**: `.dmg`, `.app`
- **Windows**: `.msi`, `.exe`
- **Linux**: `.deb`, `.AppImage`

---

## License

MIT

---

Built with ❤️ using Tauri, React, and TypeScript.
