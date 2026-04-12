# Infographic Best Practices

## Data Visualization Principles

### Edward Tufte's Core Rules
1. **Show the data**: Maximize the data-ink ratio (data-ink / total ink)
2. **Avoid chartjunk**: Decorative elements that don't convey information
3. **Use small multiples**: Repeat a chart design across subsets for comparison
4. **Integrate text and graphics**: Labels belong on the chart, not in a distant legend

### Cleveland & McGill Encoding Effectiveness
Perceptual accuracy ranking (most to least accurate for quantitative data):
1. Position along a common scale
2. Position along non-aligned scales
3. Length
4. Angle / Slope
5. Area
6. Volume / Curvature
7. Color saturation / Shading
8. Color hue

Always choose the highest-ranked encoding that fits the data relationship.

### Color Best Practices

**Sequential palettes** (low to high):
- Single hue, varying lightness: `#f7fbff` to `#08306b` (blues)
- Good for: heatmaps, choropleth maps, continuous data

**Diverging palettes** (negative to positive, with neutral center):
- Two hues with light center: red-white-blue, brown-white-teal
- Good for: deviation from a baseline, positive/negative values

**Categorical palettes** (distinct groups):
- Maximally distinct hues at similar lightness
- Limit to 5-7 colors; beyond that, use labels or patterns
- Safe set: `#4e79a7` `#f28e2b` `#e15759` `#76b7b2` `#59a14f` `#edc948` `#b07aa1`

**Colorblind safety** (affects ~8% of men):
- Avoid red-green distinctions without a secondary cue
- Test with deuteranopia and protanopia simulators
- Safe pairs: blue/orange, blue/red, purple/green

### Typography in Visualizations

| Element | Size (relative) | Weight | Case |
|---|---|---|---|
| Title | 1.5x base | Bold | Sentence |
| Subtitle | 1.2x base | Regular | Sentence |
| Axis labels | 1x base | Medium | Sentence |
| Tick labels | 0.85x base | Regular | As-is |
| Annotations | 0.85x base | Regular | Sentence |
| Source note | 0.7x base | Regular | Sentence |

### SVG Layout Template

```xml
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 800 500"
     role="img" aria-label="Chart description">
  <title>Chart Title</title>
  <desc>Detailed description for screen readers</desc>

  <style>
    .title { font: bold 24px sans-serif; fill: #1a1a1a; }
    .subtitle { font: 16px sans-serif; fill: #666; }
    .axis-label { font: 14px sans-serif; fill: #333; }
    .tick-label { font: 12px sans-serif; fill: #666; }
    .grid-line { stroke: #e0e0e0; stroke-width: 1; }
    .data-label { font: 12px sans-serif; fill: #333; }
    .source { font: 10px sans-serif; fill: #999; }
  </style>

  <!-- Margin convention: top=50, right=30, bottom=60, left=80 -->
  <g transform="translate(80, 50)">
    <!-- Chart content here -->
    <!-- Plot area: 690 x 390 -->
  </g>

  <text x="80" y="30" class="title">Chart Title</text>
  <text x="80" y="48" class="subtitle">Supporting context</text>
  <text x="80" y="490" class="source">Source: Data source</text>
</svg>
```

### Margin Convention

Use consistent margins (in pixels for an 800x500 canvas):
- **Top**: 50 (title + subtitle)
- **Right**: 30 (breathing room)
- **Bottom**: 60 (x-axis label + tick labels)
- **Left**: 80 (y-axis label + tick labels)

### Chart-Specific Guidelines

**Bar Charts:**
- Gap between bars = 40-50% of bar width
- Sort by value (descending) unless order is meaningful
- Horizontal bars for long category labels
- Always start y-axis at zero

**Line Charts:**
- Maximum 5-7 lines before it gets cluttered
- Label lines directly (not with legend) when possible
- Use dots at data points if fewer than 15 points
- Consider area fills for 1-2 series

**Scatter Plots:**
- Use opacity (0.3-0.7) when points overlap
- Size encoding should scale by area, not radius
- Add trend line only when correlation is meaningful
- Label outliers directly

**Tables (when better than charts):**
- Use when precise values matter more than patterns
- Right-align numbers, left-align text
- Use subtle row striping for readability
- Highlight key values with bold or color

### Responsive Considerations

- Use `viewBox` without fixed width/height for fluid scaling
- Test readability at 50% and 200% zoom
- Ensure text remains readable at small sizes (minimum 10px equivalent)
- Consider mobile: stack small multiples vertically, use horizontal bars
