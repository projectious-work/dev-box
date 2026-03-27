# Brand & Design Skill Category Research

**Date:** 2026-03-26
**Status:** Research complete, ready for decision
**Related:** BACK-039 (visual identity), skills-gap-analysis-2026-03.md, visual-identity-brief-2026-03.md

---

## 1. Problem Statement

aibox has 85 skills across 14 categories. The "Design & Visual" category contains 7 skills (excalidraw, logo-design, infographics, frontend-design, tailwind, pixijs-gamedev, mobile-app-design) but none address **brand identity as a system** — creating and maintaining a coherent brand package with design tokens, color systems, typography, voice guidelines, and usage rules.

The owner is building a brand package repository at `github.com/projectious-work/brand` (currently empty). A brand-package skill would both serve aibox users generally and provide a model implementation via that repo.

Meanwhile, the ecosystem has noticed this gap. Anthropic's own skills repository (`anthropics/skills`) now ships a `brand-guidelines` skill. ComposioHQ's awesome-claude-skills collection includes one. Marie Claire Dean's 63-skill design collection covers design systems and brand auditing. Claude.com has a dedicated resource page titled "Package your brand guidelines in a skill." None of these are comprehensive enough for aibox's curated, reference-backed approach.

---

## 2. What Goes in a Brand Package

Based on industry best practices, the W3C Design Tokens specification (stable since October 2025), and analysis of open-source brand repos (dotnet/brand, base/brand-kit, GitHub Brand Toolkit, Carbon Design System):

### 2.1 Core Components

| Component | Contents | Formats |
|---|---|---|
| **Design tokens** | Colors, typography, spacing, border radius, shadows, breakpoints, motion | W3C DTCG JSON (source of truth), CSS custom properties, Tailwind `@theme`, SCSS variables, JS/TS module |
| **Logo system** | Primary, horizontal, stacked, icon-only, wordmark-only, monochrome, reversed | SVG (canonical), PNG at standard sizes (32, 180, 192, 512, 1024), ICO for favicon |
| **Color system** | Primary, secondary, accent, neutrals (10-shade scale), semantic (success, warning, error, info), dark/light mode mappings | Defined as tokens; OKLCH for perceptual uniformity, with HEX/RGB fallbacks |
| **Typography** | Heading + body + code font families, size scale, weight scale, line heights, letter spacing | Font files (WOFF2), composite design tokens |
| **Brand voice** | Tone attributes, writing principles, vocabulary (preferred/avoided), example copy | Markdown guidelines |
| **Usage guidelines** | Logo clear space, minimum sizes, color do's/don'ts, dark mode rules, co-branding | Markdown with inline SVG examples |
| **Templates** | Social cards (OG 1200x630), presentation slides, letterhead, README badges | SVG templates, HTML/CSS |

### 2.2 Repository Structure (Recommended for projectious-work/brand)

```
brand/
  README.md                    # Overview, quick-start, links
  tokens/
    brand.tokens.json          # W3C DTCG format (source of truth)
    css/
      tokens.css               # CSS custom properties (generated)
      dark.css                 # Dark mode overrides (generated)
    tailwind/
      theme.css                # Tailwind @theme block (generated)
    scss/
      _tokens.scss             # SCSS variables (generated)
    js/
      tokens.mjs               # JS module export (generated)
    style-dictionary.config.js # Build config: JSON -> all formats
  logos/
    primary.svg                # Full logo (mark + wordmark)
    horizontal.svg             # Mark left, wordmark right
    stacked.svg                # Mark above wordmark
    icon.svg                   # Mark only (square)
    wordmark.svg               # Text only
    monochrome.svg             # Single-color variant
    reversed.svg               # Light-on-dark variant
    favicon.svg                # Optimized for small sizes
    social-card.svg            # 1200x630 OG template
    exports/                   # Generated PNG/ICO
      favicon-32.png
      favicon-180.png
      icon-192.png
      icon-512.png
      favicon.ico
  fonts/
    README.md                  # Font selection rationale, licensing
    *.woff2                    # Web font files
  guidelines/
    voice-and-tone.md          # Writing style, vocabulary
    logo-usage.md              # Clear space, minimum sizes, backgrounds
    color-usage.md             # Accessibility, dark mode, semantic mapping
    typography-usage.md        # Scale usage, hierarchy, code blocks
  templates/
    og-card.html               # Social card HTML template
    presentation.html          # Slide deck template
    badge.svg                  # GitHub README badge
  CHANGELOG.md                 # Brand evolution log
```

