---
name: latex-authoring
description: Comprehensive LaTeX document authoring with LuaLaTeX, modern packages, math, TikZ, and bibliography management. Use when writing or editing LaTeX documents.
---

# LaTeX Authoring

## When to Use

When the user asks to:
- Write or edit LaTeX documents (.tex, .bib, .sty)
- Set up document classes, packages, or preambles
- Create math equations, tables, figures, or diagrams
- Manage bibliographies with BibLaTeX
- Use TikZ for programmatic graphics
- Choose between LuaLaTeX and pdfLaTeX

## Instructions

### 1. Document Classes

| Class | Use For |
|---|---|
| `article` | Papers, short documents (no chapters) |
| `book` | Books, theses (chapters, front/back matter) |
| `report` | Technical reports (chapters without front/back matter) |
| `extarticle` | Articles with extended font sizes (8pt-20pt) |
| `memoir` | Flexible class combining book+article features |
| `beamer` | Presentations and slides |
| `standalone` | Single figures or TikZ diagrams for inclusion |

### 2. LuaLaTeX vs pdfLaTeX

**Prefer LuaLaTeX** for new projects:
- Native Unicode support (no input encoding issues)
- System font access via `fontspec` (OpenType, TrueType)
- Lua scripting for programmatic content
- Better `tikz` performance for complex diagrams
- Required by: `fontspec`, `emoji`

**Use pdfLaTeX** when:
- Legacy project requires it
- Publisher mandates pdfLaTeX
- Faster compile times needed for simple documents

Build command: `latexmk -lualatex main.tex` (or `-pdf` for pdfLaTeX)

### 3. Essential Packages

See `references/packages.md` for detailed usage of each package.

**Layout and typography:**
- `geometry` --- Page margins and dimensions
- `fancyhdr` --- Custom headers and footers
- `fontspec` --- System fonts (LuaLaTeX only)
- `babel` --- Multilingual support
- `enumitem` --- Customizable lists

**Math and science:**
- `amsmath` --- Advanced math environments and commands
- `siunitx` --- SI units and number formatting

**Tables and figures:**
- `tabularray` --- Modern table typesetting (replaces tabular/booktabs)
- `graphicx` --- Image inclusion
- `svg` --- SVG file inclusion (requires Inkscape)

**Graphics and color:**
- `tikz` --- Programmatic vector graphics
- `pgfkeys` --- Key-value interface for TikZ and custom packages
- `xcolor` --- Extended color support
- `tcolorbox` --- Colored boxes, theorems, code listings

**Bibliography:**
- `biblatex` --- Modern bibliography management (use with Biber backend)

**Specialty:**
- `bytefield` --- Protocol byte/bit field diagrams
- `xskak` --- Chess game notation and board diagrams
- `emoji` --- Emoji in LaTeX (LuaLaTeX only)

### 4. Document Structure Best Practices

- One sentence per line (better git diffs, easier review)
- Split large documents: `\input{sections/introduction}` per section
- Keep preamble in a separate `preamble.tex` for reuse
- Use `\label{}` on every float, section, and equation
- Reference with `\ref{}` or `\cref{}` (cleveref package)

### 5. Bibliography with BibLaTeX

```latex
\usepackage[backend=biber, style=authoryear]{biblatex}
\addbibresource{references.bib}
% In text:
\textcite{Knuth1984} writes that...  % Knuth (1984) writes that...
As shown before \autocite{Knuth1984}  % As shown before (Knuth, 1984)
% At end:
\printbibliography
```

Build: `latexmk -lualatex main.tex` (latexmk handles biber automatically)

### 6. Math (see references/math-reference.md)

- Inline: `$...$` or `\(...\)`
- Display: `\[...\]` (never `$$...$$`)
- Multi-line: `align`, `gather`, `cases` (from amsmath)
- Define custom commands: `\newcommand{\vect}[1]{\mathbf{#1}}`
- Use `siunitx` for units: `\qty{9.8}{m/s^2}`, `\num{1.23e4}`

### 7. TikZ (see references/tikz-reference.md)

- Load with `\usepackage{tikz}` and `\usetikzlibrary{...}`
- Use `standalone` class for standalone diagrams
- Common libraries: `arrows.meta`, `positioning`, `calc`, `shapes`, `fit`
- For complex diagrams, define styles in the preamble

### 8. Common Mistakes to Avoid

- Using `$$...$$` for display math (use `\[...\]`)
- Not using `\qty` from siunitx for units
- Manual spacing instead of proper environments
- Bitmap images where vector (PDF/SVG) would work
- Not running biber when bibliography doesn't appear
- Missing `\label` after `\caption` in floats

## References

- `references/packages.md` --- Detailed package usage and configuration
- `references/math-reference.md` --- Math symbols, environments, and formulas
- `references/tikz-reference.md` --- TikZ drawing commands and patterns

## Examples

**User:** "Set up a LaTeX paper with LuaLaTeX"
**Agent:** Creates a main.tex with `\documentclass{article}`, a preamble.tex loading
geometry, fontspec, amsmath, biblatex, and siunitx. Sets up section structure with
`\input{}` and provides a latexmk build command.

**User:** "Add a table of results"
**Agent:** Uses `tabularray` for a properly formatted table with caption, label,
and column alignment. Places in a `table` float environment.

**User:** "Draw a system architecture diagram"
**Agent:** Creates a TikZ diagram using the `positioning` library, with labeled
nodes connected by arrows. Uses `standalone` class for easy inclusion.
