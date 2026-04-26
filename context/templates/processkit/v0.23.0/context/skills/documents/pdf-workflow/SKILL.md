---
name: pdf-workflow
description: |
  Generating and manipulating PDF files — from HTML, DOCX, or raw content using reportlab, weasyprint, or pypdf. Use when producing PDF reports, invoices, certificates, or when merging, splitting, or extracting text from existing PDFs.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-pdf-workflow
    version: "1.0.0"
    created: 2026-04-08T00:00:00Z
    category: documents
    layer: 2
---

# PDF Workflow

## Intro

Choose your tool based on the source format. **WeasyPrint** converts
HTML+CSS to PDF — the right choice for report and invoice generation
because HTML is easy to template. **ReportLab** builds PDFs from
primitives (text, lines, images) — good for certificates and pixel-
precise layouts. **pypdf** reads, merges, splits, and annotates
existing PDFs without re-rendering.

## Overview

### Tool selection

| Need | Tool | Notes |
|---|---|---|
| HTML/CSS → PDF | `weasyprint` | Jinja2 templates + CSS for layout |
| Programmatic layout | `reportlab` | `platypus` for flow; `canvas` for precise |
| Merge / split / rotate | `pypdf` | Pure manipulation; no re-render |
| Extract text | `pypdf` or `pdfminer.six` | `pdfminer` handles complex encodings |
| Fill PDF forms | `pypdf` | AcroForms via `pypdf.PdfWriter` |
| Watermark / stamp | `pypdf` | Merge a watermark page onto each page |

Install: `uv add weasyprint reportlab pypdf` (or whichever subset
you need).

### WeasyPrint (HTML → PDF)

```python
# /// script
# dependencies = ["weasyprint", "jinja2"]
# ///
from weasyprint import HTML, CSS
from jinja2 import Environment, FileSystemLoader

env = Environment(loader=FileSystemLoader("templates"))
template = env.get_template("invoice.html")
html_str = template.render(invoice_number="INV-042", total=1250.00,
                            line_items=[...])

# Render to file
HTML(string=html_str).write_pdf("invoice.pdf")

# Render to BytesIO
from io import BytesIO
buf = BytesIO()
HTML(string=html_str).write_pdf(buf)
buf.seek(0)
```

**CSS layout tips for PDF:**
- `@page { size: A4; margin: 2cm 2.5cm; }` sets paper size and margins.
- `page-break-before: always` / `page-break-after: avoid` control page breaks.
- `@media print { .no-print { display: none; } }` hides screen-only elements.
- WeasyPrint supports Flexbox and Grid but not JavaScript. Keep CSS to what
  a print stylesheet would use.
- Embed fonts with `@font-face` to guarantee exact rendering.

### ReportLab — Platypus (document flow)

```python
# /// script
# dependencies = ["reportlab"]
# ///
from reportlab.platypus import SimpleDocTemplate, Paragraph, Spacer, Table, TableStyle
from reportlab.lib.styles import getSampleStyleSheet, ParagraphStyle
from reportlab.lib.pagesizes import A4
from reportlab.lib.units import cm
from reportlab.lib import colors

styles = getSampleStyleSheet()

doc = SimpleDocTemplate("report.pdf", pagesize=A4,
                        leftMargin=2.5*cm, rightMargin=2.5*cm,
                        topMargin=2*cm, bottomMargin=2*cm)

story = []
story.append(Paragraph("Annual Report 2026", styles["Title"]))
story.append(Spacer(1, 0.5*cm))
story.append(Paragraph("Executive summary goes here...", styles["BodyText"]))

# Table
data = [["Metric", "Value"], ["Revenue", "$1.25M"], ["Growth", "18%"]]
t = Table(data, colWidths=[8*cm, 6*cm])
t.setStyle(TableStyle([
    ("BACKGROUND", (0, 0), (-1, 0), colors.HexColor("#2E74B5")),
    ("TEXTCOLOR",  (0, 0), (-1, 0), colors.white),
    ("FONTNAME",   (0, 0), (-1, 0), "Helvetica-Bold"),
    ("GRID",       (0, 0), (-1, -1), 0.5, colors.grey),
    ("ROWBACKGROUNDS", (0, 1), (-1, -1), [colors.white, colors.HexColor("#EBF3FA")]),
]))
story.append(t)

doc.build(story)
```

### ReportLab — Canvas (precise layout)

```python
from reportlab.pdfgen import canvas
from reportlab.lib.pagesizes import A4
from reportlab.lib.units import cm

c = canvas.Canvas("certificate.pdf", pagesize=A4)
width, height = A4

c.setFont("Helvetica-Bold", 36)
c.drawCentredString(width / 2, height - 6*cm, "Certificate of Completion")

c.setFont("Helvetica", 18)
c.drawCentredString(width / 2, height - 9*cm, "Awarded to: Alice Smith")

c.drawImage("seal.png", width/2 - 3*cm, 3*cm, width=6*cm, height=6*cm,
            mask="auto")
c.showPage()
c.save()
```

Use `canvas` for certificates, labels, and receipts where exact
positioning matters. Use `platypus` for multi-page documents with
flowing text.

### pypdf — merge, split, rotate, extract

