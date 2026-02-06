# AI Studio — Future Capabilities (Desktop-First Agent Framework)

> Purpose: define *framework-level* capabilities and contracts so AI Studio can grow into an OpenClaw-class system (and beyond) **without breaking existing agents**.
>
> Scope: specification only (not immediate implementation).

---

## Goals

- **Desktop-first control plane**: the desktop app is the primary UX for creating, running, observing, and approving agent actions.
- **Extensible, non-breaking framework**: new capabilities ship as plugins and “skills” with **versioned contracts** and **compatibility rules**.
- **Local-first by default**: runs locally (sidecar + desktop) with clear security boundaries and auditable approvals.
- **Visual workflows**: pipelines are graphs that map directly to the Canvas UI.
- **Replayable runs**: record inputs/events/artifacts so you can debug and iterate.

## Capability Matrix (Product Direction)

This matrix captures the intended “framework-level” capabilities (as artifacts/specs evolve).

| Planned Capability | Key Points |
|---|---|
| Video Stream Ingestion | Webcam, RTSP, IP cameras (Hikvision, Dahua), NVR, screen capture |
| AI Detection Pipeline | YOLOv8, face recognition, motion, anomaly, OCR |
| Alert System | Desktop notifications, webhooks, Telegram, email |
| Plugin Architecture | Custom sources, detectors, alerts, UI panels |

## Non-goals (for now)

- Building every detector/model immediately.
- A public marketplace. (Design for it, defer it.)
- Supporting remote, internet-exposed sidecars by default. (Local-only is the default posture.)

---

## Architectural Pillars (Stable Interfaces)

### Processes

- **Desktop (Tauri + UI)**: control plane, approvals, notifications, UX, local persistence/visibility.
- **Sidecar (Python)**: providers, pipelines, plugins, data plane execution (capture, inference, tools).
- **Channels (future)**: Telegram/WhatsApp/etc are *transports* that call into the same engine, governed by the same permissions/policies.

### Core abstractions

- **Source**: produces frames/messages (webcam/RTSP/file/screen, or even “text stream”).
- **Detector**: consumes input (frames/text/audio) → emits detections/events.
- **Rule**: evaluates events and state → emits “rule fired” events.
- **Sink (Alert)**: consumes events → performs an action (desktop notification, webhook, email, etc).
- **Skill**: a *packaged workflow* (graph + config + prompt templates + UI affordances) that uses the above primitives.

---

## Contract: Event Bus (Do Not Break)

All capability expansions must integrate through a typed event bus.

### Transport

- Sidecar exposes: `WS /events` (recommended), plus REST endpoints for control.
- Desktop subscribes and renders: timeline, artifacts, live previews, debug panels.

### Event envelope (v1)

Every event MUST include:

- `event_id` (uuid)
- `type` (string)
- `ts` (RFC3339)
- `run_id` (uuid, optional for ad-hoc actions)
- `source` (e.g. `sidecar.sources.rtsp`, `sidecar.detectors.yolo`, `desktop.approvals`)
- `payload` (typed per event)

### Event envelope JSON example (v1)

```json
{
  "event_id": "3e8f1f6f-1c2b-4aa2-9c2a-4b1fd4c8a1b2",
  "type": "detection.created",
  "ts": "2026-02-06T21:17:04.123Z",
  "run_id": "7b5c2c1a-7e9a-4e9a-9f3a-6c1d1c4f5b6a",
  "source": "sidecar.detectors.yolo",
  "payload": {
    "detection_id": "0d8c7a1b-82aa-4d8f-9d2c-7adf17b6b1a1",
    "stream_id": "cam_front_door",
    "kind": "object",
    "label": "person",
    "confidence": 0.94,
    "bbox": { "x": 0.15, "y": 0.10, "w": 0.25, "h": 0.60 },
    "model_id": "yolov8n",
    "model_version": "8.2.0",
    "artifacts": {
      "snapshot_path": "artifacts/runs/7b5c2c1a/snapshots/000124.jpg",
      "overlay_path": "artifacts/runs/7b5c2c1a/overlays/000124.png"
    }
  }
}
```

### Canonical event types (starter set)

