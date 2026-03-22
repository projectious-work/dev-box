---
name: logo-design
description: Creates SVG logos with proper scalability, color theory, and variant generation. Use when designing logos, icons, or brand marks.
---

# Logo Design

## When to Use

When the user asks to:
- Create or design a logo or brand mark
- Generate a favicon or social media avatar from a logo
- Review a logo for scalability or design issues
- Apply color theory to branding
- Create logo variants (monochrome, dark mode, small sizes)

## Instructions

### 1. Logo Design Principles

- **Simplicity**: A logo should be recognizable at 16x16 pixels (favicon size)
- **Memorability**: One distinctive element is better than many competing ones
- **Timelessness**: Avoid trendy effects (gradients, shadows) that date quickly
- **Versatility**: Must work in color, monochrome, reversed, and at all sizes
- **Appropriateness**: Style should match the brand's personality and audience

### 2. SVG Logo Construction

- Use clean geometric shapes (`<circle>`, `<rect>`, `<path>`)
- Set `viewBox` to define the design space (e.g., `viewBox="0 0 100 100"`)
- Omit `width` and `height` for fluid scaling; or set both for fixed sizing
- Optimize paths: minimize control points, remove unnecessary precision
- Use `<g>` groups with meaningful IDs (`id="mark"`, `id="wordmark"`)
- Center the design in the viewBox; leave breathing room at edges

```xml
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
  <g id="mark">
    <!-- Logo mark paths -->
  </g>
  <g id="wordmark">
    <!-- Text elements (convert to paths for distribution) -->
  </g>
</svg>
```

### 3. Color Theory Basics

| Scheme | Description | Use Case |
|---|---|---|
| **Monochromatic** | Shades of one hue | Elegant, cohesive |
| **Complementary** | Opposite on color wheel | High contrast, energetic |
| **Analogous** | Adjacent hues | Harmonious, natural |
| **Triadic** | Three equally spaced hues | Vibrant, balanced |
| **Split-complementary** | Base + two adjacent to complement | Contrast with less tension |

Guidelines:
- Limit to 2-3 colors maximum
- Ensure 4.5:1 contrast ratio against both white and dark backgrounds
- Choose colors that reproduce well in CMYK (print) and RGB (screen)
- Test for colorblind accessibility (avoid red/green as only differentiator)

### 4. Typography in Logos

- Choose fonts that reflect brand personality (geometric = modern, serif = traditional)
- Modify letterforms for uniqueness (custom ligatures, adjusted spacing)
- Convert text to paths (`<path>`) for distribution (prevents font dependency)
- Maintain consistent optical spacing (visual, not mathematical kerning)
- Ensure readability at small sizes: avoid thin strokes, tight spacing

### 5. Logo Variants to Generate

When creating a logo, produce these variants:

| Variant | Size/Format | Use Case |
|---|---|---|
| **Full logo** | SVG, any size | Website header, documents |
| **Icon/mark only** | SVG, square | App icon, social avatar |
| **Favicon** | 32x32, 16x16 | Browser tab |
| **Monochrome** | SVG, single color | Printing, watermarks |
| **Reversed** | SVG, light on dark | Dark backgrounds |
| **Social banner** | 1200x630 | Open Graph / social sharing |

### 6. Testing and Validation

- Render at 16px, 32px, 180px, and 512px — verify recognizability at each size
- Test on white, black, and colored backgrounds
- Convert to grayscale and check that the mark remains distinct
- Simulate deuteranopia and protanopia to verify colorblind safety
- Print a black-and-white test page to check single-color reproduction
- Compare side-by-side with competitors to confirm distinctiveness

### 7. Common Mistakes

- Too many details that disappear at small sizes
- Text that is not converted to paths (font dependency)
- Colors that clash with common UI backgrounds
- Overly complex paths that bloat file size
- No monochrome variant (needed for single-color printing)
- Raster effects (drop shadows, blur) embedded in SVG
- Using trendy gradients or shadows that date the logo quickly
- Forgetting the reversed (light-on-dark) variant for dark mode

## References

- `references/design-principles.md` --- Core design principles for brand identity

## Examples

**User:** "Create a logo for my CLI tool called 'flux'"
**Agent:** Designs a geometric mark suggesting flow/movement, pairs it with a clean
sans-serif wordmark. Generates SVG with full logo, icon-only, monochrome, and
reversed variants. Explains color choices and scaling behavior.

**User:** "Make a favicon from this logo"
**Agent:** Simplifies the mark to essential shapes, tests at 16x16 and 32x32,
ensures recognizability. Provides SVG optimized for small sizes.
