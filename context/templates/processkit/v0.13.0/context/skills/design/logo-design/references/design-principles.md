# Logo Design Principles Reference

## The Five Principles of Effective Logo Design

### 1. Simple
- Can be described in one sentence
- Uses 2-3 colors maximum
- Relies on basic geometric forms
- Recognizable at a glance (< 2 seconds)
- Test: Can you draw it from memory?

### 2. Memorable
- One distinctive element that anchors recognition
- Avoid generic symbols (globes, swooshes, abstract circles)
- Negative space tricks create lasting impressions (FedEx arrow, NBC peacock)
- Unique enough to avoid confusion with competitors

### 3. Timeless
- Avoid trends: flat design trends of today are the dated looks of tomorrow
- Classic logos survive decades (Apple, Nike, Mercedes)
- If you must follow a trend, isolate it so it can be updated without redesigning
- Test: Would this look dated in 10 years?

### 4. Versatile
Must work in all these contexts without modification:
- Large format: billboards, banners
- Small format: favicon (16x16), app icon (1024x1024)
- Color: full color on white, on dark, on colored backgrounds
- Monochrome: single color (black, white, or any brand color)
- Print: CMYK, embossing, engraving, single-color printing

### 5. Appropriate
- Style matches audience expectations (playful vs. professional)
- Color psychology aligns with brand values
- Does not unintentionally resemble other brands or inappropriate imagery
- Works across cultures if the brand is international

## Color Psychology Quick Reference

| Color | Associations | Industries |
|---|---|---|
| Blue | Trust, stability, technology | Finance, tech, healthcare |
| Red | Energy, urgency, passion | Food, entertainment, sports |
| Green | Growth, nature, health | Organic, finance, environment |
| Yellow | Optimism, warmth, caution | Retail, food, children |
| Purple | Luxury, creativity, wisdom | Beauty, education, luxury goods |
| Orange | Friendliness, confidence | Tech, food, youth brands |
| Black | Sophistication, power | Luxury, fashion, tech |
| White | Clean, minimal, pure | Tech, healthcare, luxury |

## SVG Optimization Checklist

1. Remove editor metadata (Inkscape, Illustrator comments)
2. Minimize decimal precision (2 decimal places usually sufficient)
3. Use relative path commands where shorter (`l` vs `L`)
4. Combine overlapping shapes where possible
5. Remove hidden elements and unused definitions
6. Optimize with SVGO: `svgo --multipass input.svg -o output.svg`
7. Verify rendering after optimization

## Favicon Generation Sizes

| Size | Purpose |
|---|---|
| 16x16 | Browser tab (legacy) |
| 32x32 | Browser tab (standard) |
| 48x48 | Windows taskbar |
| 180x180 | Apple Touch Icon |
| 192x192 | Android Chrome |
| 512x512 | Android Chrome splash |

Modern approach: Use a single SVG favicon where supported:
```html
<link rel="icon" type="image/svg+xml" href="/favicon.svg">
<link rel="icon" type="image/png" sizes="32x32" href="/favicon-32.png">
```

## Responsive Logo System

A complete logo system includes multiple lockups for different contexts:

| Lockup | Description | Where Used |
|---|---|---|
| **Primary** | Full mark + wordmark | Website header, letterhead |
| **Horizontal** | Mark left, text right | Navigation bars, email signatures |
| **Stacked** | Mark above text | Social media, documents |
| **Icon only** | Mark without text | App icon, favicons, small spaces |
| **Wordmark only** | Text without mark | When the mark is already visible |

Design the mark and wordmark as independent `<g>` groups in one SVG, then
extract each lockup as a separate file.

## Logo File Deliverables

| Format | Purpose |
|---|---|
| SVG | Web, scalable, primary format |
| PNG (transparent) | Presentations, documents |
| PNG (with background) | Social media profiles |
| ICO | Windows favicon |
| PDF | Print-ready vector |
