# LaTeX Packages Reference

Detailed usage for each recommended package.

## Layout and Typography

### geometry
Page dimensions and margins.
```latex
\usepackage[a4paper, margin=2.5cm]{geometry}
% Or fine-grained control:
\usepackage[
  top=3cm, bottom=3cm, left=2.5cm, right=2.5cm,
  headheight=14pt
]{geometry}
```

### fancyhdr
Custom headers and footers.
```latex
\usepackage{fancyhdr}
\pagestyle{fancy}
\fancyhf{}                          % Clear all
\fancyhead[L]{\leftmark}            % Section name on left
\fancyhead[R]{\thepage}             % Page number on right
\fancyfoot[C]{\thepage}             % Page number centered in footer
\renewcommand{\headrulewidth}{0.4pt}
```

### fontspec (LuaLaTeX only)
System font access.
```latex
\usepackage{fontspec}
\setmainfont{TeX Gyre Termes}       % Serif (like Times)
\setsansfont{TeX Gyre Heros}        % Sans-serif (like Helvetica)
\setmonofont{Fira Code}[Scale=0.9]  % Monospace
```

### babel
Multilingual support.
```latex
\usepackage[english]{babel}          % Single language
\usepackage[ngerman, english]{babel} % Last = default
% Switch: \selectlanguage{ngerman} or \foreignlanguage{ngerman}{text}
```

### enumitem
Customizable lists.
```latex
\usepackage{enumitem}
\begin{itemize}[nosep]              % No extra spacing
\begin{enumerate}[label=\alph*)]    % a) b) c) labeling
\begin{description}[style=nextline] % Term on its own line
% Global: \setlist{nosep, leftmargin=*}
```

## Math and Science

### amsmath
Advanced math. See `math-reference.md` for full details.
```latex
\usepackage{amsmath}
% Provides: align, gather, cases, \text{}, \binom{}, \operatorname{}
% Also load: \usepackage{amssymb} for extra symbols
```

### siunitx
SI units and number formatting.
```latex
\usepackage{siunitx}
\qty{9.8}{m/s^2}                    % 9.8 m/s^2
\num{1.23e4}                        % 1.23 x 10^4
\num{12345.678}                     % 12 345.678 (with grouping)
\unit{kg.m/s^2}                     % kg m/s^2
\ang{45;30;0}                       % 45 degrees 30 minutes
\qtyrange{10}{20}{m}                % 10 m to 20 m
% Setup: \sisetup{per-mode=fraction, locale=DE}
```

## Tables and Figures

### tabularray
Modern table package (replaces tabular + booktabs).
```latex
\usepackage{tabularray}
\begin{tblr}{
  colspec = {lXr},                   % Left, flexible, right
  row{1} = {font=\bfseries, bg=gray9},
  hlines, vlines,
}
  Name & Description & Value \\
  Item & Long description text & 42 \\
\end{tblr}
% For long tables: \begin{longtblr} with \SetLongTblr
```

### graphicx
Image inclusion.
```latex
\usepackage{graphicx}
\graphicspath{{figures/}}            % Default search path
\includegraphics[width=0.8\textwidth]{diagram}  % .pdf, .png, .jpg
% Always wrap in figure float:
\begin{figure}[htbp]
  \centering
  \includegraphics[width=0.7\textwidth]{photo}
  \caption{Description of the figure.}
  \label{fig:photo}
\end{figure}
```

### svg (requires Inkscape installed)
SVG file inclusion via automatic conversion.
```latex
\usepackage{svg}
\includesvg[width=0.5\textwidth]{diagram}  % .svg extension optional
% Requires: --shell-escape flag for latexmk
% In latexmkrc: $lualatex = 'lualatex --shell-escape %O %S';
```

## Graphics and Color

### tikz
See `tikz-reference.md` for full drawing reference.
```latex
\usepackage{tikz}
\usetikzlibrary{arrows.meta, positioning, calc, shapes, fit}
```

### pgfkeys
Key-value interface (used internally by TikZ; useful for custom packages).
```latex
\pgfkeys{
  /mypackage/.cd,
  title/.store in = \mytitle,
  color/.store in = \mycolor,
  color = blue,                      % Default
}
\pgfkeys{/mypackage, title=Hello, color=red}
```

### xcolor
Extended color support.
```latex
\usepackage[dvipsnames, svgnames, x11names]{xcolor}
\definecolor{brand}{HTML}{2563EB}
\colorlet{accent}{brand!70!white}
\textcolor{brand}{colored text}
\colorbox{accent}{highlighted box}
% Mixing: red!50!blue = 50% red + 50% blue
```

### tcolorbox
Colored boxes for theorems, notes, code.
```latex
\usepackage[most]{tcolorbox}
\newtcolorbox{note}{
  colback=blue!5, colframe=blue!50,
  title=Note, fonttitle=\bfseries
}
\begin{note}
  This is a highlighted note.
\end{note}
% Also supports listings integration:
\newtcblisting{code}{listing only, colback=gray!5}
```

## Bibliography

### biblatex
Modern bibliography management.
```latex
\usepackage[
  backend=biber,
  style=authoryear,                  % or: numeric, ieee, apa, chicago-authordate
  sorting=nyt,                       % name-year-title
  maxbibnames=99,
  giveninits=true,
]{biblatex}
\addbibresource{references.bib}
% Usage: \textcite{Key}, \autocite{Key}, \fullcite{Key}
% Print: \printbibliography[heading=bibintoc]
```

## Specialty Packages

### bytefield
Protocol byte/bit field diagrams.
```latex
\usepackage{bytefield}
\begin{bytefield}[bitwidth=1.1em]{32}
  \bitheader{0-31} \\
  \bitbox{4}{Version} & \bitbox{4}{IHL} & \bitbox{8}{Type of Service}
  & \bitbox{16}{Total Length} \\
\end{bytefield}
```

### xskak
Chess game notation and board rendering.
```latex
\usepackage{xskak}
\newchessgame
\mainline{1. e4 e5 2. Nf3 Nc6 3. Bb5}
\chessboard[setfen=\xskakget{nextfen}]
```

### emoji (LuaLaTeX only)
Emoji support.
```latex
\usepackage{emoji}
This is great \emoji{thumbs-up} and fun \emoji{rocket}
% Requires LuaLaTeX and the Noto Color Emoji font
```
