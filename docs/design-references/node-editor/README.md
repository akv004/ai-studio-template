# Node Editor Visual Design Reference

**Status**: DRAFT — visual polish spec for Phase 3C
**Inspiration**: Blender Shader Editor (primary), Unreal Blueprints (secondary)

## Reference Screenshots

| File | Source | What to study |
|------|--------|---------------|
| `blender_node.png` | Blender 4.x Shader Editor | Node body, header, labeled sockets, inline controls |
| `blender_delete.png` | Blender context menu | Right-click menu for delete/disconnect |
| `UnrealBlueprint.png` | Unreal Engine 5.7 Blueprints | Canvas layout, comment boxes, exec flow pins |
| `Screenshot_2025-12-23_13-17-43.png` | AI Studio (current) | Our current state for comparison |

## Design Analysis: Blender vs Unreal vs Ours

### Blender Shader Editor (PRIMARY inspiration)

Blender is the better model for AI Studio because:
- **Cleaner, more minimal** — fewer visual elements, better signal-to-noise
- **Dark body blends with canvas** — nodes feel integrated, not floating cards
- **Labeled sockets with colored dots** — each handle has a name + type color
- **Inline value editing** — sliders and fields directly on the node body
- **Thin colored header** — just a slim top bar with collapse arrow + title
- **Collapsible sections** — nodes can fold to just the header
- **Right-click context menu** — Delete, Duplicate, Mute, Disconnect
- **Noodle curves** — smooth bezier connections with type-based colors

Key visual properties:
- Node bg: `#303030` (dark gray, ~15% lighter than canvas `#1a1a1a`)
- Header: thin strip, type-colored (green=shader, brown=texture, red=output)
- Socket dots: ~8px, color = data type (yellow=color, gray=float, green=shader, purple=vector)
- Socket labels: 11px, left-aligned for inputs, right-aligned for outputs
- Body text: light gray `#c0c0c0`, muted labels `#888888`
- Corners: ~4px radius (subtle, not pill-shaped)
- Border: none in normal state, white outline when selected

### Unreal Blueprints (secondary reference)

Good for:
- **Comment boxes** — large titled regions grouping related nodes
- **Exec flow pins** — triangle-shaped pins for execution order (white)
- **Compact function nodes** — small one-liner nodes for simple ops
- **Section labels** on canvas as documentation

Less relevant because:
- Busier visual style (more for game logic complexity)
- Two pin types (exec + data) adds complexity we don't need
- Color coding is less consistent than Blender

### Our Current State (problems to fix)

1. **Node body too bright** — stands out like a card, doesn't blend with canvas
2. **No socket labels** — handles are anonymous dots, must click to see what they accept
3. **No inline editing** — all config in separate right panel (context switching)
4. **No right-click menu** — can't delete/duplicate from canvas directly
5. **Header too chunky** — full colored block takes too much visual weight
6. **No node collapse** — can't minimize nodes to save canvas space
7. **No comment/group boxes** — can't annotate regions of the workflow
8. **Connection lines are straight** — should be curved bezier noodles

## Implementation Plan (Phase 3C visual polish)

### Priority 1 — Node Restyling (Blender-inspired)
- [ ] Dark node body (`--bg-tertiary` or custom `#2a2a2a`)
- [ ] Thin colored header strip (8px tall, not full block)
- [ ] Labeled handles — text next to each socket dot
- [ ] Socket dot colors per data type (text=white, json=yellow, boolean=green, any=gray)
- [ ] Subtle border: 1px `#404040`, white glow on select
- [ ] Rounded corners: 6px
- [ ] Node title in header with collapse chevron

### Priority 2 — Interaction Improvements
- [ ] Right-click context menu: Delete, Duplicate, Disconnect All, Mute/Bypass
- [ ] Node collapse (click chevron → show only header + handles)
- [ ] Bezier curved connection lines (React Flow supports `type: 'smoothstep'` or `'bezier'`)
- [ ] Connection line colors based on data type

### Priority 3 — Canvas Features (Unreal-inspired)
- [ ] Comment boxes / group frames for annotating regions
- [ ] Canvas-level text annotations
- [ ] Snap to grid option
- [ ] Auto-layout / auto-arrange button

### Priority 4 — Inline Editing (Blender-inspired)
- [ ] Simple fields editable directly on node body (name, temperature slider)
- [ ] Keep config panel for advanced/overflow settings
- [ ] Collapsible sections within nodes for advanced options

## Color Palette (Blender-inspired, adapted for AI Studio)

```
Canvas background:  #0d0d0d (existing --bg-primary)
Node body:          #1e1e1e (between canvas and current cards)
Node header (by type):
  Input:            #2d5a27 (muted green)
  Output:           #8a5a1e (muted amber)
  LLM:              #3a3a8a (muted indigo)
  Tool:             #8a2a5a (muted pink)
  Router:           #1a6a6a (muted teal)
  Approval:         #8a7a1a (muted yellow)
  Transform:        #5a3a8a (muted purple)
  Subworkflow:      #1a5a7a (muted cyan)

Socket colors (by data type):
  text:             #c8c8c8 (light gray)
  json:             #e8c84a (yellow)
  boolean:          #4ac84a (green)
  any:              #888888 (gray)
  file:             #c8884a (orange)

Selection:          #4488ff (blue glow)
Error:              #ff4444
Running:            #4488ff (blue pulse)
Completed:          #44cc44
```