### 2.3 Design Token Format (W3C DTCG 2025.10)

The W3C Design Tokens Community Group published the first stable specification in October 2025. This is now the standard format, with support from Style Dictionary, Figma, Tokens Studio, and all major design tools.

```json
{
  "color": {
    "primary": {
      "$value": "oklch(0.65 0.19 260)",
      "$type": "color",
      "$description": "Primary brand color"
    },
    "on-primary": {
      "$value": "oklch(0.99 0 0)",
      "$type": "color",
      "$description": "Text/icon color on primary surfaces"
    }
  },
  "font": {
    "heading": {
      "$value": "Inter, system-ui, sans-serif",
      "$type": "fontFamily"
    },
    "code": {
      "$value": "JetBrains Mono, monospace",
      "$type": "fontFamily"
    }
  },
  "spacing": {
    "xs": { "$value": "0.25rem", "$type": "dimension" },
    "sm": { "$value": "0.5rem", "$type": "dimension" },
    "md": { "$value": "1rem", "$type": "dimension" },
    "lg": { "$value": "1.5rem", "$type": "dimension" },
    "xl": { "$value": "2rem", "$type": "dimension" }
  }
}
```

The token pipeline: **DTCG JSON** (source) -> **Style Dictionary** (transform) -> **CSS / Tailwind / SCSS / JS** (outputs).

---

## 3. Existing AI Brand/Design Tools

### 3.1 AI Brand Generation Platforms

| Tool | Capabilities | Output Quality | Relevance to aibox |
|---|---|---|---|
| **Brandmark.io** | AI logo + business card + social graphics | Raster-focused, basic | Low — visual-only, no tokens |
| **Looka** | Logo + brand kit (colors, fonts, templates) | Good for non-designers | Medium — generates starting assets |
| **uBrand** | Logo + mockups + brand videos + color cards | Comprehensive but GUI | Low — not agent-compatible |
| **Ironov.ai** | Neural network logo generation | Creative but unpredictable | Low — image-only |
| **Ad Legends Brand Kit** | Extract brand identity from any website; export CSS/JSON/Shadcn | High for extraction | High — token extraction useful |
| **Figma AI Brand Guidelines** | Generate guidelines from Figma libraries | Excellent for Figma users | Medium — depends on Figma |

### 3.2 Design Token Tooling

| Tool | Purpose | Status |
|---|---|---|
| **Style Dictionary** (Amazon) | Transform DTCG JSON tokens to any platform format | Mature, DTCG-native since v4 |
| **Tokens Studio** (Figma plugin) | Visual token management, DTCG export | Production, W3C compliant |
| **sd-tailwindcss-transformer** | Style Dictionary plugin for Tailwind config generation | Stable, community maintained |
| **Penpot** | Open-source design tool with token support | Growing, DTCG-aligned |

### 3.3 Existing Agent Skills for Brand Work

| Skill / Source | Scope | Gap vs. aibox needs |
|---|---|---|
| **anthropics/skills brand-guidelines** | Generic brand guidelines packaging | No design tokens, no generation, no audit |
| **ComposioHQ brand-guidelines** | Anthropic-specific brand colors/typography | Company-specific, not generalizable |
| **Marie Claire Dean's design collection** | 63 skills including design systems, brand auditing | Broad but shallow; no token pipeline |
| **claudeskills.org brand-guidelines** | Brand voice and visual consistency | Voice-focused, weak on technical implementation |
| **Antigravity UI/UX skills** | UI component design, not brand identity | Different scope |

**Key gap:** No existing agent skill covers the full brand package lifecycle — creation of design tokens in W3C DTCG format, generation of multi-format outputs (CSS/Tailwind/SCSS/JS), logo variant management, and brand consistency auditing. This is the opportunity for aibox.

---

## 4. Proposed Skills

### 4.1 brand-package (Priority: High)

**Purpose:** Create and maintain a complete brand identity package with machine-readable design tokens and human-readable guidelines.

**When to use:** When the user asks to create a brand identity, set up design tokens, define a color system, establish typography, create brand guidelines, or scaffold a brand repository.

