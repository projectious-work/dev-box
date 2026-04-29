# Excalidraw JSON Schema Reference

Minimal reference for generating valid `.excalidraw` element JSON.

## Element Types

| Type | Purpose | Key extra properties |
|---|---|---|
| `rectangle` | Boxes, containers | `roundness` |
| `ellipse` | Circles, ovals | --- |
| `diamond` | Decision nodes | --- |
| `text` | Standalone labels | `text`, `fontSize`, `fontFamily`, `textAlign` |
| `arrow` | Directed connections | `points`, `startBinding`, `endBinding` |
| `line` | Undirected lines | `points` |
| `freedraw` | Freehand strokes | `points` |
| `image` | Embedded images | `fileId` (references `files` dict) |

## Common Properties (all elements)

```
id              string    Unique identifier (use descriptive IDs like "server-box")
type            string    One of the types above
x, y            number    Position (top-left corner)
width, height   number    Bounding box size
angle           number    Rotation in radians (default 0)
strokeColor     string    Border/line color (hex)
backgroundColor string    Fill color (hex, or "transparent")
fillStyle       string    "solid" | "hachure" | "cross-hatch"
strokeWidth     number    1 (thin), 2 (normal), 4 (bold)
strokeStyle     string    "solid" | "dashed" | "dotted"
roughness       number    0 (architect) | 1 (artist) | 2 (cartoonist)
opacity         number    0-100
roundness       object    { "type": 3 } for rounded corners, null for sharp
isDeleted       boolean   Soft-delete flag (default false)
boundElements   array     [{ "id": "...", "type": "text|arrow" }]
groupIds        array     Group IDs for visually grouping elements
```

## Text Properties

```
text            string    The displayed text
fontSize        number    Size in px (16-32 typical)
fontFamily      number    1 (Virgil/hand), 2 (Helvetica), 3 (Cascadia/mono)
textAlign       string    "left" | "center" | "right"
verticalAlign   string    "top" | "middle"
containerId     string    ID of parent container (for bound text)
```

When text is bound to a container:
- Text element: set `containerId` to the container's `id`
- Container element: add `{ "id": "<text-id>", "type": "text" }` to `boundElements`
- Text `width`/`height` should fit inside the container

## Arrow / Line Properties

```
points          array     [[0,0], [dx,dy]] - relative to x,y
startBinding    object    { "elementId": "id", "focus": 0, "gap": 4 }
endBinding      object    { "elementId": "id", "focus": 0, "gap": 4 }
startArrowhead  string    null | "arrow" | "dot" | "bar"
endArrowhead    string    null | "arrow" | "dot" | "bar"
```

- `focus`: -1 to 1, controls where the arrow attaches (0 = center)
- `gap`: space between arrowhead and target element border
- Target elements must list the arrow in their `boundElements`

## Grouping

Use `groupIds: ["group-1"]` on elements to visually group them. Shared
`groupIds` values link elements into a group.

## Minimal Valid File

```json
{
  "type": "excalidraw",
  "version": 2,
  "source": "generated",
  "elements": [
    {
      "id": "box1",
      "type": "rectangle",
      "x": 100,
      "y": 100,
      "width": 200,
      "height": 80,
      "strokeColor": "#1971c2",
      "backgroundColor": "#a5d8ff",
      "fillStyle": "solid",
      "strokeWidth": 2,
      "roughness": 1,
      "opacity": 100,
      "angle": 0,
      "groupIds": [],
      "roundness": { "type": 3 },
      "boundElements": [
        { "id": "label1", "type": "text" }
      ],
      "isDeleted": false
    },
    {
      "id": "label1",
      "type": "text",
      "x": 140,
      "y": 125,
      "width": 120,
      "height": 30,
      "text": "Service A",
      "fontSize": 20,
      "fontFamily": 1,
      "textAlign": "center",
      "verticalAlign": "middle",
      "containerId": "box1",
      "strokeColor": "#1e1e1e",
      "backgroundColor": "transparent",
      "fillStyle": "solid",
      "strokeWidth": 1,
      "roughness": 1,
      "opacity": 100,
      "angle": 0,
      "groupIds": [],
      "boundElements": null,
      "isDeleted": false
    }
  ],
  "appState": {
    "viewBackgroundColor": "#ffffff",
    "gridSize": 20
  },
  "files": {}
}
```

## Common Patterns

### Container with Children
Create a large rectangle as a "container" with smaller elements inside.
Bind arrows to the container, not to the children.

### Labeled Arrow
Create an arrow between two shapes, then add a text element positioned
at the arrow's midpoint. The text does not need `containerId` for arrows;
just position it manually near the center of the arrow path.

### Decision Diamond
Use `type: "diamond"` with bound text. Connect "Yes" and "No" arrows
from different sides. Label each arrow with a nearby text element.
