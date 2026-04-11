# Statistical Methods Reference

Practical guide to choosing and applying statistical methods in data analysis.

## Descriptive Statistics

Always compute before any test or model:

```python
df.describe()                          # central tendency, spread
df["col"].value_counts(normalize=True) # category proportions
df.skew()                              # asymmetry (|skew| > 1 = notable)
df.kurt()                              # tail weight
```

Key measures:
- **Mean** — sensitive to outliers; use for symmetric distributions
- **Median** — robust to outliers; use for skewed data or when reporting "typical"
- **Standard deviation** — spread in original units
- **IQR** — robust spread measure; use with median

## Common Distributions

| Distribution | Use when | Example |
|-------------|----------|---------|
| Normal | Continuous, symmetric, many small effects | Heights, measurement error |
| Binomial | Fixed trials, two outcomes | Conversion rate (n visitors, k convert) |
| Poisson | Counting events in fixed time/space | Support tickets per hour |
| Exponential | Time between events | Time between customer arrivals |
| Log-normal | Positive, right-skewed | Income, file sizes |

Check distribution fit visually (histogram + QQ plot) before assuming.

## Hypothesis Testing

### Decision Framework

1. State H0 (null) and H1 (alternative) before looking at data
2. Choose significance level (alpha = 0.05 is convention, not law)
3. Check test assumptions
4. Compute test statistic and p-value
5. Report effect size and confidence interval alongside the p-value

### Choosing a Test

| Question | Data type | Test | scipy call |
|----------|-----------|------|-----------|
| One group mean vs known value | Continuous | One-sample t-test | `ttest_1samp(x, mu)` |
| Two group means | Continuous, normal | Independent t-test | `ttest_ind(a, b)` |
| Two group means, non-normal | Continuous | Mann-Whitney U | `mannwhitneyu(a, b)` |
| Paired before/after | Continuous | Paired t-test | `ttest_rel(before, after)` |
| 3+ group means | Continuous | One-way ANOVA | `f_oneway(a, b, c)` |
| 3+ groups, non-normal | Continuous | Kruskal-Wallis | `kruskal(a, b, c)` |
| Two categorical variables | Counts | Chi-square | `chi2_contingency(table)` |
| Two proportions | Binary | Z-test for proportions | `proportions_ztest()` (statsmodels) |

### Assumption Checks

```python
from scipy import stats

# Normality (use for n < 5000; for larger samples, visual check is better)
stat, p = stats.shapiro(x)

# Equal variances
stat, p = stats.levene(a, b)

# Chi-square: all expected cell counts >= 5
chi2, p, dof, expected = stats.chi2_contingency(table)
assert expected.min() >= 5, "Use Fisher's exact test instead"
```

## P-values and Confidence Intervals

**P-value:** Probability of seeing data this extreme if H0 is true. It is NOT the
probability that H0 is true.

**Confidence interval:** Range of plausible values for the parameter. A 95% CI means
if we repeated the study many times, 95% of intervals would contain the true value.

Rules of thumb:
- p < 0.05 with tiny effect size = statistically significant but practically irrelevant
- p = 0.06 is not "trending toward significance" — report the number, skip the spin
- Always report CI width: a narrow CI is more informative than a wide one regardless of p
- Multiple comparisons: use Bonferroni (alpha / n_tests) or FDR correction

```python
from scipy.stats import sem, t
import numpy as np

# 95% CI for a mean
mean = np.mean(x)
ci = t.interval(0.95, len(x)-1, loc=mean, scale=sem(x))
```

## Correlation vs Causation

- **Pearson r** — linear relationship, both continuous, roughly normal
- **Spearman rho** — monotonic relationship, ordinal or non-normal
- **Point-biserial** — one binary, one continuous

Correlation does not imply causation. Common traps:
- Confounding variables (ice cream sales correlate with drowning — heat is the confounder)
- Reverse causation (does A cause B or B cause A?)
- Selection bias (only analyzing surviving patients)

## Regression

### Linear Regression

Use when predicting a continuous outcome:

```python
import statsmodels.api as sm

X = sm.add_constant(df[["feature1", "feature2"]])
model = sm.OLS(df["target"], X).fit()
print(model.summary())  # check R-squared, p-values, residual plots
```

Check: residuals normal, homoscedastic, no multicollinearity (VIF < 5).

### Logistic Regression

Use when predicting a binary outcome:

```python
from sklearn.linear_model import LogisticRegression
from sklearn.metrics import classification_report

model = LogisticRegression().fit(X_train, y_train)
print(classification_report(y_test, model.predict(X_test)))
```

Interpret coefficients as log-odds; exponentiate for odds ratios.

## Practical Guidance

- **Small samples (n < 30):** Use non-parametric tests; be skeptical of any result
- **Large samples (n > 10000):** Almost everything is "significant"; focus on effect size
- **Missing data:** Understand the mechanism (MCAR/MAR/MNAR) before imputing
- **Multiple testing:** If you test 20 hypotheses at alpha=0.05, expect 1 false positive
- **Bayesian alternative:** When you need P(hypothesis | data) instead of P(data | hypothesis),
  use Bayesian methods (PyMC, ArviZ)
