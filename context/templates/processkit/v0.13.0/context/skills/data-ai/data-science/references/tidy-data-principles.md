# Tidy Data Principles

Reference for reshaping messy data into analysis-ready form.
Based on Hadley Wickham's tidy data framework.

## The Three Rules

1. **Each variable forms a column** — one measured quantity per column
2. **Each observation forms a row** — one unit of analysis per row
3. **Each type of observational unit forms a table** — don't mix patients and visits in one table

## Common Messy Patterns and Fixes

### Pattern 1: Column Headers Are Values, Not Variables

Messy — year values spread across columns:
```
country    2020    2021    2022
Canada     38M     38.5M   39M
```

Fix — melt to long form:
```python
# pandas
df_tidy = df.melt(id_vars="country", var_name="year", value_name="population")

# polars
df_tidy = df.unpivot(index="country", variable_name="year", value_name="population")
```

### Pattern 2: Multiple Variables in One Column

Messy — "male_18-25" encodes both gender and age group:
```
demographic    count
male_18-25     1200
female_26-35   1500
```

Fix — split into separate columns:
```python
# pandas
df[["gender", "age_group"]] = df["demographic"].str.split("_", expand=True)

# polars
df = df.with_columns(
    pl.col("demographic").str.split("_").list.get(0).alias("gender"),
    pl.col("demographic").str.split("_").list.get(1).alias("age_group"),
)
```

### Pattern 3: Variables Stored in Both Rows and Columns

Messy — metric type is a row value, dates are columns:
```
id    metric    Jan    Feb    Mar
1     temp      20     22     25
1     precip    5      3      8
```

Fix — melt dates first, then pivot metrics:
```python
# pandas
long = df.melt(id_vars=["id", "metric"], var_name="month", value_name="value")
tidy = long.pivot_table(index=["id", "month"], columns="metric", values="value").reset_index()

# polars
long = df.unpivot(index=["id", "metric"], variable_name="month", value_name="value")
tidy = long.pivot(on="metric", index=["id", "month"], values="value")
```

### Pattern 4: Multiple Observational Units in One Table

Messy — song info repeated for every play event:
```
song_id  title       artist      play_date    user_id
1        Hey Jude    Beatles     2024-01-01   42
1        Hey Jude    Beatles     2024-01-02   77
```

Fix — normalize into two tables:
```python
songs = df[["song_id", "title", "artist"]].drop_duplicates()
plays = df[["song_id", "play_date", "user_id"]]
```

### Pattern 5: One Observation Spread Across Multiple Files

Common with time-partitioned data (one CSV per month).

Fix — concatenate then verify:
```python
import glob
# pandas
frames = [pd.read_csv(f) for f in sorted(glob.glob("data/2024-*.csv"))]
df = pd.concat(frames, ignore_index=True)

# polars
df = pl.concat([pl.read_csv(f) for f in sorted(glob.glob("data/2024-*.csv"))])
```

## Validation Checklist

After reshaping, verify tidy form:

```python
# Check uniqueness of observation key
assert df.duplicated(subset=["id", "date"]).sum() == 0, "Duplicate observations"

# Check one dtype per column (no mixed types)
print(df.dtypes)

# Check no list/dict values hiding inside cells
assert not df.applymap(lambda x: isinstance(x, (list, dict))).any().any()
```

## When Wide Format Is Fine

Tidy (long) form is not always the goal:
- **Correlation matrices** need wide form
- **Pivot tables for display** are meant to be wide
- **ML feature matrices** are one-row-per-sample, features as columns (already tidy)

Reshape for the tool you are feeding, but store and clean in tidy form.

## Quick Reference

| Operation | pandas | polars |
|-----------|--------|--------|
| Wide to long | `df.melt()` | `df.unpivot()` |
| Long to wide | `df.pivot_table()` | `df.pivot()` |
| Split column | `str.split(expand=True)` | `str.split().list.get()` |
| Combine columns | `df["a"] + "_" + df["b"]` | `pl.concat_str(["a","b"], separator="_")` |
| Deduplicate | `df.drop_duplicates()` | `df.unique()` |
| Stack files | `pd.concat(frames)` | `pl.concat(frames)` |
