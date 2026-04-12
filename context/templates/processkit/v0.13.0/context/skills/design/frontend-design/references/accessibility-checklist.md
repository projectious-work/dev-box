# WCAG 2.2 AA Accessibility Checklist

Checklist for frontend developers targeting WCAG 2.2 Level AA compliance.
Criteria marked (A) are Level A; (AA) are Level AA. Both levels are required for AA conformance.

## 1. Perceivable

Content must be presentable to users in ways they can perceive.

### Text Alternatives
- [ ] **1.1.1 Non-text Content (A)** — All images, icons, and buttons have descriptive `alt` text. Decorative images use `alt=""` or CSS backgrounds.

### Time-based Media
- [ ] **1.2.1 Audio/Video Only (A)** — Provide transcripts for audio-only; transcripts or audio descriptions for video-only.
- [ ] **1.2.2 Captions Prerecorded (A)** — All prerecorded video with audio has synchronized captions.
- [ ] **1.2.3 Audio Description (A)** — Video content has audio description or a descriptive transcript.
- [ ] **1.2.4 Captions Live (AA)** — Live audio content has real-time captions.

### Adaptable
- [ ] **1.3.1 Info and Relationships (A)** — Use semantic HTML: headings (`h1`-`h6`), landmarks (`nav`, `main`), lists, table headers. Do not rely on visual styling alone.
- [ ] **1.3.2 Meaningful Sequence (A)** — DOM order matches visual reading order.
- [ ] **1.3.3 Sensory Characteristics (A)** — Instructions do not rely solely on shape, color, size, or location.
- [ ] **1.3.4 Orientation (AA)** — Content works in both portrait and landscape.
- [ ] **1.3.5 Identify Input Purpose (AA)** — Form fields use appropriate `autocomplete` attributes.

### Distinguishable
- [ ] **1.4.1 Use of Color (A)** — Color is not the only way to convey information (e.g., error states also have icons/text).
- [ ] **1.4.2 Audio Control (A)** — Auto-playing audio (>3s) can be paused or volume adjusted.
- [ ] **1.4.3 Contrast Minimum (AA)** — Normal text: 4.5:1 ratio. Large text (18pt+ or 14pt+ bold): 3:1 ratio.
- [ ] **1.4.4 Resize Text (AA)** — Page is readable and functional at 200% zoom.
- [ ] **1.4.5 Images of Text (AA)** — Use real text, not images of text.
- [ ] **1.4.10 Reflow (AA)** — No horizontal scrolling at 320px viewport width (1280px at 400% zoom).
- [ ] **1.4.11 Non-text Contrast (AA)** — UI components and graphical objects have 3:1 contrast ratio.
- [ ] **1.4.12 Text Spacing (AA)** — No content loss when users override line height (1.5x), paragraph spacing (2x), letter spacing (0.12em), word spacing (0.16em).
- [ ] **1.4.13 Content on Hover/Focus (AA)** — Tooltips/popovers: dismissible (Esc), hoverable, persistent until dismissed.

## 2. Operable

UI components and navigation must be operable.

### Keyboard Accessible
- [ ] **2.1.1 Keyboard (A)** — All functionality is available via keyboard.
- [ ] **2.1.2 No Keyboard Trap (A)** — Focus can always be moved away from any element.

### Enough Time
- [ ] **2.2.1 Timing Adjustable (A)** — Time limits can be turned off, adjusted, or extended.
- [ ] **2.2.2 Pause, Stop, Hide (A)** — Auto-moving content (>5s) can be paused; auto-updating content can be controlled.

### Seizures and Physical Reactions
- [ ] **2.3.1 Three Flashes (A)** — No content flashes more than 3 times per second.

### Navigable
- [ ] **2.4.1 Bypass Blocks (A)** — Provide "skip to main content" link.
- [ ] **2.4.2 Page Titled (A)** — Each page has a descriptive `<title>`.
- [ ] **2.4.3 Focus Order (A)** — Tab order is logical and intuitive.
- [ ] **2.4.4 Link Purpose (A)** — Link text describes the destination (avoid "click here").
- [ ] **2.4.5 Multiple Ways (AA)** — At least two ways to find pages (nav, search, sitemap).
- [ ] **2.4.6 Headings and Labels (AA)** — Headings and labels are descriptive.
- [ ] **2.4.7 Focus Visible (AA)** — Keyboard focus indicator is clearly visible.
- [ ] **2.4.11 Focus Not Obscured (AA)** — Focused elements are not completely hidden by sticky headers/footers/modals.

