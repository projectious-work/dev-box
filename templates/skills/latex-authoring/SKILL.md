---
name: latex-authoring
description: LaTeX document authoring — structure, bibliography, figures, tables. Use when writing or editing LaTeX documents.
---

# LaTeX Authoring

## When to Use

When the user is working with LaTeX files (.tex, .bib) and asks about document structure, formatting, bibliography management, or says "help me with this LaTeX document".

## Instructions

1. **Document structure:**
   - Use `\documentclass{article}` for papers, `{report}` for longer documents
   - Split large documents: `\input{sections/introduction}` per chapter/section
   - Keep preamble in a separate `preamble.tex` for reuse
2. **Bibliography:**
   - Use BibLaTeX with Biber backend (not BibTeX)
   - Store references in `references.bib`
   - Cite with `\autocite{}` or `\textcite{}` (not `\cite{}`)
   - Use consistent BibTeX key format: `AuthorYear` (e.g., `Knuth1984`)
3. **Figures and tables:**
   - Always use `\begin{figure}[htbp]` with `\centering`
   - Include `\caption{}` and `\label{fig:name}` for every float
   - Reference with `\cref{fig:name}` (cleveref package)
   - Store images in an `figures/` subdirectory
4. **Best practices:**
   - One sentence per line (better git diffs)
   - Use `\SI{}{}` from siunitx for units
   - Define custom commands for repeated notation: `\newcommand{\vect}[1]{\mathbf{#1}}`
   - Use `latexmk` for building: `latexmk -pdf main.tex`
5. **Common packages:** geometry, amsmath, graphicx, hyperref, cleveref, siunitx, booktabs

## Examples

**User:** "Add a table of results"
**Agent:** Creates a `booktabs`-style table with proper `\caption`, `\label`, and column alignment. Places it in a `table` float environment.
