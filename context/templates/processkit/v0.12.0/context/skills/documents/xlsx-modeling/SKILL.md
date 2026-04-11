---
name: xlsx-modeling
description: |
  Creating Excel spreadsheets programmatically with openpyxl — worksheets, data, formulas, charts, formatting, and named ranges. Use when generating financial models, data reports, dashboards, or any structured spreadsheet from data or templates.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-xlsx-modeling
    version: "1.0.0"
    created: 2026-04-08T00:00:00Z
    category: documents
    layer: 2
---

# XLSX Modeling

## Intro

Use `openpyxl` to generate Excel files programmatically. Separate
inputs, calculations, and outputs into distinct sheets or named
ranges — this makes models auditable and reduces errors. Write
cell values as Python data and let Excel formulas do the math;
never pre-calculate values that Excel should own.

## Overview

### Setup

```python
# /// script
# dependencies = ["openpyxl"]
# ///
from openpyxl import Workbook, load_workbook
from openpyxl.styles import Font, PatternFill, Alignment, Border, Side
from openpyxl.utils import get_column_letter, column_index_from_string

wb = Workbook()
ws = wb.active
ws.title = "Summary"
```

Install with `uv add openpyxl` or `pip install openpyxl`.

### Writing data

```python
# Single cell
ws["A1"] = "Revenue"
ws["B1"] = 1_250_000

# Row at a time
ws.append(["Q1", 312500, "=B2/B$1"])    # mix of values and formulas
ws.append(["Q2", 340000, "=B3/B$1"])

# From a list-of-lists
data = [["Name", "Score"], ["Alice", 92], ["Bob", 87]]
for row in data:
    ws.append(row)
```

Rows are 1-indexed. `ws["A1"]` and `ws.cell(row=1, column=1)` are
equivalent. Use `get_column_letter(n)` to convert column numbers to
letters.

### Formulas

Write Excel formula strings directly. openpyxl writes the formula;
Excel evaluates it on open.

```python
ws["C2"] = "=SUM(B2:B5)"
ws["D2"] = "=AVERAGE(B2:B5)"
ws["E2"] = "=IF(B2>1000, \"High\", \"Low\")"
ws["F2"] = "=VLOOKUP(A2, Lookup!A:B, 2, FALSE)"
```

Set `data_only=True` when reading a file to get cached values
instead of formula strings: `load_workbook("file.xlsx", data_only=True)`.

### Named ranges

```python
from openpyxl.workbook.defined_name import DefinedName
from openpyxl.utils import quote_sheetname, absolute_coordinate

# Define a named range
ref = f"{quote_sheetname('Summary')}!{absolute_coordinate('B2:B13')}"
wb.defined_names["MonthlyRevenue"] = DefinedName("MonthlyRevenue", attr_text=ref)

# Reference it in a formula
ws["C2"] = "=SUM(MonthlyRevenue)"
```

Use named ranges for values referenced across multiple sheets — it
makes formulas readable and survives row/column insertions.

### Formatting

```python
# Font
ws["A1"].font = Font(name="Calibri", bold=True, size=12, color="1F497D")

# Fill
ws["A1"].fill = PatternFill(fill_type="solid", fgColor="4472C4")

# Alignment
ws["A1"].alignment = Alignment(horizontal="center", vertical="center",
                                wrap_text=True)

# Border
thin = Side(style="thin")
ws["A1"].border = Border(left=thin, right=thin, top=thin, bottom=thin)

# Number format
ws["B2"].number_format = "#,##0.00"          # 1,234.56
ws["C2"].number_format = "0.0%"              # 12.3%
ws["D2"].number_format = '"$"#,##0'          # $1,234
ws["E2"].number_format = "YYYY-MM-DD"        # 2026-04-08
```

Apply styles to header rows and totals rows consistently. Use a
defined `NamedStyle` object to apply a style set to multiple cells:

```python
from openpyxl.styles import NamedStyle
header_style = NamedStyle(name="header")
header_style.font = Font(bold=True, color="FFFFFF")
header_style.fill = PatternFill(fill_type="solid", fgColor="2E74B5")
wb.add_named_style(header_style)
ws["A1"].style = "header"
```

### Column widths and row heights

```python
ws.column_dimensions["A"].width = 20
ws.column_dimensions["B"].width = 14
ws.row_dimensions[1].height = 30   # points

# Auto-width approximation (openpyxl has no auto-fit)
for col in ws.columns:
    max_length = max(len(str(cell.value or "")) for cell in col)
    ws.column_dimensions[col[0].column_letter].width = min(max_length + 4, 60)
```

### Charts

