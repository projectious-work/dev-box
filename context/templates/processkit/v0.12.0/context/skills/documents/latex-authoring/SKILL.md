---
name: latex-authoring
description: |
  LaTeX authoring with LuaLaTeX — document classes, math, TikZ, BibLaTeX. Use when writing or editing LaTeX documents, setting up preambles, typesetting math, building TikZ diagrams, or managing bibliographies.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-latex-authoring
    version: "1.1.0"
    created: 2026-04-06T00:00:00Z
    category: documents
---

# LaTeX Authoring

## Intro

Prefer LuaLaTeX for new documents — native Unicode, system fonts
via `fontspec`, and better TikZ performance. Use modern packages
(`tabularray`, `siunitx`, `biblatex`, `cleveref`) and structure
documents for reviewable diffs: one sentence per line, sections in
their own files.

## Overview

### Document classes

| Class         | Use for                                              |
|---------------|------------------------------------------------------|
| `article`     | Papers, short documents (no chapters)                |
| `book`        | Books, theses (chapters, front/back matter)          |
| `report`      | Technical reports (chapters, no front/back matter)   |
| `extarticle`  | Articles needing 8pt–20pt font sizes                 |
| `memoir`      | Flexible class combining book and article features  |
| `beamer`      | Presentations and slides                             |
| `standalone`  | Single figures or TikZ diagrams for inclusion        |

### LuaLaTeX vs pdfLaTeX

Prefer LuaLaTeX for new projects — native Unicode, OpenType/TrueType
fonts via `fontspec`, Lua scripting, better TikZ performance on
complex diagrams, and packages like `fontspec` and `emoji` require
it. Stick with pdfLaTeX only when a legacy project or publisher
requires it, or when compile speed matters on trivial documents.

Build with `latexmk -lualatex main.tex` (or `-pdf` for pdfLaTeX).
`latexmk` runs biber and reruns the engine as needed.

### Essential packages

**Layout and typography:** `geometry` (margins), `fancyhdr`
(headers/footers), `fontspec` (system fonts, LuaLaTeX only),
`babel` (multilingual), `enumitem` (custom lists).

**Math and science:** `amsmath` (environments and commands),
`siunitx` (units and number formatting).

**Tables and figures:** `tabularray` (modern tables, replaces
`tabular`/`booktabs`), `graphicx` (images), `svg` (SVG inclusion,
requires Inkscape).

**Graphics and color:** `tikz` (programmatic vector graphics),
`pgfkeys` (key-value interface), `xcolor` (extended color),
`tcolorbox` (colored boxes, theorems, code listings).

**Bibliography:** `biblatex` with the Biber backend — modern
replacement for `bibtex` and `natbib`.

**Specialty:** `bytefield` (protocol diagrams), `xskak` (chess),
`emoji` (LuaLaTeX only).

See `references/packages.md` for detailed usage and configuration
of each package.

### Document structure

Write one sentence per line — it produces cleaner git diffs and
makes review much easier. Split large documents with
`\input{sections/introduction}` per section. Keep the preamble in
a separate `preamble.tex` so you can reuse it. Put `\label{}` on
every float, section, and equation, and reference with `\cref{}`
(from `cleveref`) rather than bare `\ref{}`.

### Bibliography with BibLaTeX

```latex
\usepackage[backend=biber, style=authoryear]{biblatex}
\addbibresource{references.bib}

% In text:
\textcite{Knuth1984} writes that...     % Knuth (1984) writes that...
As shown before \autocite{Knuth1984}    % As shown before (Knuth, 1984)

% At end:
\printbibliography
```

Build with `latexmk -lualatex main.tex` — it handles biber runs
automatically.

### Math

Inline math uses `$...$` or `\(...\)`; display math uses `\[...\]`
(never `$$...$$`). Use `align`, `gather`, and `cases` from
`amsmath` for multi-line equations. Define reusable commands in
the preamble (`\newcommand{\vect}[1]{\mathbf{#1}}`). Use `siunitx`
for all units: `\qty{9.8}{m/s^2}`, `\num{1.23e4}`. See
`references/math-reference.md` for a symbol and environment
reference.

