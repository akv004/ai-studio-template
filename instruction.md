Role:
You are a Principal Software Architect designing a future-proof, cross-platform AI Desktop Application template.
The goal is architecture + scaffolding, not a fully working product.

🎯 GOAL

Create a production-grade starter template for an AI Desktop App that:

Runs on macOS, Windows, Linux

Supports complex professional GUI (node graphs, timelines, media panels)

Is GPU-ready (Canvas/WebGL now, WebGPU later)

Cleanly separates UI, OS access, AI, and rendering

Is extensible for Vision, Audio, Agents, Training, Projects

Uses mock data only (no real AI connections yet)

Looks and feels like a serious pro tool (Maya / Blender / Figma-class UX philosophy)

🧱 REQUIRED STACK (NON-NEGOTIABLE)
Desktop Shell

Tauri

Rust backend for OS/device access

UI

React + TypeScript

Vite build

Component library (shadcn/ui or equivalent)

Rendering Strategy

Canvas / WebGL first

Rendering layer MUST be abstracted to allow WebGPU later

No SVG-heavy or DOM-only node editors

AI Integration (Mocked)

Python sidecar interface design only

No real ML calls

Use mocked JSON responses

🧠 ARCHITECTURAL PRINCIPLES (VERY IMPORTANT)

Canvas-first UI

Node graphs, timelines, media previews must render in a canvas

UI is a scene graph, not widgets

Renderer abstraction

Renderer interface

CanvasRenderer implemented

WebGPURenderer stubbed

Clear process separation

UI (React)

Native OS (Rust)

AI (Python sidecar – mocked)

Professional UX

Keyboard shortcuts

Command palette

Run timelines

Persistent project state

Non-blocking UI

📁 REQUIRED PROJECT STRUCTURE
ai-studio-template/
├── apps/
│   ├── desktop/          # Tauri + Rust backend
│   │   ├── src/
│   │   │   ├── commands.rs
│   │   │   ├── system.rs
│   │   │   └── main.rs
│   │   └── tauri.conf.json
│   │
│   ├── ui/               # React + TypeScript
│   │   ├── src/
│   │   │   ├── app/
│   │   │   │   ├── layout/
│   │   │   │   ├── pages/
│   │   │   │   └── panels/
│   │   │   ├── canvas/
│   │   │   │   ├── Renderer.ts
│   │   │   │   ├── CanvasRenderer.ts
│   │   │   │   └── WebGPURenderer.stub.ts
│   │   │   ├── state/
│   │   │   ├── commands/
│   │   │   └── main.tsx
│   │   └── vite.config.ts
│   │
│   └── sidecar/          # Python AI mock
│       ├── server.py
│       └── mock_responses/
│
├── packages/
│   └── shared/
│       ├── types/
│       └── schemas/
│
├── data/
│   └── sample-projects/
│
└── README.md

🧩 REQUIRED UI MODULES (MOCKED)

Create placeholder UI screens for:

Projects

Project list

Local persistence (JSON)

Vision

Image preview

Fake capture button

Mock detection overlays

Audio

Waveform display

Play/record buttons (mock)

Agents

Agent list

Status pills

Mock chat timeline

Training

Dataset table

Augmentation toggles

Fake progress bar

Runs / Timeline

Phase-based execution timeline

Logs panel

Settings

Models

Paths

Performance toggles

Hotkeys

🧪 MOCKING RULES

No network calls

No real AI

All data comes from:

JSON files

In-memory mocks

Simulated delays

📄 DOCUMENTATION REQUIREMENTS

Generate:

README.md explaining:

Architecture

Why Tauri + React

Rendering strategy

How WebGPU fits later

Inline comments explaining why choices were made

A short “Future Roadmap” section

🚫 DO NOT

Do not over-engineer

Do not add auth

Do not add cloud

Do not add real ML

Do not hard-lock to WebGPU today

✅ SUCCESS CRITERIA

The output should:

Compile and run as a demo app

Feel like a real AI Studio UI

Be clean, readable, extensible

Serve as a foundation for 5–10 years