# Node Editor Visual Polish

**Status**: DRAFT — INCOMPLETE
**Phase**: 3C
**Priority**: P1 (next after 3B execution engine)
**Inspiration**: Blender Shader Editor (primary), Unreal Engine 5.7 Blueprints (secondary)
**References**: `docs/design-references/node-editor/` — 6 screenshots

> This spec is intentionally incomplete. It captures the visual direction based on
> Blender and Unreal reference screenshots. Will be refined before implementation.

---

## Goal

Transform the node editor from "functional React Flow demo" to "professional visual pipeline builder"
that stands alongside Blender's Shader Editor and Unreal's Blueprints in visual quality.

Current state: bright card-style nodes, anonymous handle dots, no context menu, straight lines.
Target state: dark integrated nodes, labeled sockets, right-click menu, bezier noodles.

---

## Design References

| File | What it shows |
|------|---------------|
| `blender_node.png` | Node body style, labeled sockets, inline controls, thin header |
| `blender_delete.png` | Right-click context menu with delete/disconnect |
| `UnrealBlueprint.png` | Full canvas layout, comment boxes, section annotations |
| `Unreal_move_node.png` | Close-up: labeled pins, colored dots, bezier noodles, dark body |
| `unrealRighClickNode.png` | Right-click menu: Copy, Duplicate, Break Links, Collapse, Alignment |
| `Screenshot_2025-12-23_13-17-43.png` | Our current state (before polish) |

---

## 1. Node Restyling (Blender-inspired)

### Node Body
- Dark background: `#1e1e1e` — blends with canvas, not a bright card
- Subtle 1px border: `#3a3a3a` normal, `#4488ff` glow on selection
- Corner radius: 6px
- No drop shadow in normal state, subtle glow on hover

### Node Header
- **Thin colored strip** (24px tall, not the current full block)
- Muted type colors (not saturated):
  - Input: `#2d5a27` (muted green)
  - Output: `#8a5a1e` (muted amber)
  - LLM: `#3a3a8a` (muted indigo)
  - Tool: `#8a2a5a` (muted pink)
  - Router: `#1a6a6a` (muted teal)
  - Approval: `#8a7a1a` (muted yellow)
  - Transform: `#5a3a8a` (muted purple)
  - Subworkflow: `#1a5a7a` (muted cyan)
- Contains: collapse chevron + node type icon + title
- Collapse chevron: click to fold node to header-only (show handles but hide body)

### Labeled Handles (Sockets)
- Each handle gets a text label next to it
- Input handles: dot on left, label on right
- Output handles: label on left, dot on right
- Socket dot: 8px circle, color-coded by data type:
  - `text`: `#c8c8c8` (light gray)
  - `json`: `#e8c84a` (yellow)
  - `boolean`: `#4ac84a` (green)
  - `any`: `#888888` (gray)
  - `file`: `#c8884a` (orange)
- Label font: 11px, `#a0a0a0`

Handle labels per node type:

| Node | Input handles | Output handles |
|------|--------------|----------------|
| Input | — | `value` |
| Output | `value` | — |
| LLM | `prompt` | `response` |
| Tool | `input` | `result` |
| Router | `input` | one per branch name |
| Approval | `data` | `approved`, `rejected` |
| Transform | `input` | `output` |
| Subworkflow | `input` | `output` |

---

## 2. Connection Lines

- **Bezier curves** — React Flow `edgeType: 'smoothstep'` or custom bezier
- **Color by data type** — matches socket dot color
- **Animated flow** during execution (dashed line animation on running edges)
- Thickness: 2px normal, 3px on hover
- Selected edge: brighter color + glow

---

## 3. Right-Click Context Menu

Based on Unreal's menu structure, adapted for our use case:

```
┌─────────────────────────┐
│ [Node Type] Name        │  ← header
├─────────────────────────┤
│ ✂  Delete         Del   │
│ 📋 Duplicate     Ctrl+D │
│ 🔇 Mute/Bypass          │
│ ⛓  Disconnect All       │
├─────────────────────────┤
│ ORGANIZATION            │
│ 📝 Rename          F2   │
│ 📦 Collapse              │
│ 📐 Alignment        ▸   │
│    └ Align Left/Right/Top/Bottom
├─────────────────────────┤
│ NODE COMMENT            │
│ [text field]            │
└─────────────────────────┘
```

Canvas right-click (no node selected):
```
┌─────────────────────────┐
│ ADD NODE             ▸  │  ← opens palette submenu
│ 📝 Add Comment Box      │
│ 📐 Select All    Ctrl+A │
│ 🔍 Fit View             │
├─────────────────────────┤
│ 📋 Paste         Ctrl+V │
└─────────────────────────┘
```

---

## 4. Canvas Features (Unreal-inspired)

### Comment Boxes
- Resizable colored rectangles behind nodes
- Title text at top
- Semi-transparent fill
- Used to group related nodes ("Data Preprocessing", "LLM Chain", "Output Formatting")

### Node Collapse
- Click chevron in header → body collapses, only header + handles visible
- Useful for large workflows where you want overview

### Keyboard Shortcuts
- `Del` / `Backspace` — Delete selected nodes
- `Ctrl+D` — Duplicate selected
- `Ctrl+C/V` — Copy/Paste
- `F2` — Rename selected node
- `Ctrl+A` — Select all
- `Space` — Open node search (quick add)

---

## 5. Execution State Visuals (already built in 3B, may refine)

- `idle`: default dark body
- `running`: blue border pulse + subtle glow + spinner badge
- `completed`: green border + checkmark badge + output preview
- `error`: red border + X badge + error message
- `waiting`: yellow border + hourglass (approval node)
- `skipped`: dashed gray border

---

## 6. TODO — Needs More Design Work

- [ ] Exact CSS values need tuning in browser (colors, spacing, shadows)
- [ ] Mobile/small screen behavior (probably not needed for desktop app)
- [ ] Accessibility: keyboard navigation between nodes
- [ ] Node search popup (Space key to quick-add nodes)
- [ ] Mini inline editing on node body (temperature slider, model dropdown)
- [ ] Edge reconnection UX (drag edge to new target)
- [ ] Undo/redo system for canvas operations
- [ ] Performance: virtualization for 100+ node workflows

---

## Implementation Order

1. Node body + header restyling (CSS + JSX)
2. Labeled handles with colored dots
3. Bezier connection lines
4. Right-click context menu
5. Node collapse
6. Comment boxes
7. Keyboard shortcuts
8. Inline editing refinements
