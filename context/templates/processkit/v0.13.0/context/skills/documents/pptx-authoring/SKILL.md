---
name: pptx-authoring
description: |
  Creating PowerPoint presentations programmatically with python-pptx — slides, layouts, text boxes, charts, tables, and images. Use when generating slide decks, briefings, reports, or presentations from data or templates.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-pptx-authoring
    version: "1.0.0"
    created: 2026-04-08T00:00:00Z
    category: documents
    layer: 2
---

# PPTX Authoring

## Intro

Use `python-pptx` to generate PowerPoint files programmatically.
Always start from a template PPTX to inherit brand fonts, colors, and
slide layouts. One idea per slide — each slide should have a single,
clear takeaway expressed in the title. Use charts for quantitative
data and images for visual evidence; avoid bullet-heavy text slides.

## Overview

### Setup

```python
# /// script
# dependencies = ["python-pptx"]
# ///
from pptx import Presentation
from pptx.util import Inches, Pt, Emu
from pptx.dml.color import RGBColor
from pptx.enum.text import PP_ALIGN

prs = Presentation()                          # blank (10×7.5 in widescreen)
prs = Presentation("brand_template.pptx")    # or from a template
```

Install with `uv add python-pptx` or `pip install python-pptx`.

### Slide layouts

Every presentation has a slide master with named layouts. List
available layouts before picking one:

```python
for i, layout in enumerate(prs.slide_layouts):
    print(i, layout.name)
# Common: 0=Title Slide, 1=Title and Content, 2=Title Only,
#         5=Blank, 6=Content with Caption
```

Add a slide and populate its placeholders:

```python
layout = prs.slide_layouts[1]   # "Title and Content"
slide = prs.slides.add_slide(layout)

title = slide.shapes.title
title.text = "Slide Title — the Takeaway"

body = slide.placeholders[1]
tf = body.text_frame
tf.text = "First bullet"
tf.add_paragraph().text = "Second bullet"
```

Placeholder indices vary by template. Inspect with:
```python
for ph in slide.placeholders:
    print(ph.placeholder_format.idx, ph.name)
```

### Text boxes and formatting

```python
from pptx.util import Inches, Pt
txBox = slide.shapes.add_textbox(Inches(1), Inches(1.5), Inches(8), Inches(1))
tf = txBox.text_frame
tf.word_wrap = True

p = tf.paragraphs[0]
p.text = "Label"
p.font.size = Pt(18)
p.font.bold = True
p.font.color.rgb = RGBColor(0x1F, 0x49, 0x7D)
p.alignment = PP_ALIGN.LEFT
```

### Charts

```python
from pptx.chart.data import ChartData
from pptx.enum.chart import XL_CHART_TYPE

chart_data = ChartData()
chart_data.categories = ["Q1", "Q2", "Q3", "Q4"]
chart_data.add_series("Revenue", (1.2, 1.5, 1.8, 2.1))

x, y, cx, cy = Inches(1), Inches(2), Inches(8), Inches(4)
chart = slide.shapes.add_chart(
    XL_CHART_TYPE.COLUMN_CLUSTERED, x, y, cx, cy, chart_data
).chart

chart.has_legend = True
chart.has_title = True
chart.chart_title.text_frame.text = "Quarterly Revenue ($M)"
```

Common chart types: `COLUMN_CLUSTERED`, `BAR_CLUSTERED`, `LINE`,
`PIE`, `AREA`, `XY_SCATTER`. Prefer column/bar for comparisons, line
for trends, and scatter for correlations.

### Tables

```python
rows, cols = 3, 3
table = slide.shapes.add_table(rows, cols, Inches(1), Inches(2),
                                Inches(8), Inches(2)).table

table.cell(0, 0).text = "Metric"
table.cell(0, 1).text = "Value"
table.cell(0, 2).text = "Change"

# Column widths
table.columns[0].width = Inches(3)
table.columns[1].width = Inches(2.5)
table.columns[2].width = Inches(2.5)
```

### Images

