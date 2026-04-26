---
name: data-science
description: |
  Data analysis workflow from import through modeling and communication. Use when analyzing a dataset, exploring data, building a statistical model, selecting features, or communicating findings to stakeholders.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-data-science
    version: "1.1.0"
    created: 2026-04-06T00:00:00Z
    category: data-ai
---

# Data Science

## Intro

A solid data analysis flows from import through cleaning, tidying,
exploration, modeling, and communication — in that order. Tidy data,
honest uncertainty, and plain-language findings beat clever models on
messy inputs.

## Overview

### Import and cleaning

Inspect shape, dtypes, nulls, and duplicates immediately after
loading. Print `df.info()` and `df.describe()` before any analysis.
Handle missing data explicitly — drop, fill, or flag, but never ignore
silently. Parse dates at load time (`parse_dates=`) and set correct
dtypes early. Validate assumptions: no negatives where impossible, no
string/numeric mismatches.

### Tidy data

Reshape to tidy form: one variable per column, one observation per
row. Use `melt()` for wide-to-long, `pivot_table()` for long-to-wide.
Split mixed columns (e.g. `city_state` into two). Normalize repeated
entities into separate tables. See Level 3 for messy-pattern
recipes.

### Exploratory data analysis

Start univariate (histograms, value counts, box plots), then
bivariate (scatter, grouped bars, correlation). Look for outliers,
skewness, class imbalance, and unexpected patterns. Use `.groupby()`
to understand subgroups. Summarize findings in plain language before
moving to modeling.

### Statistical reasoning

State the question before choosing a test. Check assumptions
(normality, equal variance, independence) before applying parametric
tests. Report effect sizes alongside p-values — statistical
significance is not practical significance. Use confidence intervals
to communicate uncertainty.

### Feature selection

Remove zero- and near-zero-variance features. Drop one of each highly
correlated pair (r > 0.9). Use domain knowledge first, then
data-driven methods (mutual information, permutation importance).
Watch for leakage — never use target-derived features in training.

### Model selection workflow

Start simple: a mean/median baseline, then linear or logistic
regression. Add complexity only if simpler models underperform and
you understand why. Use train/validation/test split or
cross-validation; never evaluate on training data. Compare on a
single metric chosen before fitting. Document why a model was chosen,
not just which one won.

### Communicating results

Lead with the finding, not the method. Use plain language: "customers
who used feature X retained 23% more" beats "the coefficient was
0.23." Show uncertainty — ranges, intervals, caveats. Include a "so
what" — what action does this analysis support? Pin library
versions, set random seeds, and script everything for
reproducibility.

### Preferred libraries

| Task | First choice | Alternative |
|------|-------------|-------------|
| DataFrames | polars | pandas |
| Visualization | matplotlib + seaborn | plotly |
| Statistics | scipy.stats | statsmodels |
| ML | scikit-learn | xgboost/lightgbm |
| Notebooks | jupyter lab | marimo |

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Starting with a complex model before a simple baseline.** Training an XGBoost model before computing a mean predictor or running a logistic regression means you can't know how much of the model's value came from complexity vs. data. Build a baseline first; complexity is only justified when the baseline is inadequate.
- **Evaluating on training data.** "The model is 98% accurate" on the data it trained on is not a finding — it is a description of memorization. Always evaluate on held-out data and report the held-out metric.
- **Confusing statistical significance with practical significance.** With n > 10,000, almost any real difference is statistically significant. A p-value of 0.001 on a 0.1% conversion difference means "this is real" but does not mean "this matters." Always report effect size alongside p-values.
- **Choosing a statistical test before checking its assumptions.** Running a t-test on non-normally distributed data with small samples produces unreliable p-values. Check normality (Shapiro-Wilk for n < 5000), equal variance (Levene), and independence before choosing a parametric test; fall back to non-parametric alternatives when assumptions fail.
- **Reporting findings without uncertainty.** "Feature X predicts churn" without a confidence interval or cross-validation variance means the finding might not replicate. Always communicate uncertainty ranges alongside point estimates.
- **Truncating the bar chart y-axis to make differences look larger.** A bar chart with a y-axis starting at 0.85 makes a 0.03 difference look dramatic. Unless the baseline is well-understood (like conversion rate from 30% to 32%), bar charts should start at zero.
- **Not setting random seeds before any analysis.** Non-reproducible analysis is not science. Set `random_state=42` (or equivalent) on all sampling, model initialization, and cross-validation calls. Document the seed in the notebook or script so results can be reproduced exactly.

## Full reference

### Tidy data — messy patterns and fixes

**Headers as values (years across columns).** Melt to long form:

```python
# pandas
df_tidy = df.melt(id_vars="country", var_name="year", value_name="population")

# polars
df_tidy = df.unpivot(index="country", variable_name="year", value_name="population")
```

**Multiple variables in one column** (`male_18-25`). Split:

```python
# pandas
df[["gender", "age_group"]] = df["demographic"].str.split("_", expand=True)
```

**Variables in both rows and columns** (metric in rows, dates in
columns). Melt dates first, then pivot metrics:

```python
long = df.melt(id_vars=["id", "metric"], var_name="month", value_name="value")
tidy = long.pivot_table(index=["id", "month"], columns="metric", values="value").reset_index()
```

**Multiple observational units in one table.** Normalize:

