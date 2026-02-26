# Webhook Trigger Toolbar — UX Review

**Recommended reviewer:** Codex (GPT) — UI/UX, code-level improvements
**Date:** 2026-02-25
**Status:** OPEN

## What to Review

The webhook trigger toolbar buttons in the workflow canvas. The current implementation has UX problems flagged by the user:

1. **Too many confusing buttons**: Run, Go Live, Start, Test — four action buttons that overlap in purpose
2. **"Test" button sends a fake/mock request** — the user finds this misleading. What should it do instead? Should it exist at all?
3. **Button styling doesn't feel professional** — the Arm/Start/Stop/Test buttons don't match the visual quality of Run and Go Live
4. **Unclear workflow for webhook users** — When you have a webhook workflow, should Run/Go Live still be visible? Or should Start (webhook listen) replace them?

## Files to Read

1. `apps/ui/src/app/pages/workflow/WorkflowCanvas.tsx` — lines 919-953 (toolbar buttons), lines 680-738 (handler functions)
2. `apps/ui/src/app/pages/workflow/nodes/WebhookTriggerNode.tsx` — the canvas node
3. `apps/ui/src/app/pages/workflow/NodeConfigPanel.tsx` — search for `webhook_trigger` section

## Screenshot

The toolbar currently shows: `[Save] [Export] [Template] | [Run] | [Go Live] [Settings] | [Start] [Test]`

For a webhook workflow, the user is confused about:
- What's the difference between Run and Start?
- Why would I "Go Live" when I have a webhook?
- What does "Test" do vs "Run"?

## What to Look For

1. **Button consolidation**: Should webhook workflows show a different toolbar? (e.g., hide Run/Go Live when webhook node present, show only Start/Stop)
2. **Test button replacement**: Instead of a fake mock request, what's the better UX? Options:
   - Remove Test entirely — user uses curl or Postman
   - Replace with "Copy curl" button — copies a curl command to clipboard
   - Show a small inline request builder (path, body, send)
3. **Visual hierarchy**: How should Start/Stop look relative to Run/Go Live? Same style? Different placement?
4. **State clarity**: When the webhook server is running, how should the toolbar communicate this? A status indicator? Change the whole toolbar color?
5. **Naming**: Is "Start/Stop" the right label? Alternatives: Listen/Stop, Enable/Disable, Activate/Deactivate

## Current Behavior

- **Run**: Opens input dialog, runs workflow once with provided inputs (standard execution)
- **Go Live**: Runs workflow repeatedly on an interval (live/continuous mode)
- **Start**: Creates trigger record + starts Axum HTTP server on port 9876 + registers webhook route
- **Stop**: Stops listening, disarms the trigger
- **Test**: Calls `test_trigger` IPC — fires one mock execution through the webhook handler

## Output Format

Provide specific code-level recommendations:
- Which buttons to keep/remove/rename
- Exact className and styling changes
- Any new UI elements (status badges, copy buttons, etc.)
- Consider the user flow: "I built a webhook workflow, now what do I click?"
