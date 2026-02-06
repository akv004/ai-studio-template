# AI Studio - Future Capabilities Spec

> **Purpose:** Framework-level capabilities to support, not immediate implementation.

---

## Video Stream Ingestion

### Supported Sources (Future)
| Source Type | Protocol | Example |
|-------------|----------|---------|
| Webcam | USB/V4L2 | Built-in laptop camera |
| IP Camera | RTSP | `rtsp://192.168.1.100:554/stream` |
| Security NVR | RTSP/ONVIF | Hikvision, Dahua, Reolink |
| Video File | File path | `/path/to/recording.mp4` |
| Screen Capture | Desktop API | Monitor/window capture |

### Architecture
```
┌─────────────────────────────────────────────────────────┐
│  VIDEO INGESTION LAYER (Python Sidecar)                 │
│                                                         │
│  VideoSource (abstract)                                 │
│    ├── WebcamSource (OpenCV VideoCapture)               │
│    ├── RTSPSource (OpenCV/FFmpeg)                       │
│    ├── FileSource (OpenCV)                              │
│    └── ScreenSource (mss/Pillow)                        │
│                                                         │
│  → Frames → AI Pipeline → Detections → Events          │
└─────────────────────────────────────────────────────────┘
```

---

## AI Detection Pipeline

### Supported Models (Future)
| Task | Models | Use Case |
|------|--------|----------|
| Object Detection | YOLOv8, YOLOv10, RT-DETR | Person, vehicle, package |
| Face Recognition | InsightFace, ArcFace | Identity matching |
| Motion Detection | Frame diff, optical flow | Activity trigger |
| Anomaly Detection | Autoencoders, isolation forest | Unusual behavior |
| OCR | PaddleOCR, EasyOCR | License plates, text |

### Event System
```
Detection → Rule Engine → Alert
                ↓
    - Desktop notification (Tauri)
    - Sound alert
    - Webhook (POST to URL)
    - Telegram/Discord bot
    - Email (SMTP)
    - Log to file/database
```

---

## Extensibility Points

### Plugin Architecture (Future)
```
plugins/
├── sources/           # Custom video sources
│   └── my_camera.py
├── detectors/         # Custom AI models
│   └── my_detector.py
├── alerts/            # Custom alert handlers
│   └── my_webhook.py
└── ui/                # Custom UI panels
    └── MyPanel.tsx
```

### API Endpoints (Planned)
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

---

## Competitive Reference

| Feature | OpenClaw | AI Studio (Target) |
|---------|----------|-------------------|
| Webcam | ✅ | ✅ (planned) |
| RTSP/IP Camera | ❓ | ✅ (planned) |
| Local AI inference | ✅ | ✅ (YOLOv8 ready) |
| Desktop notifications | ❓ | ✅ (Tauri native) |
| Visual workflow editor | ❌ | ✅ (Canvas layer) |
| Plugin system | ❓ | ✅ (planned) |

---

## Priority Order

1. **P0 (Now):** Tool approval, sidecar lifecycle ✅ Done
2. **P1 (Next):** Chat → Agent → Tools flow
3. **P2:** Video ingestion framework
4. **P3:** Detection pipeline + alerts
5. **P4:** Plugin architecture
