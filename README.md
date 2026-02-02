# AI Studio Framework

A production-grade, cross-platform AI Desktop Application **framework** built with **Tauri + React + TypeScript**.

> 🎯 **Purpose**: Extensible framework for building AI-powered desktop applications with a 5-10 year foundation. Not just a template — a complete architecture for professional AI applications.

## 🆕 What's New: Framework Evolution

This project is evolving from a **template** into a full **framework** with:
- **Module System** — Extensible, pluggable modules
- **Provider Abstraction** — Swap AI backends (Mock, OpenClaw, custom)
- **Event-Driven Architecture** — Decoupled, scalable design
- **Plugin Support** — Third-party extensions and skills

📄 See [Framework Architecture](docs/specs/architecture/framework-design.md) for details.

### 🦞 Coming Soon: OpenClaw Integration

Integration with [OpenClaw](https://github.com/openclaw/openclaw) — the open-source personal AI assistant platform — is planned. This will enable:
- Real AI agent conversations (not mocked)
- Multi-channel messaging (WhatsApp, Telegram, Discord, etc.)
- Voice interaction (Voice Wake, Talk Mode)
- Skills ecosystem

📄 See [OpenClaw Integration Spec](docs/specs/features/openclaw-integration.md) for details.

## Features

- ✅ **Cross-platform**: macOS, Windows, Linux
- ✅ **Professional GUI**: Node graphs, timelines, media panels
- ✅ **GPU-ready**: Canvas/WebGL now, WebGPU architecture ready
- ✅ **Clean separation**: UI, OS access, AI, and rendering layers
- ✅ **Extensible**: Vision, Audio, Agents, Training, Projects
- ✅ **Mock-first**: All data mocked for rapid prototyping
- ✅ **Module System**: Pluggable, registerable modules
- ✅ **Provider Abstraction**: Swappable AI backends
- 🔜 **OpenClaw Integration**: Real AI assistant capabilities

---

## 📸 Screenshots

### Main Dashboard
![Main Dashboard](docs/screenshots/screenshot-1-main-dashboard.png)

### Canvas Demo (Node Graph)
![Canvas Demo](docs/screenshots/screenshot-2-canvas-demo.png)

### Vision Module
![Vision Module](docs/screenshots/screenshot-4-vision.png)

### Agents
![Agents](docs/screenshots/agents.png)

### Training
![Training](docs/screenshots/screenshot-training.png)

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

This creates platform-specific installers in `apps/desktop/src-tauri/target/release/bundle/`.

### macOS Installation

1. Build creates:
   - `bundle/macos/AI Studio.app` - Application bundle
   - `bundle/dmg/AI Studio_X.X.X_aarch64.dmg` - Installer disk image

2. **Install via DMG (Recommended)**:
   ```bash
   # Open the DMG
   open apps/desktop/src-tauri/target/release/bundle/dmg/AI\ Studio_*.dmg
   # Drag "AI Studio" to Applications folder
   ```

3. **Or copy directly**:
   ```bash
   cp -r "apps/desktop/src-tauri/target/release/bundle/macos/AI Studio.app" /Applications/
   ```

4. **First launch**: Right-click → Open (bypasses Gatekeeper for unsigned apps)

### Windows Installation

1. Build creates:
   - `bundle/msi/AI Studio_X.X.X_x64_en-US.msi` - MSI installer
   - `bundle/nsis/AI Studio_X.X.X_x64-setup.exe` - NSIS installer

2. **Install via MSI**:
   - Double-click the `.msi` file
   - Follow the installation wizard
   - App installs to `C:\Program Files\AI Studio\`

3. **Or run portable**:
   - Use `target/release/ai-studio.exe` directly

### Linux Installation

1. Build creates:
   - `bundle/deb/ai-studio_X.X.X_amd64.deb` - Debian package
   - `bundle/appimage/ai-studio_X.X.X_amd64.AppImage` - Portable AppImage

2. **Install via .deb (Debian/Ubuntu)**:
   ```bash
   sudo dpkg -i apps/desktop/src-tauri/target/release/bundle/deb/ai-studio_*.deb
   # Fix dependencies if needed:
   sudo apt-get install -f
   ```

3. **Or run AppImage (Any distro)**:
   ```bash
   chmod +x apps/desktop/src-tauri/target/release/bundle/appimage/ai-studio_*.AppImage
   ./ai-studio_*.AppImage
   ```

---

## 🗺️ Roadmap

### v0.2.0 - Framework Foundation (Current)
- [x] Core architecture design *(spec complete)*
- [x] Module system specification *(spec complete)*
- [x] Provider abstraction design *(spec complete)*
- [ ] Event bus implementation
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

📄 See [Framework Architecture](docs/specs/architecture/framework-design.md) for detailed roadmap.

---

## 📚 Documentation

- [Framework Architecture](docs/specs/architecture/framework-design.md) — Core framework design
- [OpenClaw Integration](docs/specs/features/openclaw-integration.md) — AI assistant integration spec
- [Specifications Index](docs/specs/README.md) — All technical specs

---

## License

MIT

---

Built with ❤️ using Tauri, React, and TypeScript.