### Input Modalities
- [ ] **2.5.1 Pointer Gestures (A)** — Multi-finger or path-based gestures have single-pointer alternatives.
- [ ] **2.5.2 Pointer Cancellation (A)** — Actions fire on pointer up, not down; can be aborted.
- [ ] **2.5.3 Label in Name (A)** — Visible label text is contained in the accessible name.
- [ ] **2.5.4 Motion Actuation (A)** — Shake/tilt actions have button alternatives and can be disabled.
- [ ] **2.5.7 Dragging Movements (AA)** — Drag-and-drop has a non-dragging alternative (e.g., buttons).
- [ ] **2.5.8 Target Size Minimum (AA)** — Interactive targets are at least 24x24 CSS pixels.

## 3. Understandable

Information and UI operation must be understandable.

### Readable
- [ ] **3.1.1 Language of Page (A)** — `<html lang="en">` (or appropriate language code).
- [ ] **3.1.2 Language of Parts (AA)** — Content in other languages uses `lang` attribute.

### Predictable
- [ ] **3.2.1 On Focus (A)** — Focusing an element does not trigger unexpected changes.
- [ ] **3.2.2 On Input (A)** — Changing a form input does not cause unexpected navigation/submission.
- [ ] **3.2.3 Consistent Navigation (AA)** — Navigation order is consistent across pages.
- [ ] **3.2.4 Consistent Identification (AA)** — Same functionality uses same labels/icons site-wide.

### Input Assistance
- [ ] **3.3.1 Error Identification (A)** — Errors are clearly described in text (not color alone).
- [ ] **3.3.2 Labels or Instructions (A)** — Form inputs have visible labels or clear instructions.
- [ ] **3.3.3 Error Suggestion (AA)** — Error messages suggest how to fix the problem.
- [ ] **3.3.4 Error Prevention (AA)** — Reversible, verified, or confirmed submissions for legal/financial data.
- [ ] **3.3.7 Redundant Entry (A)** — Previously entered info is auto-populated in multi-step forms.
- [ ] **3.3.8 Accessible Authentication (AA)** — Login does not require cognitive tests; allow paste in password fields, support password managers.

## 4. Robust

Content must be robust enough for assistive technologies.

- [ ] **4.1.2 Name, Role, Value (A)** — Custom components expose correct name, role, and state via ARIA. Use native HTML elements when possible.
- [ ] **4.1.3 Status Messages (AA)** — Dynamic status messages (success, error, progress) announced via `aria-live` or `role="alert"`.

## Quick Implementation Guide

### HTML Structure
```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Descriptive Page Title</title>
</head>
<body>
  <a href="#main" class="sr-only focus:not-sr-only">Skip to main content</a>
  <header><nav aria-label="Main">...</nav></header>
  <main id="main">...</main>
  <footer>...</footer>
</body>
</html>
```

### Common ARIA Patterns
```html
<!-- Icon button -->
<button aria-label="Close dialog">
  <svg aria-hidden="true">...</svg>
</button>

<!-- Live region for dynamic updates -->
<div aria-live="polite" aria-atomic="true">
  3 items in cart
</div>

<!-- Form with error -->
<label for="email">Email</label>
<input id="email" type="email" aria-invalid="true" aria-describedby="email-error">
<p id="email-error" role="alert">Please enter a valid email address.</p>

<!-- Modal dialog -->
<dialog aria-labelledby="dialog-title">
  <h2 id="dialog-title">Confirm deletion</h2>
  ...
</dialog>
```

### Testing Tools
- **axe DevTools** — browser extension for automated checks
- **Lighthouse** — accessibility audit in Chrome DevTools
- **NVDA** (Windows) / **VoiceOver** (macOS) — screen reader testing
- **Keyboard-only navigation** — unplug the mouse and test every flow
- **Colour Contrast Analyser** — verify contrast ratios
- **WAVE** — web accessibility evaluation tool