### TikZ

Load with `\usepackage{tikz}` plus `\usetikzlibrary{...}`. Common
libraries: `arrows.meta`, `positioning`, `calc`, `shapes`, `fit`.
Use the `standalone` class for diagrams you want to include from
multiple documents. Define styles once in the preamble to keep
picture bodies readable. See `references/tikz-reference.md` for
drawing patterns.

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Using `$$...$$` for display math instead of `\[...\]`.** `$$` is a plain TeX primitive that does not integrate with LaTeX's vertical spacing or equation numbering systems. Use `\[...\]` or `equation`/`align` environments; `$$` produces subtle spacing bugs and incompatibilities with `amsmath`.
- **Defaulting to pdfLaTeX for new projects.** pdfLaTeX does not support Unicode source natively or OpenType fonts without workarounds. New projects should use LuaLaTeX with `fontspec`; reserve pdfLaTeX for legacy compatibility or publisher requirements that specifically mandate it.
- **Loading packages without checking known conflict order rules.** Package load order matters: `hyperref` must almost always be loaded last, and combinations like `babel` with `fontspec` have documented interaction requirements. When packages conflict, changing load order is the first fix to try before reaching for hacks.
- **Using `{\large ...}` or `{\small ...}` to adjust font sizes inline.** Hardcoded size switches do not adapt to document class changes and break if the base font size changes. Define named semantic commands in the preamble and apply them consistently throughout.
- **Writing units via string concatenation instead of `siunitx`.** `$9.8 m/s^2$` produces inconsistent spacing and formatting; `\qty{9.8}{m/s^2}` from `siunitx` formats consistently across locales and handles magnitude formatting automatically. Use `siunitx` for all quantities and units.
- **Running the LaTeX engine once instead of `latexmk`.** A single engine pass leaves cross-references, table of contents, and bibliography stale. `latexmk -lualatex` reruns the engine and biber as many times as needed until the output stabilizes.
- **Relying on locally installed fonts without documenting the dependency.** A document that compiles on one machine because a specific OpenType font is installed will fail elsewhere. Document all font dependencies and prefer fonts available via a standard TeX distribution or bundled with the project.

## Full reference

### Minimal LuaLaTeX preamble

```latex
\documentclass{article}
\usepackage[margin=1in]{geometry}
\usepackage{fontspec}
\usepackage{amsmath}
\usepackage{siunitx}
\usepackage{graphicx}
\usepackage{tabularray}
\usepackage{tikz}
\usepackage{xcolor}
\usepackage[backend=biber, style=authoryear]{biblatex}
\usepackage{cleveref}
\addbibresource{references.bib}
```

### Common mistakes

- Using `$$...$$` for display math — it is not plain-TeX in
  LaTeX. Use `\[...\]` or an `equation` environment.
- Manual spacing with `\,` and `~` instead of proper environments.
- Forgetting `\label` immediately after `\caption` inside a float
  — the label must come after the caption, or it references the
  enclosing section instead.
- Including bitmap images where vector (PDF or SVG) would produce
  crisper output at no cost.
- Forgetting to run biber when the bibliography does not appear —
  use `latexmk` and it is handled automatically.
- Using `\ref{}` alone when `\cref{}` would produce "Figure 3"
  instead of just "3".
- Writing multi-sentence lines, which makes git diffs unreadable.
- Building with pdfLaTeX when the preamble loads `fontspec`,
  `unicode-math`, or `emoji`.

### Tables with tabularray

`tabularray` replaces the older `tabular`/`booktabs` combo with a
key-value interface and saner defaults. Use `tblr` for tables in a
`table` float and the `booktabs` library for rules.

### Standalone diagrams

When a TikZ diagram will be reused or is complex enough to warrant
its own file, build it with the `standalone` class so it produces a
cropped PDF you can `\includegraphics` from the main document. This
keeps the main compile fast and lets you iterate on diagrams
independently.

### Further reading

- `references/packages.md` — per-package usage and configuration
- `references/math-reference.md` — math symbols, environments, and
  formulas
- `references/tikz-reference.md` — TikZ drawing commands and
  patterns
