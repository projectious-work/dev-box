# Visualization Guidelines

Practical chart selection, styling, and accessibility reference.

## Chart Selection Guide

| Relationship | Chart type | When to use |
|-------------|-----------|-------------|
| Comparison across categories | Bar chart (vertical) | < 15 categories, comparing magnitudes |
| Comparison, many categories | Horizontal bar | Labels are long or > 10 categories |
| Trend over time | Line chart | Continuous time axis, 1-5 series |
| Distribution, one variable | Histogram | Show shape, skew, outliers |
| Distribution, compare groups | Box plot / violin | Compare medians and spread across groups |
| Two continuous variables | Scatter plot | Look for correlation, clusters, outliers |
| Three variables | Scatter + color/size | Third variable mapped to hue or marker size |
| Part of whole | Stacked bar | Composition over time or across categories |
| Part of whole, single | Pie chart (rarely) | Only with 2-4 slices; bar is usually better |
| Correlation matrix | Heatmap | Many pairwise relationships at once |
| Geographic | Choropleth / point map | Location is a key variable |

**Default rule:** When unsure, start with a bar chart (categorical) or scatter plot (continuous).

## Matplotlib + Seaborn Patterns

### Basic Setup

```python
import matplotlib.pyplot as plt
import seaborn as sns

sns.set_theme(style="whitegrid", palette="colorblind")
fig, ax = plt.subplots(figsize=(8, 5))
```

### Common Charts

```python
# Distribution
sns.histplot(df["value"], bins=30, kde=True, ax=ax)

# Comparison
sns.barplot(data=df, x="category", y="value", errorbar="ci", ax=ax)

# Relationship
sns.scatterplot(data=df, x="x", y="y", hue="group", ax=ax)

# Trend
ax.plot(df["date"], df["value"], marker="o", markersize=4)

# Correlation heatmap
sns.heatmap(df.corr(), annot=True, fmt=".2f", cmap="coolwarm", center=0, ax=ax)

# Box plot comparison
sns.boxplot(data=df, x="group", y="value", ax=ax)
```

### Finishing Touches

```python
ax.set_title("Clear Descriptive Title", fontsize=14, fontweight="bold")
ax.set_xlabel("X Label (units)")
ax.set_ylabel("Y Label (units)")
fig.tight_layout()
fig.savefig("plot.png", dpi=300, bbox_inches="tight")
```

## Plotly for Interactive Charts

```python
import plotly.express as px

# Scatter with hover details
fig = px.scatter(df, x="x", y="y", color="group", hover_data=["name"],
                 title="Clear Title", labels={"x": "X (units)", "y": "Y (units)"})
fig.update_layout(template="plotly_white")
fig.show()

# Time series
fig = px.line(df, x="date", y="value", color="series")
```

Use plotly when the audience will interact with the chart (notebooks, dashboards).
Use matplotlib/seaborn when exporting to static reports or papers.

## Color and Accessibility

### Palette Selection

- **Categorical:** Use `"colorblind"` (seaborn) or `px.colors.qualitative.Safe` (plotly)
- **Sequential:** `"viridis"`, `"cividis"` — perceptually uniform, colorblind safe
- **Diverging:** `"coolwarm"`, `"RdBu"` — centered data (correlation, deviation from mean)
- **Avoid:** Rainbow/jet (not perceptually uniform), red-green only distinctions

### Accessibility Rules

- Do not rely on color alone — add patterns, markers, or direct labels
- Ensure sufficient contrast (WCAG AA: 4.5:1 for text)
- Use at least 2pt line width; 6pt+ marker size
- Add alt text when embedding charts in documents or HTML
- Test with a colorblindness simulator (Coblis, Color Oracle)

## Annotation Practices

Annotations turn a chart from "here is data" into "here is insight":

```python
# Highlight a specific point
ax.annotate("Peak: 1,247", xy=(peak_x, peak_y), fontsize=10,
            arrowprops=dict(arrowstyle="->", color="gray"),
            xytext=(peak_x + 5, peak_y + 100))

# Add a reference line
ax.axhline(y=threshold, color="red", linestyle="--", label="Target")

# Direct label instead of legend
ax.text(x[-1], y_a[-1], " Group A", va="center", fontsize=10)
```

When to annotate:
- Max/min values that matter to the story
- Threshold or target lines
- Events that explain trend changes (product launch, policy change)
- Direct labels when there are only 2-3 series (replace the legend)

## Common Mistakes

1. **Truncated y-axis** — Bar charts should start at zero; line charts may not need to
2. **Too many categories** — More than 7 colors is unreadable; aggregate or facet instead
3. **Dual y-axes** — Misleading; use two separate panels instead
4. **3D charts** — Almost always worse than 2D; avoid unless showing actual 3D data
5. **Pie charts with many slices** — Use horizontal bar; humans compare lengths better than angles
6. **Missing units** — Every axis needs a label with units
7. **Overplotting** — Too many scatter points; use alpha, hexbin, or 2D density
8. **Default titles** — "Figure 1" says nothing; state the finding: "Revenue grew 40% in Q3"

## Export Checklist

Before sharing any visualization:
- [ ] Title states the finding, not just the topic
- [ ] Both axes labeled with units
- [ ] Legend is present (or direct labels used)
- [ ] Colorblind-friendly palette
- [ ] Saved at 300 dpi for print, SVG/PDF for scalable
- [ ] Fonts readable at final display size (12pt+ for presentations)