```python
songs = df[["song_id", "title", "artist"]].drop_duplicates()
plays = df[["song_id", "play_date", "user_id"]]
```

Wide form is fine when feeding correlation matrices, display pivot
tables, or ML feature matrices. Store and clean tidy; reshape for
the consumer.

| Operation | pandas | polars |
|-----------|--------|--------|
| Wide to long | `df.melt()` | `df.unpivot()` |
| Long to wide | `df.pivot_table()` | `df.pivot()` |
| Split column | `str.split(expand=True)` | `str.split().list.get()` |
| Deduplicate | `df.drop_duplicates()` | `df.unique()` |

### Statistical methods

Always compute descriptive stats first (`df.describe()`,
`value_counts(normalize=True)`, `df.skew()`). Mean is sensitive to
outliers; median is robust. Standard deviation reports spread in
original units; IQR is the robust analogue.

| Distribution | Use when | Example |
|---|---|---|
| Normal | Continuous, symmetric | Heights, measurement error |
| Binomial | Fixed trials, two outcomes | Conversion rate |
| Poisson | Counting events in fixed window | Tickets per hour |
| Exponential | Time between events | Customer arrivals |
| Log-normal | Positive, right-skewed | Income, file sizes |

**Choosing a test:**

| Question | Data | Test | scipy call |
|---|---|---|---|
| One mean vs known | Continuous | One-sample t | `ttest_1samp(x, mu)` |
| Two means | Continuous, normal | Independent t | `ttest_ind(a, b)` |
| Two means, non-normal | Continuous | Mann-Whitney U | `mannwhitneyu(a, b)` |
| Paired before/after | Continuous | Paired t | `ttest_rel(b, a)` |
| 3+ means | Continuous | One-way ANOVA | `f_oneway(a, b, c)` |
| 3+ groups, non-normal | Continuous | Kruskal-Wallis | `kruskal(a, b, c)` |
| Two categoricals | Counts | Chi-square | `chi2_contingency(t)` |

Check normality with `shapiro` (n < 5000), equal variance with
`levene`, and chi-square cell counts >= 5 (otherwise Fisher's exact).

A p-value is the probability of data this extreme if H0 is true — not
the probability that H0 is true. A 95% CI means 95% of intervals
from repeated studies contain the true value. p < 0.05 with a tiny
effect is significant but irrelevant; p = 0.06 is not "trending."
For multiple comparisons use Bonferroni or FDR correction.

For regression, use `statsmodels` OLS for continuous outcomes and
check residual normality, homoscedasticity, and VIF < 5 for
multicollinearity. Use logistic regression for binary outcomes;
exponentiate coefficients for odds ratios.

Practical guidance: small samples (n < 30) call for non-parametric
tests; large samples (n > 10000) make almost everything significant,
so focus on effect size. Understand missingness mechanism before
imputing. Bayesian methods (PyMC, ArviZ) when you actually need
P(hypothesis | data).

### Visualization guidelines

| Relationship | Chart | When to use |
|---|---|---|
| Compare categories | Bar | < 15 categories |
| Many categories | Horizontal bar | Long labels or > 10 |
| Trend over time | Line | 1–5 series |
| Distribution | Histogram | Shape, skew, outliers |
| Group distributions | Box / violin | Compare medians + spread |
| Two continuous | Scatter | Correlation, clusters |
| Part of whole | Stacked bar | Composition over time |
| Correlation matrix | Heatmap | Many pairwise relations |
| Geographic | Choropleth / point | Location matters |

Default rule: bar (categorical) or scatter (continuous) when unsure.

Basic seaborn setup uses `sns.set_theme(style="whitegrid",
palette="colorblind")`. Categorical palettes: `"colorblind"` or
`px.colors.qualitative.Safe`. Sequential: `viridis`, `cividis`.
Diverging: `coolwarm`, `RdBu`. Avoid rainbow/jet and red-green-only
distinctions. Never rely on color alone — add patterns, markers, or
labels. Test with a colorblindness simulator.

Annotate the insight, not just the data: peak labels, threshold
lines, events that explain trend changes. Use direct labels instead
of legends with 2–3 series. Plotly for interactive (notebooks,
dashboards); matplotlib/seaborn for static (reports, papers).

Export checklist: title states the finding, both axes labeled with
units, legend or direct labels, colorblind-friendly palette, 300 dpi
for print and SVG/PDF for scalable, fonts readable at final size.

Common mistakes: truncated y-axis on bar charts, dual y-axes, 3D
charts, pies with > 4 slices, missing units, default titles like
"Figure 1," and overplotting (use alpha, hexbin, or 2D density).

### Worked examples

- **CSV of transactions, churn patterns:** Load, inspect
  shape/dtypes/nulls, build a tidy customer time series, run EDA on
  churn-rate distributions and cohorts, t-test usage frequency
  between churned and retained groups (with effect size), produce
  annotated visualizations of key drivers.
- **Top features for house prices:** Load, clean nulls, encode
  categoricals, check correlations, fit a baseline linear regression,
  use permutation importance to rank features, report top 10 with
  CIs and a bar chart, note multicollinearity concerns.
- **A/B conversion difference significant?** Compute rates and
  sample sizes, run chi-square (verify expected cell counts >= 5),
  report p-value and 95% CI for the difference, visualize rates with
  error bars, state whether the effect is practically meaningful.
