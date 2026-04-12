# TikZ Drawing Reference

Quick reference for TikZ programmatic graphics.

## Setup

```latex
\usepackage{tikz}
\usetikzlibrary{
  arrows.meta,    % Modern arrow tips
  positioning,    % Relative positioning (above=of, right=of)
  calc,           % Coordinate calculations
  shapes,         % Additional node shapes
  fit,            % Fitting nodes around other nodes
  backgrounds,    % Background layers
  decorations.pathreplacing,  % Braces, waves
}
```

## Basic Drawing

```latex
\begin{tikzpicture}
  % Lines
  \draw (0,0) -- (2,0) -- (2,2) -- cycle;    % Triangle
  \draw[thick, ->] (0,0) -- (3,1);            % Arrow
  \draw[dashed, red] (0,0) -- (2,2);          % Dashed red line

  % Shapes
  \draw (0,0) circle (1cm);                    % Circle
  \draw (0,0) ellipse (2cm and 1cm);           % Ellipse
  \draw (0,0) rectangle (3,2);                 % Rectangle
  \filldraw[fill=blue!20, draw=blue] (0,0) circle (0.5);

  % Curves
  \draw (0,0) .. controls (1,2) and (3,2) .. (4,0);  % Bezier
  \draw plot[smooth] coordinates {(0,0) (1,1) (2,0) (3,1)};

  % Grid
  \draw[step=1, gray, very thin] (0,0) grid (4,4);
\end{tikzpicture}
```

## Nodes

```latex
% Basic nodes
\node at (0,0) {Text};
\node[draw] at (2,0) {Boxed};
\node[draw, circle] at (4,0) {Circle};
\node[draw, rounded corners] at (6,0) {Rounded};

% Node styling
\node[
  draw, fill=blue!20,
  minimum width=3cm, minimum height=1cm,
  font=\bfseries, text=white,
  rounded corners=5pt
] (mynode) at (0,0) {Styled Node};

% Relative positioning (requires positioning library)
\node[draw] (a) {A};
\node[draw, right=2cm of a] (b) {B};
\node[draw, below=1cm of a] (c) {C};
\node[draw, above right=1cm and 2cm of a] (d) {D};
```

## Connecting Nodes

```latex
% Straight arrows
\draw[->] (a) -- (b);
\draw[->] (a) -- node[above] {label} (b);

% Right-angle connections
\draw[->] (a) -| (b);          % Horizontal then vertical
\draw[->] (a) |- (b);          % Vertical then horizontal

% Curved arrows
\draw[->, bend left=30] (a) to (b);
\draw[->, bend right=30] (a) to node[right] {label} (b);
\draw[->, out=90, in=180] (a) to (b);

% Arrow tips (arrows.meta)
\draw[-Stealth] (a) -- (b);          % Filled triangle
\draw[-{Latex[scale=1.5]}] (a) -- (b);  % Large latex arrow
\draw[<->] (a) -- (b);               % Bidirectional
```

## Styles

```latex
% Define styles in tikzpicture options or preamble
\tikzset{
  process/.style = {
    draw, rectangle, rounded corners,
    minimum width=3cm, minimum height=1cm,
    fill=blue!10, font=\small
  },
  decision/.style = {
    draw, diamond, aspect=2,
    fill=yellow!20, font=\small
  },
  arrow/.style = {
    -Stealth, thick
  },
}

% Use styles
\node[process] (start) {Start};
\node[decision, below=of start] (check) {OK?};
\draw[arrow] (start) -- (check);
```

## Common Diagram Patterns

### Flowchart
```latex
\begin{tikzpicture}[
  process/.style={draw, rectangle, rounded corners, minimum width=3cm,
    minimum height=0.8cm, fill=blue!10},
  decision/.style={draw, diamond, aspect=2, fill=yellow!10},
  io/.style={draw, trapezium, trapezium left angle=70,
    trapezium right angle=110, fill=green!10},
  arrow/.style={-Stealth, thick},
  node distance=1.2cm
]
  \node[io] (input) {Input};
  \node[process, below=of input] (proc) {Process};
  \node[decision, below=of proc] (dec) {Valid?};
  \node[process, below=of dec] (output) {Output};
  \node[process, right=2cm of dec] (error) {Error};

  \draw[arrow] (input) -- (proc);
  \draw[arrow] (proc) -- (dec);
  \draw[arrow] (dec) -- node[left] {Yes} (output);
  \draw[arrow] (dec) -- node[above] {No} (error);
  \draw[arrow] (error) |- (proc);
\end{tikzpicture}
```

### Block Diagram
```latex
\begin{tikzpicture}[
  block/.style={draw, rectangle, minimum width=2.5cm, minimum height=1cm,
    rounded corners, fill=blue!10},
  db/.style={draw, cylinder, shape border rotate=90, aspect=0.3,
    minimum width=2cm, minimum height=1.2cm, fill=yellow!10},
  node distance=1.5cm and 2cm,
  arrow/.style={-Stealth, thick}
]
  \node[block] (client) {Client};
  \node[block, right=of client] (api) {API};
  \node[db, right=of api] (db) {Database};

  \draw[arrow] (client) -- node[above] {HTTP} (api);
  \draw[arrow] (api) -- node[above] {SQL} (db);
\end{tikzpicture}
```

### Fit and Grouping
```latex
\begin{tikzpicture}
  \node[draw] (a) {A};
  \node[draw, right=of a] (b) {B};
  \node[draw, below=of a] (c) {C};

  % Fit a box around nodes (requires fit library)
  \node[draw, dashed, rounded corners, fit=(a)(b)(c),
    inner sep=10pt, label=above:Group] {};
\end{tikzpicture}
```

## Plots with pgfplots

```latex
\usepackage{pgfplots}
\pgfplotsset{compat=1.18}

\begin{tikzpicture}
\begin{axis}[
  xlabel={$x$}, ylabel={$f(x)$},
  grid=major, width=10cm, height=6cm,
  legend pos=north west,
]
  \addplot[blue, thick, domain=0:6.28, samples=100] {sin(deg(x))};
  \addlegendentry{$\sin(x)$}

  \addplot[red, thick, domain=0:6.28, samples=100] {cos(deg(x))};
  \addlegendentry{$\cos(x)$}
\end{axis}
\end{tikzpicture}
```

## Tips

- Use `standalone` document class for diagrams meant for inclusion
- Use `\tikzset` in the preamble for project-wide styles
- Use `calc` library for computed coordinates: `($(a)!0.5!(b)$)` = midpoint
- Use `foreach` for repetitive elements: `\foreach \x in {1,...,5} { ... }`
- Debug positioning: temporarily add `\draw[help lines] (0,0) grid (10,10);`
- Scale: `\begin{tikzpicture}[scale=0.8, every node/.style={scale=0.8}]`