```python
from openpyxl.chart import BarChart, Reference

chart = BarChart()
chart.type = "col"
chart.title = "Quarterly Revenue"
chart.y_axis.title = "Revenue ($)"
chart.x_axis.title = "Quarter"

data_ref = Reference(ws, min_col=2, min_row=1, max_row=5)
categories = Reference(ws, min_col=1, min_row=2, max_row=5)
chart.add_data(data_ref, titles_from_data=True)
chart.set_categories(categories)
chart.shape = 4         # rectangle (no 3D)

ws.add_chart(chart, "D2")
```

Chart types: `BarChart`, `LineChart`, `PieChart`, `AreaChart`,
`ScatterChart`, `BubbleChart`. Set `chart.shape = 4` (flat) for all
chart types — 3D charts are harder to read.

### Freeze panes and filters

```python
ws.freeze_panes = "B2"       # freeze row 1 and column A
ws.auto_filter.ref = "A1:D100"   # enable autofilter on header row
```

### Multiple sheets

```python
ws_data  = wb.create_sheet("Raw Data")
ws_calc  = wb.create_sheet("Calculations")
ws_out   = wb.create_sheet("Output")

# Reorder: move Output to first position
wb.move_sheet("Output", offset=-wb.index(wb["Output"]))

# Remove the default sheet if unused
del wb["Sheet"]
```

### Saving

```python
wb.save("report.xlsx")

from io import BytesIO
buf = BytesIO()
wb.save(buf)
buf.seek(0)
```

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Pre-calculating values that Excel formulas should own.** Writing `ws["C2"] = sum(data)` instead of `ws["C2"] = "=SUM(B2:B100)"` makes the spreadsheet static — users cannot change inputs and see results update. Write formulas for all derived values; let Excel own the calculation.
- **Not separating inputs, calculations, and outputs into distinct sheets.** A single sheet that mixes raw data, intermediate calculations, and final results becomes impossible to audit. Organize into at minimum an "Inputs" sheet and an "Output" or "Summary" sheet; keep raw data on its own sheet untouched.
- **Using hard-coded cell references in formulas instead of named ranges.** A formula like `=SUM(Sheet1!B2:B13)` breaks silently if a row is inserted before row 2. Named ranges (e.g. `MonthlyRevenue`) survive structural changes and make formulas self-documenting.
- **Omitting number formats on numeric cells.** Raw numbers without formatting (`1234567.89`) are harder to read than formatted ones (`$1,234,567.89`). Set `.number_format` on all monetary, percentage, and date cells before saving.
- **Reading a file with `data_only=False` expecting cached values.** When `data_only=False` (the default), openpyxl returns formula strings, not computed values. To read the last computed result, open with `load_workbook("file.xlsx", data_only=True)`. If the file has never been opened in Excel, the cached values will be `None`.
- **Adding 3D chart types.** 3D bar and pie charts are visually misleading and harder to read than flat equivalents. Set `chart.shape = 4` (rectangular) and never use 3D variants in generated output.
- **Not freezing header rows on data sheets.** A data sheet without `ws.freeze_panes = "A2"` forces users to scroll to remember column labels. Always freeze at least the header row on any sheet with more than one screen of rows.

## Full reference

### openpyxl vs xlsxwriter

| | openpyxl | xlsxwriter |
|---|---|---|
| Read existing files | Yes | No |
| Write formulas | Yes (as strings) | Yes |
| Charts | Basic | Rich |
| Performance (large files) | Moderate | Fast |
| Streams (constant memory) | `WriteOnlyWorkbook` | `use_memory_strings=True` |

Use `openpyxl` when you need to read and modify an existing file.
Use `xlsxwriter` when producing large files from scratch with rich
chart formatting and you don't need to read back the result.

### Large files (streaming write)

```python
from openpyxl import Workbook

wb = Workbook(write_only=True)
ws = wb.create_sheet()
ws.append(["Name", "Score"])    # header
for name, score in dataset:
    ws.append([name, score])
wb.save("large.xlsx")
```

`write_only=True` uses constant memory — cells cannot be read back
after writing.

### Conditional formatting

```python
from openpyxl.formatting.rule import ColorScaleRule, DataBarRule

rule = ColorScaleRule(
    start_type="min", start_color="F8696B",
    mid_type="percentile", mid_value=50, mid_color="FFEB84",
    end_type="max", end_color="63BE7B",
)
ws.conditional_formatting.add("B2:B100", rule)
```

### Data validation (dropdown lists)

```python
from openpyxl.worksheet.datavalidation import DataValidation

dv = DataValidation(type="list", formula1='"Yes,No,N/A"', allow_blank=True)
ws.add_data_validation(dv)
dv.add("C2:C100")
```

### Password protection

```python
ws.protection.sheet = True
ws.protection.password = "readonly"
ws.protection.enable()
```
