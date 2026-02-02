# AI Agent Roadmap

Feature roadmap to achieve OpenClaw-level capabilities.

## Current State ✅

| Feature | Status | Notes |
|---------|--------|-------|
| Multi-provider LLM | ✅ Done | Ollama, Anthropic, OpenAI |
| Conversation Memory | ✅ Done | ChatService with history |
| Telegram Bot | ✅ Done | Full command support |
| Shell Commands | ✅ Done | sandboxed/restricted/full modes |
| Filesystem Access | ✅ Done | Read/write/delete with safety |
| Browser Automation | ✅ Done | Playwright-based |
| Docker Compose | ✅ Done | GPU support |

---

## Phase 1: Additional Channels (Priority: High)

### WhatsApp Integration
- Use `whatsapp-web.js` or Baileys library
- Same `ChatService` backend, different transport
- Estimated: ~150 lines of code

### Discord Bot
- Use `discord.py` library
- Support DM and server channels
- Slash commands (`/ask`, `/clear`, `/provider`)

### Slack Integration
- Use Slack Bolt SDK
- App mentions and DMs
- Thread-aware conversations

---

## Phase 2: Advanced Tools (Priority: Medium)

### Voice/Speech
- **Wake Word Detection**: Picovoice Porcupine
- **Speech-to-Text**: Whisper (local) or cloud
- **Text-to-Speech**: ElevenLabs or Coqui TTS
- Integration with Telegram voice messages

### Screen/Camera Access
- Screen capture via Pillow/mss
- Webcam access via OpenCV
- Send screenshots to LLM for analysis

### Clipboard Integration
- Read/write system clipboard
- Watch clipboard changes
- Auto-process copied content

---

## Phase 3: Automation (Priority: Medium)

### Cron/Scheduled Tasks
- Built-in scheduler (APScheduler)
- Natural language scheduling ("remind me at 5pm")
- Recurring tasks

### Webhooks
- Inbound webhook endpoints
- Trigger agent actions from external services
- GitHub, JIRA, etc. integration

### Email/Gmail
- Gmail API integration
- Process incoming emails
- Draft/send responses

---

## Phase 4: Skills System (Priority: Low)

### Plugin Architecture
```python
class Skill:
    name: str
    description: str
    
    def execute(self, context: SkillContext) -> SkillResult:
        pass
```

### Skill Registry
- Local skills directory
- Remote skill hub (like ClawHub)
- Hot-reload without restart

### Built-in Skills
- `web_search`: DuckDuckGo/Google search
- `calculator`: Math expressions
- `weather`: Weather API
- `news`: News aggregation
- `translate`: Language translation

---

## Phase 5: Multi-Agent (Priority: Future)

### Agent Routing
- Route different channels to different agents
- Per-user agent configurations
- Shared context pool

### Agent-to-Agent Communication
- Agents can call other agents
- Specialist delegation (coding agent, research agent)

---

## Implementation Priority

| Phase | Feature | Effort | Impact |
|-------|---------|--------|--------|
| 1 | WhatsApp | Medium | High |
| 1 | Discord | Low | Medium |
| 2 | Voice | High | High |
| 2 | Screen capture | Low | Medium |
| 3 | Cron jobs | Low | Medium |
| 4 | Skills system | High | High |
| 5 | Multi-agent | Very High | Medium |

---

## Quick Wins (Can Add Today)

1. **URL Fetching Tool** - Fetch and parse web content
2. **Image Generation** - DALL-E/Stable Diffusion integration
3. **Code Execution** - Python/Node sandbox
4. **Memory/Notes** - Persistent key-value storage
