<p align="center">
  <img src="docs/screenshots/screenshot-1-main-dashboard.png" alt="AI Studio" width="800">
</p>

<h1 align="center">🧠 AI Studio</h1>

<p align="center">
  <strong>Your Personal AI Assistant. Any OS. Any Provider. Local-First.</strong>
</p>

<p align="center">
  A production-grade, cross-platform AI desktop application that runs LLMs locally on your machine, connects to any cloud provider, and gives you full control of your AI experience.
</p>

<p align="center">
  <a href="#-quick-start">Quick Start</a> •
  <a href="#-features">Features</a> •
  <a href="#-architecture">Architecture</a> •
  <a href="#-roadmap">Roadmap</a> •
  <a href="#-contributing">Contributing</a>
</p>

---

## ✨ Why AI Studio?

| Problem | Solution |
|---------|----------|
| **Vendor Lock-in** | Swap providers with one config change |
| **Privacy Concerns** | Run 100% local with Ollama |
| **Limited Access** | Works from Telegram, Desktop, Web |
| **Black Box AI** | Full control, open source, self-hosted |

**AI Studio is the open-source alternative** for developers who want a personal AI assistant that:
- 🦙 **Runs locally** on your RTX 4090/5090 with Ollama
- ☁️ **Connects to clouds** — Anthropic, OpenAI, Google AI
- 💬 **Works anywhere** — Desktop app, Telegram, Web UI
- 🛠️ **Has real tools** — Shell, browser, filesystem access
- 🔐 **Respects privacy** — Your keys, your data, your machine

---

## 📸 Screenshots

<table>
  <tr>
    <td><img src="docs/screenshots/agents.png" alt="Agents" width="400"></td>
    <td><img src="docs/screenshots/screenshot-2-canvas-demo.png" alt="Canvas" width="400"></td>
  </tr>
  <tr>
    <td align="center"><strong>AI Agents Dashboard</strong></td>
    <td align="center"><strong>Canvas & Node Graphs</strong></td>
  </tr>
  <tr>
    <td><img src="docs/screenshots/screenshot-training.png" alt="Training" width="400"></td>
    <td><img src="docs/screenshots/setting_providers.png" alt="Settings" width="400"></td>
  </tr>
  <tr>
    <td align="center"><strong>Training Pipeline</strong></td>
    <td align="center"><strong>Multi-Provider Settings</strong></td>
  </tr>
</table>

---

## 🎯 Features

### ✅ What's Built

| Feature | Description |
|---------|-------------|
| **Multi-Provider LLM** | Ollama, Anthropic Claude, OpenAI GPT, Google Gemini |
| **Conversation Memory** | Persistent sessions with context retention |
| **Telegram Bot** | Chat with your AI from anywhere via Telegram |
| **Desktop App** | Native macOS/Windows/Linux with Tauri |
| **Shell Tool** | Execute commands with safety controls |
| **Browser Tool** | Playwright automation — navigate, click, extract |
| **Filesystem Tool** | Read/write files with path safety |
| **Docker Compose** | One-command deployment with GPU support |
| **Settings UI** | Configure providers, API keys, models |

### 🗺️ Roadmap

| Phase | Feature | Status |
|-------|---------|--------|
| **1** | WhatsApp / Discord / Slack channels | 🔜 Planned |
| **2** | Voice (Wake word + STT/TTS) | 🔜 Planned |
| **2** | Screen capture + Vision | 🔜 Planned |
| **3** | Cron jobs & Webhooks | 🔜 Planned |
| **4** | Skills/Plugin system | 🔜 Planned |
| **5** | Multi-agent routing | 🔜 Future |

> See full roadmap: [`docs/specs/agent-roadmap.md`](docs/specs/agent-roadmap.md)

---

## 🚀 Quick Start

### Option 1: Docker (Recommended)

```bash
# Clone the repo
git clone https://github.com/akv004/ai-studio-template.git
cd ai-studio-template

# Start with Docker (includes Ollama)
docker compose up

# With GPU support (NVIDIA)
docker compose --profile gpu up
```

### Option 2: Local Development

```bash
# Install dependencies
npm install
cd apps/sidecar && pip install -r requirements.txt

# Start the AI backend
python server.py  # http://localhost:8765

# Start the desktop app (in another terminal)
npm run tauri:dev
```

### Option 3: Cloud Providers Only

```bash
# Set your API keys
export ANTHROPIC_API_KEY=sk-ant-...
export OPENAI_API_KEY=sk-...
export GOOGLE_API_KEY=...

# Start the server
cd apps/sidecar && python server.py
```

---

## 📦 Installation Guide

### Prerequisites

| Tool | Version | Purpose |
|------|---------|---------|
| **Node.js** | 18+ | JavaScript runtime |
| **npm** | 9+ | Package manager |
| **Rust** | Latest stable | Native desktop shell (Tauri) |
| **Python** | 3.10+ | AI sidecar |

### Step 1: Install Node.js

```bash
# macOS (Homebrew)
brew install node

# Windows (Chocolatey)
choco install nodejs

# Or download from https://nodejs.org
```