```python
# /// script
# dependencies = ["pypdf"]
# ///
from pypdf import PdfReader, PdfWriter

# Merge PDFs
writer = PdfWriter()
for path in ["cover.pdf", "body.pdf", "appendix.pdf"]:
    reader = PdfReader(path)
    for page in reader.pages:
        writer.add_page(page)
with open("merged.pdf", "wb") as f:
    writer.write(f)

# Extract pages 2-5 (0-indexed)
reader = PdfReader("report.pdf")
writer = PdfWriter()
for page in reader.pages[1:5]:
    writer.add_page(page)
with open("excerpt.pdf", "wb") as f:
    writer.write(f)

# Extract text
reader = PdfReader("document.pdf")
for page in reader.pages:
    print(page.extract_text())

# Rotate a page
reader.pages[0].rotate(90)
```

### Watermarking

```python
from pypdf import PdfReader, PdfWriter

watermark = PdfReader("watermark.pdf").pages[0]
reader = PdfReader("original.pdf")
writer = PdfWriter()

for page in reader.pages:
    page.merge_page(watermark)
    writer.add_page(page)

with open("watermarked.pdf", "wb") as f:
    writer.write(f)
```

### Accessibility checklist

Before delivering a generated PDF:
- Set document title, author, and language in metadata.
- Ensure text is selectable (not a rasterized image).
- Add alt text for images (requires PDF/UA tagging — ReportLab
  Platypus supports this via `platypus.flowables.AnchoredFlowable`).
- Verify 4.5:1 color contrast for body text.
- Test that copy-paste produces legible text.

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Choosing ReportLab canvas for a multi-page report.** Canvas is a low-level drawing API — positioning every element manually on every page is tedious and error-prone for flowing content. Multi-page documents with headings, body text, and tables belong in ReportLab's Platypus framework or in WeasyPrint. Reserve canvas for certificate-style layouts where exact pixel placement matters.
- **Using WeasyPrint with JavaScript-dependent HTML.** WeasyPrint does not execute JavaScript. A template that uses JS to populate data, collapse sections, or set styles will silently produce a blank or wrong PDF. All data must be server-rendered into the HTML before passing to WeasyPrint.
- **Not embedding fonts in the PDF.** A PDF that references system fonts will render differently on machines where those fonts are not installed. WeasyPrint embeds fonts via `@font-face`; ReportLab registers fonts with `pdfmetrics.registerFont`. Always embed or use the built-in safe set (Helvetica, Times-Roman, Courier) for documents that travel.
- **Treating `page.extract_text()` output as structured data.** `pypdf`'s text extraction preserves reading order in simple PDFs but loses column structure in multi-column layouts, tables, and PDFs produced from scans. For structured extraction from complex PDFs use `pdfminer.six` with its layout analysis, or a specialized OCR pipeline.
- **Merging PDFs without normalizing page sizes first.** Merging an A4 page and a Letter-size page produces a PDF where every other page is a different size. Verify that all source PDFs have the same page dimensions before merging, or resize pages to a common size with ReportLab canvas as an intermediary.
- **Not setting document metadata (title, author, language).** A PDF without `title` and `lang` metadata fails basic accessibility checks and shows "Untitled" in browser tab bars and screen readers. Set `writer.add_metadata({"/Title": "...", "/Author": "..."})` (pypdf) or `doc.build(story, onFirstPage=set_metadata)` (ReportLab) before saving.
- **Generating PDFs with rasterized text (converting HTML to image first).** Tools that screenshot a page and save the image as a PDF produce files where text is not selectable, searchable, or accessible. Always generate PDFs from text-based sources (HTML, Platypus, canvas) so the text layer exists in the output.

## Full reference

### Page sizes

```python
from reportlab.lib.pagesizes import A4, A3, LETTER, LEGAL, landscape
# A4 = (595.28, 841.89) points
# landscape(A4) = (841.89, 595.28)
```

WeasyPrint CSS: `@page { size: A4 landscape; }` or
`@page { size: 210mm 297mm; }`.

### PDF metadata (pypdf)

```python
writer.add_metadata({
    "/Title": "Q1 Report",
    "/Author": "Finance Team",
    "/Subject": "Quarterly Financial Report",
    "/Keywords": "finance, Q1, 2026",
    "/Creator": "processkit pdf-workflow",
    "/Lang": "en-US",
})
```

### PDF/A compliance (archival)

For archival PDFs (legal, government, long-term storage) you may need
PDF/A-1b compliance. WeasyPrint supports this with a CSS flag:
`HTML(string=html).write_pdf("out.pdf", presentational_hints=True)`.
Full PDF/A-1b requires embedding all fonts, ICC color profiles, and
setting the correct XMP metadata — use a specialized library like
`pdfa-generation` or validate output with `VeraPDF`.

### Encryption (pypdf)

```python
writer.encrypt("owner_password", "user_password",
               use_128bit=True, permissions_flag=-1)
```

`permissions_flag=-1` allows all operations. To restrict printing,
set specific bits in the permissions integer.

### pdfminer text extraction (complex layouts)

```python
from pdfminer.high_level import extract_text, extract_pages
from pdfminer.layout import LTTextBox

text = extract_text("document.pdf")

for page_layout in extract_pages("document.pdf"):
    for element in page_layout:
        if isinstance(element, LTTextBox):
            print(element.get_text())
```