**Key capabilities:**
- Scaffold a brand repository with the structure from section 2.2
- Define design tokens in W3C DTCG JSON format (colors, typography, spacing, shadows, breakpoints, motion)
- Generate multi-format token outputs: CSS custom properties, Tailwind `@theme`, SCSS variables, JS/TS modules
- Configure Style Dictionary build pipeline (JSON source -> all outputs)
- Define color systems: primary/secondary/accent/neutral scales, semantic mappings, dark/light mode
- Select and configure typography: heading + body + code fonts, size/weight/line-height scales
- Write brand voice guidelines from user input (tone attributes, vocabulary, examples)
- Write usage guidelines (logo clear space, color do's/don'ts, accessibility requirements)
- Generate social card and badge SVG templates
- Reference `github.com/projectious-work/brand` as the model implementation

**Tools:** `Bash(npm:*) Bash(npx:*) Read Write`

**References:**
- `references/dtcg-token-format.md` — W3C DTCG JSON schema, token types, composite tokens, theming
- `references/brand-repo-structure.md` — Repository layout, file naming, build pipeline setup
- `references/color-system-guide.md` — OKLCH color space, contrast requirements, dark mode, semantic mapping

**Relationship to existing skills:**
- Extends `logo-design` (brand-package references logo variants; logo-design handles SVG construction)
- Feeds into `tailwind` (brand-package generates `@theme` tokens that tailwind skill consumes)
- Feeds into `frontend-design` (design tokens drive component theming)

### 4.2 style-guide (Priority: Medium)

**Purpose:** Create living style guides — browsable HTML/CSS documentation of a brand package, with rendered component examples and copy-paste code snippets.

**When to use:** When the user asks to create a style guide, document their design system visually, or generate a component showcase from their brand tokens.

**Key capabilities:**
- Generate a static HTML style guide from design tokens (color swatches, typography specimens, spacing scale)
- Render logo variants at multiple sizes with background previews
- Show color accessibility matrix (contrast ratios for all text/background combinations)
- Generate component examples: buttons, cards, forms, navigation, with token-driven theming
- Produce both light and dark mode previews side by side
- Export as standalone HTML (no build step) or as a page within an existing docs site

**Tools:** `Bash(npm:*) Bash(npx:*) Read Write`

**References:**
- `references/style-guide-patterns.md` — Page structure, section templates, accessibility matrix generation

**Relationship to existing skills:**
- Consumes output of `brand-package` (reads design tokens)
- Complements `documentation` skill (style guide is a specific documentation type)

### 4.3 design-system (Priority: Medium)

**Purpose:** Design component libraries using atomic design principles, token-driven theming, and accessibility-first patterns.

**When to use:** When the user asks to create a component library, design a UI kit, implement theming across components, or set up a design system with tokens.

**Key capabilities:**
- Apply atomic design: atoms (button, input) -> molecules (search bar) -> organisms (header) -> templates -> pages
- Wire design tokens to components via CSS custom properties (not hardcoded values)
- Implement multi-theme support: brand themes via token swap, dark/light mode, high-contrast mode
- Define component API surfaces: props, variants, slots, compound components
- Ensure accessibility in every component: keyboard navigation, ARIA, focus management, contrast
- Document component states: default, hover, focus, active, disabled, loading, error
- Set up component testing: visual regression, accessibility audits, interaction testing

**Tools:** `Bash(npm:*) Bash(npx:*) Read Write`

**References:**
- `references/atomic-design-reference.md` — Atomic levels, composition patterns, token-to-component wiring
- `references/component-api-patterns.md` — Props design, variant patterns, accessibility requirements per component

**Relationship to existing skills:**
- Builds on `brand-package` (consumes design tokens) and `frontend-design` (component architecture)
- Extends `tailwind` (Tailwind is one implementation path for token-driven components)

### 4.4 visual-identity-audit (Priority: Low-Medium)

**Purpose:** Review existing brand assets for consistency, accessibility, and completeness.

**When to use:** When the user asks to audit their brand, check for consistency issues, verify accessibility compliance, or identify missing brand assets.

**Key capabilities:**
- Check token completeness: all required categories present (color, typography, spacing, shadow, breakpoint)
- Verify color accessibility: WCAG AA/AAA contrast ratios for all text/background combinations
- Verify color accessibility: simulate deuteranopia, protanopia, tritanopia on color palette
- Check logo variants: all required lockups present, render at 16/32/180/512px, test on light/dark/colored backgrounds
- Check dark mode: all semantic tokens have dark mode mappings, contrast ratios pass in both modes
- Check typography: font files present, fallback stack defined, size scale is consistent
- Detect inconsistencies: hardcoded colors that should be tokens, mismatched font stacks, orphaned variables
- Generate audit report with pass/fail/warning per criterion and remediation suggestions