- `run.started`, `run.ended`, `run.error`
- `stream.added`, `stream.removed`, `stream.connected`, `stream.disconnected`
- `frame.sampled` (metadata only; frame bytes handled as artifact/URL)
- `detection.created`
- `rule.fired`
- `alert.sent`, `alert.failed`
- `tool.requested`, `tool.approved`, `tool.denied`, `tool.completed`

This is the spine that keeps the system evolvable and debuggable.

---

## Capability: Video Stream Ingestion (Future)

### Supported sources

| Source Type | Protocol | Examples |
|---|---|---|
| Webcam | USB/V4L2 | built-in camera |
| IP camera | RTSP | `rtsp://.../stream` |
| NVR | RTSP/ONVIF | Hikvision, Dahua, Reolink |
| Video file | file path | `.mp4` recordings |
| Screen capture | OS APIs | monitor/window capture |

### Reliability requirements (what makes it “better than OpenClaw”)

- Reconnect policy (backoff, jitter, max attempts)
- Backpressure rules (frame drop strategy, max queue length)
- Clocking model (source fps vs sampling fps)
- Health telemetry per stream (uptime, lag, decode errors)

### Implementation note (informative)

OpenCV alone is often insufficient for production RTSP; prefer an FFmpeg/PyAV/GStreamer ingestion backend with explicit reconnect/backpressure.

---

## Capability: Detection Pipeline (Future)

### Detectors (examples)

| Task | Models | Notes |
|---|---|---|
| Object detection | YOLOv8/YOLOv10/RT-DETR | bbox + class + confidence |
| Face recognition | InsightFace/ArcFace | embeddings + identity match |
| Motion | frame diff / optical flow | efficient gating signal |
| Anomaly | isolation forest / autoencoder | requires baselines |
| OCR | PaddleOCR/EasyOCR | text + regions |

### Event flow (reference)

```
Detection → Rule Engine → Alert
                ↓
    - Desktop notification (Tauri)
    - Sound alert
    - Webhook (POST to URL)
    - Telegram/Discord bot (future)
    - Email (SMTP) (future)
    - Log to file/database
```

### Output contract: `Detection` (v1)

Minimum recommended fields:

- `detection_id`, `run_id`, `stream_id`
- `ts`
- `kind` (`object`, `face`, `ocr`, `motion`, `anomaly`, …)
- `label`, `confidence`
- `bbox` (optional), `polygon` (optional)
- `model_id`, `model_version`
- `artifacts` (optional: snapshot path, clip path, debug overlays)

---

## Capability: Alerts + Rules (Future)

### Rules engine (v1 requirements)

- Thresholds and cooldown windows
- De-duplication (avoid alert spam)
- Schedules (quiet hours)
- Zones (polygons), persistence (must hold N seconds)

### Sinks (alerts)

- Desktop notifications (native)
- Webhook (POST)
- Telegram/Discord (future)
- Email (SMTP) (future)
- File/DB logging

---

## Visual Workflow Graph (Framework Core)

AI Studio’s differentiator is **graph-native execution**, not chat-only orchestration.

### Graph model (v1)

- `Graph` contains `Nodes` and `Edges`
- Nodes are typed: `source.*`, `detector.*`, `rule.*`, `sink.*`, `tool.*`
- Each node has:
  - `node_id`, `type`, `config`, `inputs`, `outputs`
  - `resource_needs` (cpu/gpu/mem hints)
  - `permissions` (declared)

The Canvas UI is a direct editor/view for this graph.

---

## Extensibility: Plugin Architecture (Isolation + Compatibility)

You’re right to insist on isolation. Without it, adding new “skills” will break existing agents.

### Design principles

- **Versioned contracts**: plugins implement a stable interface (v1/v2). The host enforces compatibility.
- **Permission declarations**: plugins declare what they need (network, filesystem, camera, screen, GPU).
- **Isolation options** (progressive):
  1) *In-process* (fast iteration; least safe)
  2) *Subprocess* plugin host (recommended default for third-party)
  3) Container/VM sandbox (future)

### Plugin layout (suggested)

