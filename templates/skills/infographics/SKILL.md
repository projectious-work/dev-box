---
name: infographics
description: Creates data-driven infographics and charts as SVG. Use when visualizing data, creating charts, or designing informational graphics.
---

# Infographics

## When to Use

When the user asks to:
- Create a chart, graph, or data visualization
- Design an infographic or visual summary
- Convert data into a visual format
- Choose the right chart type for their data
- Review a visualization for clarity or accuracy

## Instructions

### 1. SVG Generation Principles

- Generate standalone SVG files with `xmlns="http://www.w3.org/2000/svg"`
- Set explicit `viewBox` for responsive scaling (e.g., `viewBox="0 0 800 600"`)
- Use `width="100%"` and `height="auto"` for embedding flexibility
- Group related elements with `<g>` and use `transform` for positioning
- Define reusable styles in `<defs>` and `<style>` blocks
- Use semantic IDs and classes for maintainability
- Keep text as `<text>` elements (not paths) for accessibility and editability

### 2. Data-to-Visual Mapping

1. **Identify the message**: What should the viewer learn from this graphic?
2. **Choose the encoding**: Position, length, angle, area, color, shape
3. **Apply visual hierarchy**: Most important data gets the strongest visual treatment
4. **Add context**: Title, axis labels, legend, source attribution
5. **Simplify**: Remove anything that doesn't support the message

Encoding effectiveness (most to least accurate for quantitative data):
Position > Length > Angle > Area > Color saturation > Shape

### 3. Chart Type Selection

| Data Relationship | Chart Type |
|---|---|
| Change over time | Line chart, area chart |
| Comparison of categories | Bar chart (horizontal for many categories) |
| Part-to-whole | Stacked bar, treemap (avoid pie charts for > 5 slices) |
| Distribution | Histogram, box plot, violin plot |
| Correlation | Scatter plot, bubble chart |
| Ranking | Horizontal bar chart, lollipop chart |
| Flow / process | Sankey diagram, flowchart |
| Geographic | Choropleth map, symbol map |
| Hierarchy | Treemap, sunburst |

### 4. Visual Design

- **Color palette**: Use 3-5 colors max; ensure they work in grayscale
- **Typography**: One sans-serif font; size hierarchy for title > subtitle > labels > annotations
- **Whitespace**: Leave generous margins and padding; ample whitespace prevents clutter
- **Grid lines**: Light gray, thin; remove when not needed
- **Data labels**: Add directly to chart elements when space permits (reduces legend lookups)
- **Aspect ratio**: ~16:9 for presentations, ~4:3 for documents, ~1:1 for social media
- **Mobile-first**: Over 60% of content is consumed on mobile; prefer vertical layout
- **One focal point**: Visuals with one clear focal point outperform complex ones by ~2x in retention

### 5. Accessibility

- Minimum 4.5:1 contrast ratio for text, 3:1 for large text and UI elements
- Never rely solely on color to convey information (add patterns, labels, or shapes)
- Include `<title>` and `<desc>` elements for screen readers
- Use `role="img"` and `aria-label` on the SVG element

### 6. Common Pitfalls to Avoid

- **Truncated y-axis**: Starting at non-zero exaggerates differences
- **3D effects**: Distort perception of values; always use 2D
- **Pie charts**: Hard to compare; use bar charts instead for precision
- **Dual y-axes**: Confusing; use small multiples instead
- **Excessive decoration**: Chartjunk distracts from data
- **Poor contrast**: Light colors on white backgrounds
- **Missing units**: Always label what the numbers mean
- **Rainbow colormaps**: Perceptually non-uniform; use sequential or diverging palettes

### 7. Alternative Formats

When SVG is not the right fit, consider:

- **Mermaid**: Use for flowcharts, sequence diagrams, and Gantt charts embeddable in Markdown
- **ASCII art**: Use for terminal output, READMEs, or plain-text contexts
- **CSV + narrative**: When the user just needs the data organized with commentary

## References

- `references/best-practices.md` --- Detailed best practices for data visualization

## Examples

**User:** "Create a bar chart comparing these quarterly revenues"
**Agent:** Generates a horizontal bar chart SVG with labeled axes, consistent color,
data labels on each bar, a clear title, and source note. Uses a single brand color
with opacity variation for visual hierarchy.

**User:** "What chart type should I use for this data?"
**Agent:** Asks about the relationship being shown (comparison, trend, distribution),
recommends the appropriate chart type with rationale, then generates it.