```python
slide.shapes.add_picture("screenshot.png", Inches(1), Inches(2),
                          width=Inches(4))

# From BytesIO (no disk write)
from io import BytesIO
slide.shapes.add_picture(image_stream, left, top, width=Inches(4))
```

### Saving

```python
prs.save("deck.pptx")

# To BytesIO
buf = BytesIO()
prs.save(buf)
buf.seek(0)
```

### Slide design principles

- **One takeaway per slide.** The title states the insight; the body
  provides evidence. Avoid "Section 3" or "Data" as slide titles.
- **5×5 rule.** At most 5 bullets, at most 5 words each. Prefer one
  key data point over three mediocre ones.
- **Slide count.** Aim for 1 slide per minute of talk time; executive
  briefings: 10-15 slides maximum.
- **Sequence.** Title → Problem/Context → Key Data → Insight →
  Recommendation → Next Steps. Each slide answers one question the
  previous raised.
- **Consistent layout.** Use slide layouts from the template — do not
  position elements by eye. Grid-aligned elements look professional;
  manually-placed text boxes drift.

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Starting from a blank `Presentation()` instead of a brand template.** A blank deck uses python-pptx's default fonts and a white background. Corporate and client decks must open from the organization's template PPTX to inherit brand colors, fonts, and slide layouts. Always use `Presentation("template.pptx")` when the output will be shared.
- **Using placeholder index 1 without verifying the layout.** Placeholder indices vary by slide layout and template. Hardcoding `slide.placeholders[1]` for the body works on the default "Title and Content" layout but fails silently on others, placing text in the wrong element or raising an `IndexError`. Always inspect `ph.placeholder_format.idx` and `ph.name` for the target layout.
- **Putting the same text in every slide title.** Titles like "Data", "Results", or "Analysis" describe the format, not the insight. Slide titles should state the takeaway so an executive reading only the titles can follow the narrative. Rewrite generic titles as findings.
- **Adding text boxes instead of using layout placeholders.** Manually positioned text boxes do not respect the template's text styles, and they shift when the file is opened on different screen sizes. Use `slide.placeholders` for content that fits a layout; reserve `add_textbox()` only for truly free-form annotations.
- **Not setting chart title and axis labels.** A chart without a title and labeled axes forces the reader to infer context from surrounding text. Every chart must have a descriptive `chart_title` and, for all axes with units, an axis title set via `chart.value_axis.axis_title`.
- **Saving to the same path as the template.** `prs.save("brand_template.pptx")` overwrites the template, making the next generation run produce corrupt output or inherit changes from the previous run. Always save to a distinct output path.
- **Not opening the generated PPTX in PowerPoint or LibreOffice before delivery.** python-pptx can produce XML that opens without error but renders incorrectly — font fallbacks, missing charts, mispositioned images. Always open the output file visually before sharing it.

## Full reference

### Shape positioning constants

All positions and sizes in python-pptx are in EMUs (English Metric
Units). Helpers: `Inches(n)` = n×914400 EMUs, `Pt(n)` = n×12700 EMUs,
`Cm(n)` = n×360000 EMUs.

Widescreen (16:9) slide: 12192000 × 6858000 EMU (9.14 × 5.14 in).
Standard (4:3) slide: 9144000 × 6858000 EMU.

### Custom slide size

```python
from pptx.util import Inches
prs.slide_width  = Inches(13.33)   # widescreen 16:9
prs.slide_height = Inches(7.5)
```

### Speaker notes

```python
notes_slide = slide.notes_slide
notes_slide.notes_text_frame.text = "Talking points for this slide..."
```

### Duplicate a slide

python-pptx has no built-in slide-copy API. The pattern:

```python
import copy
from lxml import etree

template_slide = prs.slides[0]
xml_str = etree.tostring(template_slide._element)
new_el = etree.fromstring(xml_str)
prs.slides._sldIdLst.append(new_el)
```

### Animation

python-pptx does not support animation. To add animations, inject
OOXML directly into the slide's `_element` or use a macro-enabled
PPTM as the template.

### Conversion to PDF

`libreoffice --headless --convert-to pdf deck.pptx`

Or use the Google Slides API to import and export as PDF.
