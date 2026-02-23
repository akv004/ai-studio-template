# README Review — Open-Source First Impression
**Date**: 2026-02-22
**Reviewer**: Gemini 3 Pro
**Status**: RESOLVED — all findings accepted, README rewritten

### Findings Table
| Check | Priority | Verdict | Findings |
|-------|----------|---------|----------|
| Outdated numbers & phase | HIGH | FAIL | The README currently states 8 node types, 10 templates, 31 tests, and lists Phase 3 as "In progress". The reality is much stronger: 16 node types, 13 bundled templates, 129+ Rust tests, and Phase 4C is actively in progress. |
| Missing features | HIGH | FAIL | Many of the most impressive recently-added features are absent. Streaming output, Live Workflow mode, Data I/O nodes (File Glob, Iterator, Aggregator), user-saved templates, Rich Output, and multi-provider vision support are completely missing from the feature list. |
| Competitor naming | HIGH | FAIL | The "Why AI Studio?" table explicitly names ChatGPT, Claude, Cursor, and LangFlow. This should be rewritten to focus on capability-based comparisons (e.g. "Cloud Chat Apps", "Code Editors", "Web-based Node Editors") to maintain a professional, self-assured tone without directly calling out alternatives. |
| First 10 seconds | MED | WARN | The hero section is decent but lacks a "wow" artifact. A static screenshot is good, but a `[demo.gif placeholder]` showing a live workflow execution (streaming + Inspector update) would drastically increase the 10-second conversion rate. |
| Feature hierarchy | MED | WARN | The "Agent Inspector" is listed first, but the "Node Editor" has clearly become the flagship core product experience. The hierarchy should prioritize the Node Editor, followed by the Inspector, and highlight Live Mode and the upcoming RAG Knowledge Base. |
| Scannability | LOW | PASS | The README is generally well-structured with clear headings and a good table of contents. Adding a few more visual aids or badges for the newer features would improve it further. |

### Actionable Checklist
- [ ] Update all numbers in the README: 16 node types, 13 templates, 129+ tests.
- [ ] Update the Roadmap section to mark Phase 3 as Done and Phase 4 as "In progress".
- [ ] Add a new high-level feature section for "Live Workflows & Streaming" highlighting the real-time execution features.
- [ ] Add a section or bullet points for the new Data I/O capabilities (File Glob, Iterator, Aggregator, Vision).
- [ ] Rewrite the "Why AI Studio?" table to remove explicit competitor names (replace with generic categories like "SaaS Chat UIs", "Cloud Workflow Builders").
- [ ] Reorder the Features section to put "Visual Node Editor" first, as it is the primary interaction paradigm.
- [ ] Add a placeholder for a demo GIF (`![Demo](docs/screenshots/demo.gif)`) right below the hero description.
- [ ] Mention the upcoming local-first RAG Knowledge Base functionality under a "Coming Soon" or "Roadmap" highlight.

### Notes
**Suggested rewrite for the Comparison Table (Why AI Studio?):**

| Feature | SaaS Chat UIs | Cloud Workflow Builders | **AI Studio** |
|---------|---------------|-------------------------|---------------|
| **Full Inspector (traces, replay, branch)** | No | No | **Yes** |
| **Visual pipeline builder** | No | Yes | **Yes (+ local engine)** |
| **Hybrid intelligence (auto-pick model)** | No | No | **Yes** |
| **Local-first (data stays on machine)** | No | Partial | **Yes (SQLite)** |
| **MCP-native tool system** | Limited | No | **Full Support** |
| **Open source & Desktop native** | No | No | **Yes** |
