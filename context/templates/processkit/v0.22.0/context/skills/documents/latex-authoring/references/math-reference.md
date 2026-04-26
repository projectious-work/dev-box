# LaTeX Math Reference

Quick reference for mathematical typesetting with amsmath.

## Math Modes

```latex
Inline math: $a^2 + b^2 = c^2$ or \(a^2 + b^2 = c^2\)
Display math: \[ E = mc^2 \]    % Never use $$...$$
```

## Environments (amsmath)

### align (numbered, multi-line with alignment)
```latex
\begin{align}
  f(x) &= x^2 + 2x + 1 \\
       &= (x + 1)^2       \label{eq:square}
\end{align}
% Use align* for unnumbered
```

### gather (numbered, centered, no alignment)
```latex
\begin{gather}
  x + y = z \\
  a + b = c
\end{gather}
```

### cases (piecewise functions)
```latex
f(x) = \begin{cases}
  x^2  & \text{if } x \geq 0 \\
  -x^2 & \text{if } x < 0
\end{cases}
```

### split (single equation number for multi-line)
```latex
\begin{equation}
\begin{split}
  (a+b)^2 &= a^2 + 2ab + b^2 \\
           &= a^2 + b^2 + 2ab
\end{split}
\end{equation}
```

### matrix variants
```latex
\begin{pmatrix} a & b \\ c & d \end{pmatrix}  % (parentheses)
\begin{bmatrix} a & b \\ c & d \end{bmatrix}  % [brackets]
\begin{vmatrix} a & b \\ c & d \end{vmatrix}  % |determinant|
```

## Common Symbols

### Greek Letters
| Lower | Command | Upper | Command |
|---|---|---|---|
| alpha | `\alpha` | A | (roman A) |
| beta | `\beta` | B | (roman B) |
| gamma | `\gamma` | Gamma | `\Gamma` |
| delta | `\delta` | Delta | `\Delta` |
| epsilon | `\epsilon`, `\varepsilon` | E | (roman E) |
| theta | `\theta`, `\vartheta` | Theta | `\Theta` |
| lambda | `\lambda` | Lambda | `\Lambda` |
| mu | `\mu` | | |
| pi | `\pi` | Pi | `\Pi` |
| sigma | `\sigma` | Sigma | `\Sigma` |
| phi | `\phi`, `\varphi` | Phi | `\Phi` |
| omega | `\omega` | Omega | `\Omega` |

### Operators and Relations
| Symbol | Command | Symbol | Command |
|---|---|---|---|
| times | `\times` | div | `\div` |
| cdot | `\cdot` | pm | `\pm` |
| leq | `\leq` | geq | `\geq` |
| neq | `\neq` | approx | `\approx` |
| subset | `\subset` | supset | `\supset` |
| in | `\in` | notin | `\notin` |
| infty | `\infty` | partial | `\partial` |
| nabla | `\nabla` | forall | `\forall` |
| exists | `\exists` | implies | `\implies` |
| iff | `\iff` | therefore | `\therefore` |

### Big Operators
```latex
\sum_{i=1}^{n} x_i          % Summation
\prod_{i=1}^{n} x_i         % Product
\int_{a}^{b} f(x)\,dx       % Integral (\, for thin space before dx)
\iint \iiint                 % Double, triple integrals
\oint                        % Contour integral
\lim_{x \to \infty} f(x)    % Limit
\max_{x \in S} f(x)         % Maximum
```

### Decorations
```latex
\hat{x}     \bar{x}     \vec{x}     \dot{x}     \ddot{x}
\tilde{x}   \widetilde{xyz}   \widehat{xyz}
\overline{abc}    \underline{abc}
\overbrace{abc}^{\text{label}}    \underbrace{abc}_{\text{label}}
```

## Common Constructs

```latex
% Fractions
\frac{a}{b}          % Display fraction
\tfrac{a}{b}         % Text-style fraction (inline)
\dfrac{a}{b}         % Display-style fraction (force large)

% Binomial coefficient
\binom{n}{k}

% Roots
\sqrt{x}      \sqrt[3]{x}

% Text in math
\text{if } x > 0    % Roman text
\mathrm{Var}(X)      % Roman math
\operatorname{argmax}  % Proper operator spacing

% Custom operators (preamble)
\DeclareMathOperator{\argmin}{arg\,min}

% Spacing
\, (thin)  \: (medium)  \; (thick)  \quad  \qquad  \! (negative thin)
```

## Numbering and References

```latex
\begin{equation} \label{eq:euler}
  e^{i\pi} + 1 = 0
\end{equation}
See Equation~\ref{eq:euler}          % Basic reference
See \eqref{eq:euler}                 % With parentheses: (1)
% With cleveref: \cref{eq:euler}     % "Eq. (1)" auto-formatted
```

## Tips

- Use `\,` before `dx` in integrals: `\int f(x)\,dx`
- Use `\left( ... \right)` for auto-sized delimiters
- Use `\bigl( ... \bigr)` for manually sized delimiters
- Define repeated notation: `\newcommand{\R}{\mathbb{R}}`
- Use `\phantom{x}` for invisible spacing
- Use `\tag{*}` to replace equation numbers with custom labels
