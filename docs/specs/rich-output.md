# Rich Output & Visualization

**Status**: PLANNED
**Phase**: 5A (demo-ready polish)
**Priority**: P1 — transforms raw text/JSON into presentable results
**Author**: AI Studio PM
**Date**: 2026-02-21

---

## Problem Statement

All node outputs render as plain text or raw JSON. When an LLM returns a markdown table, the user sees `| col1 | col2 |` markup instead of a rendered table. When a workflow outputs structured data, there's no way to visualize it as a chart or formatted table. This makes AI Studio feel like a terminal tool instead of a visual IDE.

---

## Output Rendering Modes

Every Output node and node preview should auto-detect content type and render appropriately. Users can also force a specific mode.

### Auto-Detection Logic

```
Content → detect type:
  1. Starts with `{` or `[` → JSON mode
  2. Contains `| --- |` pattern → Markdown table mode
  3. Contains `# ` or `## ` headers → Markdown mode
  4. Contains ```code fences → Code mode
  5. Starts with `data:image/` or binary PNG/JPEG header → Image mode
  6. Default → Plain text mode
```

### Render Modes

| Mode | Trigger | Rendering |
|------|---------|-----------|
| **Plain text** | Default | Monospace text, line wrapping |
| **Markdown** | `#` headers, `**bold**`, lists | Full markdown rendering (headers, bold, italic, lists, links) |
| **Table** | JSON array of objects or markdown table | Sortable/filterable table with column headers |
| **Code** | \`\`\` fences with language tag | Syntax-highlighted code block |
| **JSON** | Valid JSON object/array | Collapsible tree view with syntax highlighting |
| **Image** | Base64 image data or file path | Rendered image with zoom/pan |
| **Chart** | Explicit chart config or numeric table data | Bar/line/pie chart |

---

## Markdown Rendering

### What to render
- Headers (h1-h6)
- Bold, italic, strikethrough
- Ordered and unordered lists
- Links (open in external browser via Tauri shell)
- Inline code and code blocks
- Block quotes
- Horizontal rules
- Tables (see Table section below)

### Library
Use `react-markdown` + `remark-gfm` (GitHub-flavored markdown) — already popular, lightweight, handles tables natively.

### Security
- Sanitize HTML in markdown output (no `<script>`, `<iframe>`, `<form>`)
- Links open in external browser only (never in-app navigation)
- Images: only render base64 data URIs and local file paths (no external URL fetching)

---

## Table Rendering

### Auto-table from JSON

When output is a JSON array of objects, render as a table:

```json
[
  {"name": "Alice", "score": 95, "grade": "A"},
  {"name": "Bob", "score": 82, "grade": "B"},
  {"name": "Carol", "score": 91, "grade": "A"}
]
```

Renders as:

```
┌────────┬───────┬───────┐
│ name   │ score │ grade │
├────────┼───────┼───────┤
│ Alice  │ 95    │ A     │
│ Bob    │ 82    │ B     │
│ Carol  │ 91    │ A     │
└────────┴───────┴───────┘
[Sort ▼]  [Filter]  [Copy]  [Export CSV]
```

### Table Features
- **Sort**: Click column header to sort asc/desc
- **Filter**: Per-column text filter
- **Copy**: Copy table as TSV to clipboard
- **Export**: Download as CSV
- **Pagination**: For tables with >50 rows, paginate (25/50/100 per page)
- **Column resize**: Drag column borders

### Markdown Tables

Detect markdown table syntax and render as a proper HTML table:
```markdown
| Model | Latency | Cost |
|-------|---------|------|
| GPT   | 0.8s    | $0.003 |
| Claude| 1.2s    | $0.004 |
```

---

## Code Rendering

### Syntax Highlighting

When output contains fenced code blocks with a language tag:

````
```python
def hello():
    print("world")
```
````

Render with syntax highlighting using `Prism.js` or `highlight.js`:
- Support: Python, JavaScript, TypeScript, Rust, SQL, JSON, YAML, Bash, HTML, CSS
- Line numbers
- Copy button (top-right corner)
- Word wrap toggle

### Standalone Code Output

If the entire output is code (detected by content heuristics or user-selected mode), render the whole output as a highlighted code block.

---

## JSON Tree View

For complex JSON objects, render as a collapsible tree:

```
▼ {
    "results": ▼ [
        ▼ {
            "model": "claude-sonnet",
            "output": "Quantum computing...",
            "scores": ▶ { accuracy: 9, ... }
        },
        ▶ { "model": "gemini-flash", ... }
    ],
    "winner": "claude-sonnet"
  }
```

Features:
- Collapse/expand all levels
- Click to copy any value or subtree
- Search within JSON (highlight matching keys/values)
- Path breadcrumb: `results[0].scores.accuracy`

---

## Image Rendering

### Inline Image Display

When output contains base64 image data (from vision workflows, chart generation, etc.):

```
┌──────────────────────────────┐
│  Output · Image              │
│  ┌────────────────────────┐  │
│  │                        │  │
│  │   [rendered image]     │  │
│  │                        │  │
│  └────────────────────────┘  │
│  1024×768 · PNG · 245 KB     │
│  [Zoom] [Save] [Copy]       │
└──────────────────────────────┘
```

Features:
- Auto-fit to node/panel width
- Click to zoom (modal with pan/zoom)
- Save to file
- Copy to clipboard

### Multi-Image Gallery

When output is an array of images (from batch vision workflows):
- Thumbnail grid (4 per row)
- Click to expand any image
- Navigation arrows between images

---

## Chart Rendering

### Chart Node (new output mode, not a new node type)

When numeric tabular data is detected, offer a "View as Chart" toggle:

**Auto-chart heuristics**:
- JSON array of objects with at least one string column and one numeric column
- First string column → X axis labels
- Numeric columns → Y axis series

**Chart types**:
| Type | When to use | Auto-detect |
|------|-------------|-------------|
| Bar | Categorical comparison | Default for <20 items |
| Line | Time series or trends | When X axis looks like dates |
| Pie | Proportions | When single numeric column sums to ~100 |

**Library**: `recharts` (React-native, lightweight, already in React ecosystem) or `Chart.js` via `react-chartjs-2`.

**Chart config** (in Output node or Display node):
| Field | Type | Default | Description |
|-------|------|---------|-------------|
| chartType | enum | auto | auto / bar / line / pie / none |
| xColumn | string | auto | Column for X axis |
| yColumns | array | auto | Columns for Y axis (multi-series) |
| title | string | — | Chart title |

**Example**:

Input data:
```json
[
  {"month": "Jan", "revenue": 45000, "costs": 32000},
  {"month": "Feb", "revenue": 52000, "costs": 34000},
  {"month": "Mar", "revenue": 48000, "costs": 31000}
]
```

Renders as a grouped bar chart with month on X axis, revenue and costs as two bar series.

---

## Output Node Config Changes

Add `renderMode` to Output node and Display node config:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| renderMode | enum | auto | auto / text / markdown / table / code / json / image / chart |
| codeLanguage | string | — | For code mode: language hint |
| chartConfig | json | — | For chart mode: type, axes, title |

---

## Node Preview Integration

The inline node preview (visible on canvas) also benefits from rich rendering:

| Content type | Preview rendering |
|-------------|-------------------|
| Short text (<100 chars) | Full text inline |
| Long text | First 3 lines + "..." |
| JSON object | First 2 key-value pairs |
| JSON array | First 2 items + count |
| Table | First 3 rows, 3 columns |
| Image | Thumbnail (64px) |
| Code | First 3 lines, highlighted |

Full rendering appears in:
1. Config panel → Output section
2. Expanded node view (double-click)
3. Inspector → Event detail

---

## Implementation Plan

### Phase 1: Markdown + Code rendering (1 session)
- [ ] Install `react-markdown` + `remark-gfm`
- [ ] OutputPreview component: auto-detect markdown, render
- [ ] Code block syntax highlighting (Prism.js)
- [ ] Copy button for code blocks
- [ ] Sanitization (no script/iframe)

### Phase 2: Table rendering (1 session)
- [ ] JSON array → auto-table detection
- [ ] Sortable columns
- [ ] Filter per column
- [ ] Export CSV + copy TSV
- [ ] Pagination for large tables
- [ ] Markdown table rendering

### Phase 3: JSON tree view (1 session)
- [ ] Collapsible JSON tree component
- [ ] Search within JSON
- [ ] Copy subtree
- [ ] Path breadcrumb

### Phase 4: Image + Chart (1 session)
- [ ] Base64 image rendering with zoom/save
- [ ] Multi-image gallery view
- [ ] `recharts` integration for basic charts (bar/line/pie)
- [ ] Auto-chart detection from numeric data
- [ ] Chart type selector in config

---

## Dependencies

| Feature | New Dependency | Size |
|---------|---------------|------|
| Markdown | `react-markdown` + `remark-gfm` | ~50KB |
| Code highlighting | `prismjs` or `highlight.js` | ~30KB (with common languages) |
| Charts | `recharts` | ~100KB |
| JSON tree | Custom component (no dep needed) | — |
| Image zoom | Existing CSS transforms | — |

Total new JS bundle: ~180KB (acceptable for a desktop app).
