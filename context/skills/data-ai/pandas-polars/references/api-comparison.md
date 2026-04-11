# Pandas vs Polars API Comparison

## Reading Data

```python
# Pandas
df = pd.read_csv("data.csv", usecols=["a", "b"], dtype={"a": "int64"})
df = pd.read_parquet("data.parquet", columns=["a", "b"])

# Polars
df = pl.read_csv("data.csv", columns=["a", "b"], schema={"a": pl.Int64})
df = pl.read_parquet("data.parquet", columns=["a", "b"])

# Polars lazy (recommended for large files)
lf = pl.scan_csv("data.csv")
lf = pl.scan_parquet("data.parquet")
df = lf.filter(...).select(...).collect()
```

## Selecting Columns

```python
# Pandas
df[["name", "age"]]
df.loc[:, "name":"age"]          # slice by label

# Polars
df.select("name", "age")
df.select(pl.col("name"), pl.col("age"))
df.select(pl.col("^sales_.*$"))  # regex column selection
```

## Filtering Rows

```python
# Pandas
df[df["age"] > 30]
df[(df["age"] > 30) & (df["city"] == "Berlin")]
df.query("age > 30 and city == 'Berlin'")

# Polars
df.filter(pl.col("age") > 30)
df.filter((pl.col("age") > 30) & (pl.col("city") == "Berlin"))
```

## Adding / Transforming Columns

```python
# Pandas
df["total"] = df["price"] * df["qty"]
df = df.assign(total=lambda d: d["price"] * d["qty"])

# Polars
df = df.with_columns((pl.col("price") * pl.col("qty")).alias("total"))
df = df.with_columns(
    pl.col("name").str.to_uppercase().alias("name_upper"),
    (pl.col("price") * 1.1).alias("price_with_tax"),
)
```

## GroupBy and Aggregation

```python
# Pandas
df.groupby("category").agg(
    avg_price=("price", "mean"),
    total_qty=("qty", "sum"),
    count=("id", "count"),
).reset_index()

# Polars
df.group_by("category").agg(
    pl.col("price").mean().alias("avg_price"),
    pl.col("qty").sum().alias("total_qty"),
    pl.col("id").count().alias("count"),
)
```

## Window Functions

```python
# Pandas
df["rank"] = df.groupby("category")["price"].rank(ascending=False)
df["avg_in_group"] = df.groupby("category")["price"].transform("mean")

# Polars
df = df.with_columns(
    pl.col("price").rank(descending=True).over("category").alias("rank"),
    pl.col("price").mean().over("category").alias("avg_in_group"),
)
```

## Joins

```python
# Pandas
pd.merge(left, right, on="id", how="left")
pd.merge(left, right, left_on="user_id", right_on="id", how="inner")

# Polars
left.join(right, on="id", how="left")
left.join(right, left_on="user_id", right_on="id", how="inner")

# Anti-join (rows in left NOT in right)
# Pandas
left[~left["id"].isin(right["id"])]

# Polars
left.join(right, on="id", how="anti")
```

## Sorting

```python
# Pandas
df.sort_values(["price", "name"], ascending=[False, True])

# Polars
df.sort("price", descending=True).sort("name")
df.sort(["price", "name"], descending=[True, False])
```

## Pivot and Melt

```python
# Pivot — Pandas
df.pivot_table(index="date", columns="product", values="sales", aggfunc="sum")

# Pivot — Polars
df.pivot(on="product", index="date", values="sales", aggregate_function="sum")

# Melt — Pandas
df.melt(id_vars=["date"], value_vars=["product_a", "product_b"],
        var_name="product", value_name="sales")

# Melt — Polars
df.unpivot(index="date", on=["product_a", "product_b"],
           variable_name="product", value_name="sales")
```

## Missing Data

```python
# Pandas
df.isna().sum()
df.fillna({"price": 0, "name": "Unknown"})
df.dropna(subset=["critical_col"])
df["value"].ffill()

# Polars
df.null_count()
df.with_columns(
    pl.col("price").fill_null(0),
    pl.col("name").fill_null("Unknown"),
)
df.drop_nulls(subset=["critical_col"])
df.with_columns(pl.col("value").forward_fill())
```

## When to Use Which

| Scenario | Recommendation |
|---|---|
| Exploratory data analysis | Pandas (Jupyter integration, wider ecosystem) |
| Dataset > 1 GB | Polars (lower memory, parallel execution) |
| Production data pipeline | Polars (lazy eval, deterministic performance) |
| Integration with sklearn/statsmodels | Pandas (native support) |
| Streaming / chunked processing | Polars (lazy scan) or Pandas (chunked read) |
| Simple one-off scripts | Either — use what the team knows |
| Multi-threaded workloads | Polars (releases GIL, built-in parallelism) |