**Tools:** `Bash(npm:*) Bash(npx:*) Read Write`

**References:**
- `references/audit-checklist.md` — Complete checklist: tokens, colors, logos, typography, voice, templates

**Relationship to existing skills:**
- Audits output of `brand-package`, `style-guide`, `design-system`
- Extends accessibility checks from `frontend-design`

---

## 5. Category Placement

### 5.1 Current "Design & Visual" Category Analysis

The existing category mixes three distinct concerns:

| Concern | Skills | Notes |
|---|---|---|
| **Brand & Identity** | logo-design | Creating brand assets |
| **UI/Frontend** | frontend-design, tailwind, mobile-app-design | Building user interfaces |
| **Creative/Visual** | excalidraw, infographics, pixijs-gamedev | Diagrams, charts, games |

Adding 4 more brand skills to this 7-skill category would tip it to 11 skills — the largest category — with an even more confused identity.

### 5.2 Options

**Option A: Split into two categories**

| Category | Skills (new in bold) |
|---|---|
| **Brand & Design System** (new) | logo-design, **brand-package**, **style-guide**, **design-system**, **visual-identity-audit** |
| **UI & Visual** (renamed) | frontend-design, tailwind, mobile-app-design, excalidraw, infographics, pixijs-gamedev |

- Pros: Clean separation of concerns. Brand skills cluster naturally. UI skills remain grouped.
- Cons: logo-design moves away from its current neighbors. Two smaller categories instead of one medium.

**Option B: Keep one category, add subcategory headers**

The docs page (`docs-site/docs/skills/design.md`) already lists skills with `###` headings. Add a visual grouping:

```
# Design & Visual Skills

## Brand & Identity
- brand-package
- logo-design
- style-guide
- design-system
- visual-identity-audit

## UI & Frontend
- frontend-design
- tailwind
- mobile-app-design

## Creative & Diagrams
- excalidraw
- infographics
- pixijs-gamedev
```

- Pros: No structural change to the category system. Easy to implement.
- Cons: Category grows to 12 skills. Subcategories are cosmetic (docs-only), not reflected in the skill system itself.

**Option C: Create a new "Brand" category**

| Category | Skills |
|---|---|
| **Brand** (new) | **brand-package**, logo-design, **style-guide**, **design-system**, **visual-identity-audit** |
| **Design & Visual** (unchanged minus logo-design) | frontend-design, tailwind, mobile-app-design, excalidraw, infographics, pixijs-gamedev |

- Pros: Dedicated category signals brand work is a first-class concern. Clean.
- Cons: "Brand" is a narrow category name for 5 skills. May feel over-specialized vs. other categories.

### 5.3 Recommendation: Option A (Split into two categories)

**"Brand & Design System"** and **"UI & Visual"** are the clearest split. The brand skills form a natural pipeline (brand-package -> style-guide -> design-system -> visual-identity-audit), while the UI skills are about building interfaces. logo-design belongs with brand work because logos are brand assets, not UI components.

This brings the category count from 14 to 15 and keeps both categories at 5-6 skills — consistent with other categories (Architecture: 4, API: 4, Security: 5, Observability: 4).

---

## 6. Relationship to projectious-work/brand Repository

The `brand-package` skill should reference `github.com/projectious-work/brand` in two ways:

1. **As a model implementation.** The skill's reference material should include a "Model Repository" section pointing to the repo as a working example of the recommended structure and token format. This gives users a concrete reference rather than abstract guidelines.

2. **As the aibox brand source of truth.** The skill can be used by the owner to populate the brand repo itself — using the skill to scaffold the structure, define tokens based on the visual identity brief (BACK-039 research), and generate all output formats.

The brand repo should contain a `CLAUDE.md` that references the `brand-package` skill, creating a feedback loop: the skill teaches agents how to build brand packages; the repo demonstrates what a completed package looks like; agents working in the repo use the skill to maintain it.

---

## 7. Implementation Roadmap

### Phase 1: brand-package skill (immediate)

1. Create `templates/skills/brand-package/SKILL.md`
2. Create reference files:
   - `references/dtcg-token-format.md` — DTCG JSON schema, token types, Style Dictionary config
   - `references/brand-repo-structure.md` — Recommended directory layout, file inventory
   - `references/color-system-guide.md` — OKLCH, contrast math, dark mode, semantic mapping
3. Populate `github.com/projectious-work/brand` using the skill (dogfooding)
4. Add to docs-site

