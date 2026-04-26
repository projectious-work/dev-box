---
name: mobile-app-design
description: |
  Mobile app UX — touch targets, navigation, platform conventions, accessibility. Use when designing mobile interfaces, reviewing mobile UX, adapting web designs for mobile, or reconciling iOS and Android conventions.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-mobile-app-design
    version: "1.1.0"
    created: 2026-04-06T00:00:00Z
    category: design
---

# Mobile App Design

## Intro

Mobile UX is shaped by finger-sized touch targets, one-thumb reach,
platform conventions, and flaky connectivity. Start with the smallest
supported screen, respect platform idioms for navigation and
components, and design for offline-first use from the beginning.

## Overview

### Touch target sizing

- Minimum 44x44 points (iOS) / 48x48 dp (Android) for all
  interactive elements.
- At least 8pt gap between targets to prevent mis-taps.
- Primary CTAs 48-56pt high.
- Thumbs reach the lower two-thirds of the screen easily — put
  frequent actions there.
- Keep destructive actions away from frequent-tap zones.
- Floating action buttons: 56dp standard, 40dp mini.

### Navigation patterns

- **Tab bar (bottom):** 3-5 top-level destinations, always visible,
  best for peer sections.
- **Navigation drawer (hamburger):** 5+ destinations or infrequent
  sections.
- **Stack navigation:** push/pop for hierarchical detail screens.
- **Modal sheets:** temporary tasks, filters, confirmations —
  dismiss with swipe or button.

Keep depth shallow (3 levels max). Show a back button or
swipe-to-go-back gesture on every non-root screen. Persistent bottom
nav should highlight the active tab and not reset tab state on
re-tap.

### Responsive layouts

Design for the smallest supported screen first (375pt width for iOS,
360dp for Android). Use a 4pt or 8pt spacing grid. Support landscape
unless the content is strongly portrait-oriented. Adapt to tablets
with split-view or increased content width via max-width constraints.
Support Dynamic Type (iOS) and font scaling (Android) — never
hardcode font sizes — and test with the system font set to its
maximum size to catch overflow.

### iOS vs Android conventions

- **Back navigation:** iOS swipe-from-edge or top-left button;
  Android system back button or gesture.
- **Tabs:** iOS bottom tab bar; Android bottom navigation bar or top
  tabs.
- **Buttons:** iOS plain text or rounded rect; Android filled or
  outlined Material buttons.
- **Alerts:** iOS centered alerts with stacked buttons; Android
  dialogs with side-by-side buttons.
- **Typography:** iOS uses SF Pro; Android uses Roboto or system
  font.
- **Icons:** iOS SF Symbols (outline by default); Android Material
  Icons (filled or outlined).

When in doubt, follow platform conventions — users expect familiar
behavior.

### Gesture patterns

- **Tap:** primary interaction — feedback under 100ms.
- **Long press:** secondary actions, context menus — always provide
  a visible alternative.
- **Swipe:** list actions (delete, archive), sheet dismissal, page
  navigation.
- **Pull to refresh:** standard for list content — show a spinner.
- **Pinch to zoom:** images, maps — provide zoom controls as an
  alternative.

Make swipe actions discoverable with a peek animation on first use.
Never make gestures the only way to perform an action.

### Accessibility

Support screen readers with labels on every interactive element and
image. Maintain 4.5:1 text contrast, 3:1 UI contrast. Support Dynamic
Type and font scaling without layout breakage. Provide captions or
transcripts for audio/video. Test with VoiceOver (iOS) and TalkBack
(Android). Ensure touch targets stay accessible at all accessibility
zoom levels. Respect reduced-motion preferences with static
alternatives.

### Offline-first design

- Show cached content immediately while fetching updates in the
  background.
- Display clear "offline" indicators — never silently fail.
- Queue user actions when offline and sync when reconnected.
- Use optimistic UI: show the result immediately, reconcile later.
- Store critical data locally — users expect apps to work without
  signal.
- Design conflict resolution for collaborative data.

### Push notifications

- Request permission at a contextually relevant moment, not on first
  launch.