### Step 2: Install Rust (Required for Desktop App)

```bash
# macOS / Linux
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Restart terminal or run:
source ~/.cargo/env

# Verify
rustc --version
```

### Step 3: Install Tauri CLI

```bash
cargo install tauri-cli
```

> ⏱️ First time takes ~2-3 minutes (compiles 700+ crates)

### Step 4: Clone & Install

```bash
git clone https://github.com/akv004/ai-studio-template.git
cd ai-studio-template
npm install
```

### Step 5: Run

```bash
# Browser only (for UI dev)
npm run dev

# Native desktop app
npm run tauri:dev
```

> ⏱️ First Tauri build takes ~3-5 minutes. Subsequent runs are instant.

## 🏗️ Architecture

```
AI Studio
├── 🖥️  Desktop App (Tauri + React)
│   ├── Professional UI with node graphs, timelines
│   ├── Settings for providers, models, API keys
│   └── Native performance, cross-platform
│
├── 🤖  AI Sidecar (Python FastAPI)
│   ├── Multi-provider LLM abstraction
│   ├── Conversation memory
│   └── Tools (shell, browser, filesystem)
│
├── 📱  Channels
│   ├── Telegram bot
│   ├── Web UI (built-in)
│   └── (Planned: WhatsApp, Discord, Slack)
│
└── 🐳  Docker Compose
    ├── Ollama (local LLM)
    ├── Sidecar (agent server)
    └── GPU support (NVIDIA)
```

---

## 🔧 Configuration

### Environment Variables

```bash
# Local LLM (Ollama)
OLLAMA_HOST=http://localhost:11434
OLLAMA_MODEL=llama3.2

# Cloud Providers (optional)
ANTHROPIC_API_KEY=sk-ant-...
OPENAI_API_KEY=sk-...
GOOGLE_API_KEY=...

# Telegram (optional)
TELEGRAM_BOT_TOKEN=...

# Tools safety mode: sandboxed | restricted | full
TOOLS_MODE=restricted
```

### Providers

| Provider | Models | Get API Key |
|----------|--------|-------------|
| **Ollama** | llama3.2, mistral, codellama | [ollama.ai](https://ollama.ai) |
| **Anthropic** | claude-sonnet-4, claude-opus-4 | [console.anthropic.com](https://console.anthropic.com) |
| **OpenAI** | gpt-4o, o1 | [platform.openai.com](https://platform.openai.com/api-keys) |
| **Google AI** | gemini-2.0-flash, gemini-1.5-pro | [aistudio.google.com](https://aistudio.google.com/apikey) |

---

## 📡 API Reference

### Chat

```bash
# Start a conversation
curl -X POST http://localhost:8765/chat \
  -H "Content-Type: application/json" \
  -d '{"message": "Hello!", "provider": "ollama"}'

# List providers
curl http://localhost:8765/providers

# Health check
curl http://localhost:8765/status
```

### Tools

```bash
# Shell command
curl -X POST http://localhost:8765/tools/shell \
  -d '{"command": "ls -la"}'

# Read file
curl -X POST http://localhost:8765/tools/filesystem \
  -d '{"action": "read", "path": "README.md"}'

# Browser automation
curl -X POST http://localhost:8765/tools/browser/start
curl -X POST http://localhost:8765/tools/browser \
  -d '{"action": "navigate", "url": "https://example.com"}'
```

### Telegram Bot

Chat with your AI from Telegram:
1. Create bot via [@BotFather](https://t.me/BotFather)
2. Set `TELEGRAM_BOT_TOKEN=your_token`
3. Run: `python -m channels.telegram`

**Commands:** `/start`, `/clear`, `/provider <name>`, `/model <name>`, `/status`

---

## 🛠️ Development

### Project Structure

```
ai-studio-template/
├── apps/
│   ├── desktop/          # Tauri native shell
│   ├── ui/               # React frontend
│   └── sidecar/          # Python AI backend
│       ├── agent/
│       │   ├── providers/  # LLM providers
│       │   ├── tools/      # Shell, browser, filesystem
│       │   └── chat.py     # Chat service
│       ├── channels/       # Telegram, etc.
│       └── server.py       # FastAPI server
├── docs/
│   └── specs/            # Roadmap, API docs
└── docker-compose.yml
```

### Prerequisites

| Tool | Version | Purpose |
|------|---------|---------|
| Node.js | 18+ | UI development |
| Rust | Latest | Desktop app (Tauri) |
| Python | 3.10+ | AI backend |
| Docker | Latest | Container deployment |

---

## 🤝 Contributing

Contributions welcome! Areas where help is needed:

- 📱 **New Channels** — WhatsApp, Discord, Slack integrations
- 🎤 **Voice** — Wake word, STT/TTS
- 🧩 **Skills** — Plugin system for extensibility
- 📚 **Docs** — Tutorials, examples

---

## 📄 License

MIT License — Free for personal and commercial use.

---

<p align="center">
  <strong>Built for developers who want AI on their terms.</strong>
</p>

<p align="center">
  ⭐ Star if you find this useful!
</p>
