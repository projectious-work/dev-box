# Chart Selection Guide

Decision tree for choosing the right chart type based on your data relationship.

## Decision Tree

Ask: **What relationship am I showing?**

### Comparison (values across categories)

**Bar Chart (vertical)**
- When to use: Comparing magnitudes across < 15 categories
- Best for: Revenue by region, counts by category, survey responses
- When NOT to use: More than 15 categories (use horizontal bar), time series (use line)

**Horizontal Bar Chart**
- When to use: Category labels are long, or > 10 categories
- Best for: Feature importance rankings, country comparisons, sorted lists
- When NOT to use: Time-based data, continuous x-axis

**Grouped Bar Chart**
- When to use: Comparing 2-3 sub-groups within each category
- Best for: Revenue by region split by product, quarterly results by department
- When NOT to use: More than 3 sub-groups (use faceted charts), showing totals (use stacked)

### Trend (change over time)

**Line Chart**
- When to use: Showing change over a continuous time axis, 1-5 series
- Best for: Revenue over months, temperature over time, stock prices
- When NOT to use: Fewer than 5 time points (use bar), more than 7 lines (facet instead)

**Area Chart**
- When to use: Showing volume or magnitude over time, emphasizing the total
- Best for: Traffic volume, cumulative metrics, stacked composition over time
- When NOT to use: Comparing series with similar magnitudes (lines occlude), precise reading needed

### Distribution (how values are spread)

**Histogram**
- When to use: Showing the shape of a single continuous variable
- Best for: Age distributions, response time distributions, salary ranges
- When NOT to use: Categorical data (use bar), comparing multiple groups (use overlaid or violin)

**Box Plot**
- When to use: Comparing distributions across groups, showing median and spread
- Best for: Salary by department, test scores by class, latency by endpoint
- When NOT to use: When the audience is unfamiliar with box plots (use histogram facets)

**Violin Plot**
- When to use: Like box plot but shows the full distribution shape
- Best for: Bimodal distributions, detailed group comparison
- When NOT to use: Simple comparisons where box plot suffices, presentations to non-technical audiences

### Relationship (correlation between variables)

**Scatter Plot**
- When to use: Showing relationship between two continuous variables
- Best for: Height vs weight, spend vs revenue, feature correlations
- When NOT to use: More than ~1000 points without transparency (use hexbin/density)

**Bubble Chart**
- When to use: Scatter plot with a third variable mapped to bubble size
- Best for: Country GDP vs life expectancy sized by population
- When NOT to use: Precise comparisons (humans are bad at comparing areas)

**Heatmap**
- When to use: Showing magnitude across two categorical or ordinal dimensions
- Best for: Correlation matrices, pivot tables, time-of-day patterns
- When NOT to use: Sparse data (most cells empty), more than ~20 items per axis

### Composition (parts of a whole)

**Stacked Bar Chart**
- When to use: Showing how parts contribute to a total across categories or time
- Best for: Revenue breakdown by source over quarters, budget allocation
- When NOT to use: More than 5-6 segments (hard to compare middle segments)

**Pie Chart**
- When to use: Only with 2-4 slices where approximate proportion is sufficient
- Best for: Binary splits (yes/no), market share with a dominant leader
- When NOT to use: More than 4 slices, comparing across groups, precise values needed. Almost always use horizontal bar instead.

**Treemap**
- When to use: Hierarchical part-of-whole with many categories
- Best for: Disk usage, portfolio allocation, organizational breakdown
- When NOT to use: When precise comparison matters (use bar), non-hierarchical data

### Geographic (location matters)

**Choropleth Map**
- When to use: Showing values for defined geographic regions
- Best for: Election results by state, population density by country
- When NOT to use: Point-level data (use dot map), data that does not correlate with area size

**Point/Dot Map**
- When to use: Showing individual locations or events
- Best for: Store locations, earthquake epicenters, crime incidents
- When NOT to use: Regional aggregates (use choropleth), dense urban data without clustering

## Quick Reference Table

| Relationship | First choice | Alternative | Avoid |
|-------------|-------------|-------------|-------|
| Compare categories | Vertical bar | Horizontal bar, lollipop | Pie chart |
| Trend over time | Line chart | Area chart | Bar (unless few points) |
| Distribution | Histogram | Box plot, violin | Pie chart |
| Two variables | Scatter plot | Hexbin, 2D density | Line (unless time) |
| Parts of whole | Stacked bar | Treemap | Pie (> 4 slices) |
| Correlation matrix | Heatmap | Clustered pairs | Table of numbers |
| Geographic | Choropleth | Point map | Non-geographic chart |

## Universal Rules

1. Start with the simplest chart that answers the question
2. If you need a legend with more than 7 entries, redesign the chart
3. If a colleague cannot understand the chart in 10 seconds, simplify it
4. When comparing, use the same axis scale across panels
5. Titles should state the insight: "Sales doubled in Q3" not "Sales by Quarter"