This mirrors the originally proposed “sources/detectors/alerts/ui” split while adding a manifest and versioning.

```
plugins/
  <plugin-id>/
    plugin.toml
    python/
      sources/*.py
      detectors/*.py
      alerts/*.py
      sinks/*.py
    ui/
      panels/*.tsx
```

### `plugin.toml` (minimum)

- `id`, `name`, `version`
- `api_version` (e.g. `1`)
- `entrypoints` (sources/detectors/sinks/ui)
- `permissions` (declared)
- `compat` (min/max AI Studio version)

### `plugin.toml` example (v1)

```toml
id = "com.acme.rtsp-pro"
name = "ACME RTSP Pro"
version = "0.1.0"

# Contract version implemented by this plugin.
api_version = 1

# AI Studio compatibility guardrails.
[compat]
min_ai_studio = "0.1.0"
max_ai_studio = "0.2.x"

# What code/UI this plugin provides.
[entrypoints]
sources = ["python.sources:RtspSource", "python.sources:OnvifDiscoverySource"]
detectors = ["python.detectors:YoloDetector"]
alerts = ["python.alerts:WebhookAlert"]
ui_panels = ["ui.panels:StreamsPanel"]

# Permissions must be explicit so the desktop can show meaningful prompts and enforce policy.
[permissions]
network = ["rtsp://*", "http://*", "https://*"]
filesystem_read = ["${WORKSPACE}/artifacts/**"]
filesystem_write = ["${WORKSPACE}/artifacts/**"]
camera = true
screen_capture = false
shell = false
gpu = true
```

### “Skills” are not plugins

- A **plugin** is code that adds new primitives (source/detector/sink/ui).
- A **skill** is a packaged workflow + prompts + defaults that composes primitives.

This separation keeps your ecosystem stable: skills can evolve quickly without binary compatibility risk.

---

## Planned API Surface (Sidecar)

Control plane APIs should stay small and stable.

### API endpoints (planned)

This list is kept intentionally close to the original capability artifact; the exact shapes are governed by the contracts above.

```
GET  /streams                 # List active streams
POST /streams                 # Add stream source
DELETE /streams/:id           # Remove stream
GET  /streams/:id/frame       # Get latest frame
WS   /streams/:id/live        # WebSocket live feed
POST /streams/:id/detect      # Run detection on frame
GET  /detections              # Query detection history
POST /alerts/rules            # Configure alert rules
```

### Minimal stable control surface (recommended)

```
GET  /streams                 # list streams
POST /streams                 # add stream source
DELETE /streams/:id           # remove stream

GET  /runs                    # list runs
POST /runs                    # start a run from a graph/skill
GET  /runs/:id                # run details

WS   /events                  # unified event stream

GET  /detections              # query detections
POST /alerts/rules            # configure rules
```

---

## Competitive Positioning (What beats OpenClaw)

- **Desktop-first approvals + audit log**: every tool/camera/screen/filesystem action is visible and approvable.
- **Visual workflow editor**: graph-native workflows with live telemetry and replay.
- **Replayable runs with artifacts**: reproducibility and debugging, not just chat logs.
- **Pluggable sources/detectors/sinks** with versioned contracts and isolation.

## Competitive Reference (Informative)

This is a positioning aid, not a claim of current implementation.

| Feature | OpenClaw | AI Studio (Target) |
|---|---:|---:|
| Webcam | ✅ | ✅ (planned) |
| RTSP / IP camera | varies | ✅ (planned) |
| Local AI inference | ✅ | ✅ (YOLOv8-ready path) |
| Desktop notifications | varies | ✅ (Tauri native) |
| Visual workflow editor | ❌ | ✅ (Canvas layer) |
| Plugin system | varies | ✅ (planned) |

---

## Priority Order (MVP-first)

1) **P0 (Now):** sidecar lifecycle + token auth + tool approval (done)
2) **P1 (Next):** run graph executor + event bus + run timeline UI
3) **P2:** video ingestion framework (RTSP/webcam/file/screen) + preview + artifacts
4) **P3:** detection pipeline + rules + desktop notifications/webhooks
5) **P4:** plugin runtime isolation + signed plugins + compatibility gates
