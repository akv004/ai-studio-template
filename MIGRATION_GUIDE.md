# Migration Guide: Legacy Apps → AI Studio Stack

> **Goal:** Migrate PyQt, Electron, or other desktop apps to the AI Studio architecture for consistency, performance, and long-term maintainability.

---

## Target Architecture

```
Desktop App (Tauri)
 ├─ Web UI (React + TypeScript)
 │   ├─ Panels, forms, settings (DOM)
 │   └─ Heavy visuals (Canvas/WebGL/WebGPU)
 │
 ├─ Native Core (Rust)
 │   ├─ File system access
 │   ├─ Device control / performance
 │   └─ Long-running background tasks
 │
 └─ AI Sidecar (Python)
     └─ ML inference, training, data processing
```

---

## Why This Stack?

| Old Stack | Problem | New Stack | Benefit |
|-----------|---------|-----------|---------|
| **Electron** | 100MB+ bundle, high memory | **Tauri** | 5-10MB, native performance |
| **PyQt** | Python UI limits, no web tech | **React** | Modern UI, huge ecosystem |
| **Qt Canvas** | Limited GPU acceleration | **Canvas/WebGPU** | GPU-ready, future-proof |
| **Python everywhere** | Slow UI, GIL issues | **Rust + Python sidecar** | Fast core, Python for AI only |

---

## Migration Checklist

### Phase 1: Project Setup
- [ ] Create new project from AI Studio Template
- [ ] Copy over any shared types/schemas
- [ ] Set up Python sidecar for AI logic

### Phase 2: UI Migration
- [ ] Map old UI screens → React pages
- [ ] Replace Qt widgets → React components
- [ ] Migrate heavy visuals → Canvas abstraction

### Phase 3: Native Core
- [ ] Port file system logic → Rust commands
- [ ] Port device access → Rust + Tauri plugins
- [ ] Port long-running tasks → Rust async

### Phase 4: AI Integration
- [ ] Keep Python AI code in sidecar
- [ ] Add HTTP endpoints for each AI operation
- [ ] Connect UI → Rust → Python pipeline

---

## Component Mapping

### UI Components

| Old (PyQt/Electron) | New (React) | Location |
|---------------------|-------------|----------|
| `QMainWindow` | `AppShell` | `apps/ui/src/app/layout/` |
| `QMenuBar` | `Header` + `CommandPalette` | `apps/ui/src/app/layout/` |
| `QDockWidget` | `Panel` components | `apps/ui/src/app/layout/` |
| `QListWidget` | Table/List components | React + CSS |
| `QGraphicsView` | `CanvasRenderer` | `apps/ui/src/canvas/` |
| `QMediaPlayer` | Audio module + waveform | `apps/ui/src/app/pages/AudioPage` |

### Native Operations

| Old (Python/Node) | New (Rust) | How |
|-------------------|------------|-----|
| `os.path`, `pathlib` | `std::path`, `std::fs` | Rust stdlib |
| `subprocess.run()` | `std::process::Command` | Tauri command |
| `sqlite3` | `rusqlite` or `sqlx` | Cargo dependency |
| `requests` | `reqwest` | Cargo dependency |
| File dialogs | `tauri-plugin-dialog` | Tauri plugin |
| System tray | `tauri-plugin-tray` | Tauri plugin |

---

## Canvas Migration Guide

### Before (Qt)
```python
# PyQt5 approach
class NodeEditor(QGraphicsView):
    def paintEvent(self, event):
        painter = QPainter(self)
        painter.drawRect(10, 10, 100, 50)
        painter.drawText(20, 30, "Node")
```

### After (Canvas Abstraction)
```typescript
// React + CanvasRenderer approach
import { CanvasRenderer } from './canvas';

const renderer = new CanvasRenderer();
renderer.init(canvasElement);

renderer.beginFrame();
renderer.drawRoundedRect(10, 10, 100, 50, 8, '#1e3a5f');
renderer.drawText('Node', 20, 30, '#ffffff');
renderer.drawBezier(/* connection curve */);
renderer.endFrame();
```

### Canvas API Reference

| Method | Purpose | Example |
|--------|---------|---------|
| `drawRect()` | Simple rectangles | Backgrounds |
| `drawRoundedRect()` | Rounded rectangles | Nodes, cards |
| `drawCircle()` | Circles | Ports, indicators |
| `drawLine()` | Straight lines | Grid, guides |
| `drawBezier()` | Curved connections | Node graphs |
| `drawText()` | Labels | Node names |

### WebGPU Future Path
```typescript
// Current: Canvas 2D
const renderer = new CanvasRenderer();

// Future: WebGPU (same interface!)
const renderer = new WebGPURenderer();
```

---

## Rust Command Pattern

### Creating a New Command

```rust
// apps/desktop/src-tauri/src/commands.rs

#[tauri::command]
pub async fn process_file(path: String) -> Result<ProcessResult, String> {
    // Rust logic here - fast, safe, concurrent
    let data = std::fs::read(&path)
        .map_err(|e| e.to_string())?;
    
    // Return serializable result
    Ok(ProcessResult { 
        size: data.len(),
        hash: calculate_hash(&data),
    })
}
```

### Registering Commands
```rust
// apps/desktop/src-tauri/src/lib.rs

tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
        commands::process_file,
        commands::list_projects,
        // Add new commands here
    ])
```

### Calling from UI
```typescript
// apps/ui/src/...
import { invoke } from '@tauri-apps/api/core';

const result = await invoke<ProcessResult>('process_file', { 
    path: '/path/to/file' 
});
```

---

## Python Sidecar Pattern

### HTTP Endpoints

```python
# apps/sidecar/server.py

class AIHandler(BaseHTTPRequestHandler):
    def do_POST(self):
        if self.path == '/inference':
            data = json.loads(self.rfile.read())
            result = run_model(data['input'])
            self.send_json(result)
```

### Calling from Rust

```rust
// Future: Call Python sidecar from Rust
let response = reqwest::Client::new()
    .post("http://localhost:8765/inference")
    .json(&input)
    .send()
    .await?;
```

---

## File Structure Template

```
my-migrated-app/
├── apps/
│   ├── desktop/          # Tauri shell
│   │   └── src-tauri/
│   │       ├── src/
│   │       │   ├── main.rs
│   │       │   ├── lib.rs
│   │       │   └── commands.rs  ← Add your Rust logic
│   │       └── Cargo.toml
│   │
│   ├── ui/               # React UI
│   │   └── src/
│   │       ├── app/
│   │       │   ├── layout/    ← Shell components
│   │       │   ├── pages/     ← Your screens
│   │       │   └── components/ ← Reusable UI
│   │       ├── canvas/        ← Heavy visuals
│   │       └── state/         ← Zustand store
│   │
│   └── sidecar/          # Python AI
│       ├── server.py
│       └── models/        ← Your ML code
│
└── packages/
    └── shared/           # Shared types
```

---

## Quick Start for Migration

```bash
# 1. Clone template
git clone <ai-studio-template-repo> my-new-app
cd my-new-app

# 2. Install dependencies
npm install

# 3. Start development
npm run tauri:dev

# 4. Start adding your logic:
#    - UI → apps/ui/src/app/pages/
#    - Rust → apps/desktop/src-tauri/src/commands.rs
#    - Python AI → apps/sidecar/
```

---

## Support

- **Tauri Docs:** https://tauri.app/
- **React Docs:** https://react.dev/
- **Rust Book:** https://doc.rust-lang.org/book/