### Phase 2: visual-identity-audit skill (near-term)

1. Create skill with audit checklist reference
2. Run audit against the projectious-work/brand repo (dogfooding)
3. This validates both the skill and the brand package

### Phase 3: style-guide + design-system skills (medium-term)

1. Create both skills with reference materials
2. These build on a populated brand repo and are less urgent until brand-package exists

### Phase 4: Category restructure

1. Split "Design & Visual" into "Brand & Design System" + "UI & Visual"
2. Update docs-site sidebar and category pages
3. Update addon-skill mapping (these skills likely belong in a future `brand` process package or remain as manual-include skills)

---

## 8. Process Package / Addon Mapping

These skills are not tied to a specific programming language or tool, so they do not naturally map to an addon. Options:

| Mapping | Skills | Notes |
|---|---|---|
| **New "brand" process package** | brand-package, style-guide, design-system, visual-identity-audit | Auto-deployed when user selects brand work. Includes logo-design. |
| **Standalone (manual include)** | All four | Users add via `[skills] include = ["brand-package", ...]`. Simpler but less discoverable. |
| **docs-docusaurus addon** | style-guide | If the user has Docusaurus, a style guide page is natural. |

**Recommendation:** Start as standalone skills (manual include). Once all four exist and are validated, create a "brand" process package that bundles them with logo-design. This avoids premature packaging.

---

## Sources

- [W3C Design Tokens Specification 2025.10](https://www.designtokens.org/tr/drafts/format/)
- [Design Tokens Community Group — First Stable Version Announcement](https://www.w3.org/community/design-tokens/2025/10/28/design-tokens-specification-reaches-first-stable-version/)
- [Style Dictionary — DTCG Support](https://styledictionary.com/info/dtcg/)
- [Design Tokens: Foundation of Scalable Design Systems (Medium, 2026)](https://medium.com/design-bootcamp/design-tokens-b880c9d78579)
- [Design Systems & Design Tokens Complete Guide (design.dev)](https://design.dev/guides/design-systems/)
- [A Complete Visual Branding Guide for 2026](https://www.wearetenet.com/blog/visual-branding)
- [9 New Design System Examples to Scale Brands in 2026](https://www.superside.com/blog/design-systems-examples)
- [Figma AI Brand Guidelines Generator](https://www.figma.com/solutions/ai-brand-guideline-generator/)
- [5 Best Brand Guidelines Generators for Designers (2026)](https://www.akrivi.io/learn/best-brand-guidelines-generator)
- [Package Your Brand Guidelines in a Skill (Claude)](https://claude.com/resources/use-cases/package-your-brand-guidelines-in-a-skill)
- [48 Design Skills for Claude and Other AI Coding Agents (DEV Community)](https://dev.to/zoltanszogyenyi/48-design-skills-for-claude-and-other-ai-coding-agents-10a8)
- [I Built 63 Design Skills For Claude (Marie Claire Dean)](https://marieclairedean.substack.com/p/i-built-63-design-skills-for-claude)
- [Claude Skills 2.0 for Product Designers (UX Planet)](https://uxplanet.org/claude-skills-2-0-for-product-designers-a86f4518b3ba)
- [anthropics/skills brand-guidelines](https://github.com/anthropics/skills/tree/main/skills/brand-guidelines)
- [ComposioHQ/awesome-claude-skills brand-guidelines](https://github.com/ComposioHQ/awesome-claude-skills/blob/master/brand-guidelines/SKILL.md)
- [dotnet/brand Repository](https://github.com/dotnet/brand)
- [base/brand-kit Repository](https://github.com/base/brand-kit)
- [GitHub Brand Toolkit](https://brand.github.com/)
- [Primer Design System (GitHub)](https://primer.style/)
- [Carbon Design System (IBM)](https://github.com/carbon-design-system)
- [Style Dictionary Tailwind CSS Transformer](https://github.com/nado1001/style-dictionary-tailwindcss-transformer)
- [How to Structure a Branding Package (File Architect)](https://filearchitect.com/blueprints/design-creative/branding-package)
- [Ad Legends AI Brand Kit Generator](https://www.adlegends.ai/ai-brand-kit-generator)
- [Brandmark.io](https://brandmark.io/)
- [Figma Canvas Open to Agents (MCP)](https://www.figma.com/blog/the-figma-canvas-is-now-open-to-agents/)
- [Penpot Design Tokens Guide](https://penpot.app/blog/design-tokens-for-designers/)