- Explain the value before asking ("Get notified when your order
  ships").
- Group related notifications to avoid overwhelming the user.
- Deep link from notifications directly to the relevant content.
- Respect notification settings — provide in-app controls to
  customize.
- Never use notifications for marketing on first install — earn
  trust first.

### Onboarding flows

- Keep onboarding to 3-5 screens maximum — users want to use the
  app.
- Show value immediately — let users explore before requiring
  sign-up.
- Progressive disclosure: teach features as users encounter them.
- Skip onboarding for returning users by checking for existing data.
- Provide a "Skip" option on every onboarding screen.
- Prefer contextual tooltips and coach marks over upfront tutorials.

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Designing for the largest screen first and scaling down.** Starting with a wide screen allows fitting too much content, which breaks on the common 375pt iPhone viewport. Design for the smallest supported screen first, then progressively enhance for larger displays.
- **Using touch targets smaller than platform minimums.** Anything below 44×44 points (iOS) or 48×48 dp (Android) causes mis-taps and generates disproportionate accessibility complaints. Apply the minimums to every interactive element, including secondary and icon-only buttons.
- **Placing destructive actions in the high-frequency thumb zone.** The lower two-thirds of the screen is the easiest reach area. Putting "Delete" or "Log out" near frequently tapped actions causes accidental destructive interactions. Keep destructive actions away from high-frequency zones and require confirmation.
- **Requesting permissions on first launch.** Users who have not yet experienced the app's value deny permissions at high rates, and iOS/Android only allow one native prompt per permission. Request permissions at a contextually relevant moment after the user has seen the benefit.
- **Not testing with the system font at maximum scale.** Dynamic Type (iOS) and font scaling (Android) can triple the effective font size. Fixed-height containers, truncated labels, and overflowing text are only visible when tested at the maximum system accessibility font scale.
- **Designing for online-only without an offline state.** Users regularly use apps in low-signal environments. An app that silently fails or shows blank screens when offline creates broken experiences. Design the offline state explicitly: show cached content, queue writes, and display clear "offline" indicators.
- **Making gestures the only way to perform an action.** Swipe-to-delete or pull-to-refresh as the sole access method excludes users who cannot perform those gestures. Always pair gestures with a discoverable alternative — a button, long-press menu, or context action.

## Full reference

### iOS HIG vs Material Design — navigation

| Aspect | iOS (HIG) | Android (Material 3) |
|---|---|---|
| Primary nav | Bottom tab bar (UITabBarController) | Bottom navigation bar (NavigationBar) |
| Tab limit | 5 max (more goes to "More" tab) | 3-5 destinations |
| Secondary nav | Back button (top-left) | System back gesture/button |
| Drawer | Rare — prefer tabs or lists | Common for 5+ destinations |
| Stack | UINavigationController push/pop | Fragment/Activity back stack |
| Modals | Sheet sliding up, swipe to dismiss | Bottom sheet or dialog |

### Typography

| Element | iOS (SF Pro) | Android (Roboto / Material Type Scale) |
|---|---|---|
| Large title | 34pt bold | Display Large: 57sp |
| Title | 17pt semibold | Title Large: 22sp |
| Body | 17pt regular | Body Large: 16sp |
| Caption | 12pt regular | Body Small: 12sp |
| Line height | ~1.2x font size | ~1.25-1.5x font size |

Both platforms require support for dynamic font sizing — never
hardcode sizes.

### Iconography

| Aspect | iOS | Android |
|---|---|---|
| Icon set | SF Symbols (5000+ icons) | Material Symbols (3000+ icons) |
| Style | Outline by default, filled for active tab | Filled or outlined per Material theme |
| Size | 22-28pt for toolbar/tab icons | 24dp standard |
| Weight | Matches text weight (dynamic) | Fixed weight per variant |

### Spacing and layout

| Aspect | iOS | Android |
|---|---|---|
| Base grid | 8pt grid (some 4pt) | 8dp / 4dp grid |
| Content margins | 16pt standard, 20pt on larger devices | 16dp standard |
| List row height | 44pt minimum | 48dp minimum (one-line), 56dp (two-line) |
| Card padding | 16pt internal | 16dp internal |
| Safe areas | Required (notch, home indicator) | Edge-to-edge with system bar insets |

### Color

| Aspect | iOS | Android |
|---|---|---|
| System colors | Dynamic (adapt to light/dark) | Material color roles (primary, secondary, surface) |
| Accent color | App tint color | Primary color from seed |
| Dark mode | Elevated surfaces are lighter | Tonal elevation (surface color shifts) |
| Contrast | System provides high-contrast variants | Medium/High contrast themes available |
| Color scheme | System colors + custom via asset catalog | Dynamic color from wallpaper (Android 12+) |

### Components

| Component | iOS | Android |
|---|---|---|
| Primary button | Filled or plain text | Filled button (FilledButton) |
| Secondary button | Tinted or outline | Outlined button (OutlinedButton) |
| Toggle | UISwitch (green/gray) | Material Switch |
| Date picker | Wheel or compact popup | Calendar dialog |
| Alert | Centered, stacked buttons | Dialog, side-by-side buttons |
| Action menu | Action sheet (bottom) | Bottom sheet or popup menu |
| Search | Search bar in navigation | SearchBar widget |
| Pull to refresh | UIRefreshControl | SwipeRefreshLayout |

### When to follow platform vs cross-platform

Follow platform conventions when:

- The control has a direct platform equivalent (alerts, date
  pickers, sheets).
- The gesture is platform-specific (iOS swipe-back, Android system
  back).
- Users expect platform-consistent behavior (share sheets,
  notifications).
- Accessibility features depend on native patterns (VoiceOver,
  TalkBack).

Use cross-platform consistency when:

- The feature is unique to your app (custom visualizations, brand
  experience).
- The interaction has no strong platform equivalent.
- Maintaining two designs would create user confusion between
  platforms.
- Your brand identity requires visual consistency.

### Key principles

1. **Respect the platform** — users spend 99% of their time in other
   apps.
2. **Prioritize usability** — a familiar pattern beats a clever
   custom solution.
3. **Test on both platforms** — what feels natural on iOS may feel
   foreign on Android.
4. **Use adaptive layouts** — design once, adapt to platform
   conventions in code.
5. **Accessibility first** — both platforms provide excellent a11y
   tools; use them.

### Further reading

- `references/platform-guidelines.md` — source for the iOS vs
  Material comparison tables above.
- [Apple Human Interface Guidelines](https://developer.apple.com/design/human-interface-guidelines/)
- [Material Design 3](https://m3.material.io/)
