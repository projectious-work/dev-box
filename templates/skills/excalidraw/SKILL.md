---
name: excalidraw
description: Generates Excalidraw diagrams programmatically as JSON. Use when creating architecture diagrams, flowcharts, or hand-drawn-style visuals for documentation.
---

# Excalidraw

## When to Use

When the user asks to:
- Create an architecture diagram, flowchart, or system diagram
- Generate hand-drawn-style visuals for documentation
- Produce Excalidraw-compatible JSON for editing in the Excalidraw app
- Create diagrams that can be embedded in Markdown or docs

## Instructions

### 1. Excalidraw JSON Format

Excalidraw files (`.excalidraw`) are JSON with this top-level structure:

```json
{
  "type": "excalidraw",
  "version": 2,
  "source": "generated",
  "elements": [],
  "appState": {
    "viewBackgroundColor": "#ffffff",
    "gridSize": 20
  },
  "files": {}
}
```

Each element in `elements` is an object with:
- `id`: unique string (use random alphanumeric, 8+ chars)
- `type`: `"rectangle"`, `"ellipse"`, `"diamond"`, `"line"`, `"arrow"`, `"text"`, `"freedraw"`, `"image"`
- `x`, `y`: position (top-left corner)
- `width`, `height`: dimensions (for shapes)
- `strokeColor`: border color (default `"#1e1e1e"`)
- `backgroundColor`: fill color (`"transparent"` or a color)
- `fillStyle`: `"solid"`, `"hachure"`, `"cross-hatch"`
- `strokeWidth`: 1, 2, or 4
- `roughness`: 0 (architect), 1 (artist/hand-drawn), 2 (cartoonist)
- `opacity`: 0-100
- `groupIds`: array of group IDs for grouping elements
- `boundElements`: array linking text labels and arrows to shapes

### 2. Element Types

**Rectangle** (boxes, containers):
```json
{
  "id": "rect1", "type": "rectangle",
  "x": 100, "y": 100, "width": 200, "height": 80,
  "strokeColor": "#1e1e1e", "backgroundColor": "#a5d8ff",
  "fillStyle": "solid", "strokeWidth": 2, "roughness": 1,
  "roundness": { "type": 3 }
}
```

**Text** (labels, standalone text):
```json
{
  "id": "text1", "type": "text",
  "x": 140, "y": 130, "width": 120, "height": 25,
  "text": "API Server", "fontSize": 20, "fontFamily": 1,
  "textAlign": "center", "verticalAlign": "middle"
}
```

**Arrow** (connections between elements):
```json
{
  "id": "arrow1", "type": "arrow",
  "x": 300, "y": 140,
  "points": [[0, 0], [150, 0]],
  "startBinding": { "elementId": "rect1", "focus": 0, "gap": 5 },
  "endBinding": { "elementId": "rect2", "focus": 0, "gap": 5 },
  "endArrowhead": "arrow"
}
```

**Font families**: 1 = Virgil (hand-drawn), 2 = Helvetica, 3 = Cascadia (code)

### 3. Binding Text to Shapes

To label a shape, create a text element and bind it:

1. On the shape: add `"boundElements": [{"id": "textId", "type": "text"}]`
2. On the text: add `"containerId": "shapeId"` and set dimensions to fit
3. Text auto-centers within the container shape

### 4. Layout Guidelines

- **Grid alignment**: Use multiples of 20 for coordinates (matches default grid)
- **Spacing**: 80-120px between connected elements
- **Flow direction**: Left-to-right for flows/pipelines; top-to-bottom for hierarchies
- **Grouping**: Related elements share a `groupIds` entry
- **One idea per diagram**: If it needs scrolling, split it into multiple diagrams
- **Label everything**: Every shape gets a text binding or a bound text element
- **Font sizes**: Titles 28-32px, labels 20-24px, annotations 16-18px
- **Consistent sizing**: Similar elements should have the same width and height
- **Readable at 1x zoom**: Minimum text size 16px

### 5. Semantic Color Palette

Use these colors consistently within a diagram:

| Role | Fill (`backgroundColor`) | Stroke (`strokeColor`) |
|---|---|---|
| Primary / action | `#a5d8ff` | `#1971c2` |
| Secondary / support | `#d0bfff` | `#6741d9` |
| Success / positive | `#b2f2bb` | `#2f9e44` |
| Warning / caution | `#ffec99` | `#e8590c` |
| Danger / error | `#ffc9c9` | `#e03131` |
| Neutral / container | `#e9ecef` | `#495057` |

Set `"fillStyle": "solid"` for filled shapes.

### 6. Diagram Types

**Architecture Diagram:**
- Use rectangles for services, databases (with rounded corners)
- Arrows for data flow with labels
- Group related services with a larger rectangle container
- Color-code by layer (frontend = blue, backend = green, data = yellow)

**Flowchart:**
- Diamonds for decisions, rectangles for processes, ellipses for start/end
- Consistent arrow direction (top-to-bottom or left-to-right)
- Label decision arrows with "Yes"/"No"

**Sequence-style Diagram:**
- Vertical arrangement with horizontal arrows
- Label arrows with operation names
- Use dashed arrows for responses

### 7. Embedding in Documentation

- Save as `.excalidraw` file for editable source
- Export to SVG or PNG for embedding in Markdown
- In MkDocs or similar: embed the SVG directly or use `![diagram](diagram.svg)`
- For GitHub: `.excalidraw.png` files render directly in Markdown
- Excalidraw VS Code extension renders `.excalidraw` files in-editor

## References

- `references/json-schema.md` --- Full element type reference, properties, and valid file template

## Examples

**User:** "Create an architecture diagram for a web app with React frontend, Node API, and PostgreSQL"
**Agent:** Generates Excalidraw JSON with three labeled rectangles arranged
left-to-right, connected by arrows labeled "HTTP/REST" and "SQL". Uses blue for
frontend, green for API, yellow for database. Saves as `.excalidraw` file.

**User:** "Make a flowchart for this decision process"
**Agent:** Creates a top-to-bottom flowchart with diamond decision nodes, rectangle
process nodes, and labeled arrows. Uses consistent colors and grid-aligned positioning.

**User:** "Convert this Mermaid diagram to Excalidraw"
**Agent:** Parses the Mermaid syntax, maps nodes to Excalidraw rectangles/diamonds,
creates arrows for edges, and arranges with proper spacing. Produces editable
`.excalidraw` JSON.
