# iOS HIG vs Material Design Comparison

## Navigation

| Aspect | iOS (HIG) | Android (Material 3) |
|---|---|---|
| Primary nav | Bottom tab bar (UITabBarController) | Bottom navigation bar (NavigationBar) |
| Tab limit | 5 max (more goes to "More" tab) | 3-5 destinations |
| Secondary nav | Back button (top-left) | System back gesture/button |
| Drawer | Rare — prefer tabs or lists | Common for 5+ destinations |
| Stack | UINavigationController push/pop | Fragment/Activity back stack |
| Modals | Sheet sliding up, swipe to dismiss | Bottom sheet or dialog |

## Typography

| Element | iOS (SF Pro) | Android (Roboto / Material Type Scale) |
|---|---|---|
| Large title | 34pt bold | Display Large: 57sp |
| Title | 17pt semibold | Title Large: 22sp |
| Body | 17pt regular | Body Large: 16sp |
| Caption | 12pt regular | Body Small: 12sp |
| Line height | ~1.2x font size | ~1.25-1.5x font size |

Both platforms require support for dynamic font sizing — never hardcode sizes.

## Iconography

| Aspect | iOS | Android |
|---|---|---|
| Icon set | SF Symbols (5000+ icons) | Material Symbols (3000+ icons) |
| Style | Outline by default, filled for active tab | Filled or outlined per Material theme |
| Size | 22-28pt for toolbar/tab icons | 24dp standard |
| Weight | Matches text weight (dynamic) | Fixed weight per variant |

## Spacing and Layout

| Aspect | iOS | Android |
|---|---|---|
| Base grid | 8pt grid (some 4pt) | 8dp / 4dp grid |
| Content margins | 16pt standard, 20pt on larger devices | 16dp standard |
| List row height | 44pt minimum | 48dp minimum (one-line), 56dp (two-line) |
| Card padding | 16pt internal | 16dp internal |
| Safe areas | Required (notch, home indicator) | Edge-to-edge with system bar insets |

## Color

| Aspect | iOS | Android |
|---|---|---|
| System colors | Dynamic (adapt to light/dark) | Material color roles (primary, secondary, surface) |
| Accent color | App tint color | Primary color from seed |
| Dark mode | Elevated surfaces are lighter | Tonal elevation (surface color shifts) |
| Contrast | System provides high-contrast variants | Medium/High contrast themes available |
| Color scheme | System colors + custom via asset catalog | Dynamic color from wallpaper (Android 12+) |

## Components

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

## When to Follow Platform vs Cross-Platform

**Follow platform conventions when:**
- The control has a direct platform equivalent (alerts, date pickers, sheets)
- The gesture is platform-specific (iOS swipe-back, Android system back)
- Users expect platform-consistent behavior (share sheets, notifications)
- Accessibility features depend on native patterns (VoiceOver, TalkBack)

**Use cross-platform consistency when:**
- The feature is unique to your app (custom visualizations, brand experience)
- The interaction has no strong platform equivalent
- Maintaining two designs would create user confusion between platforms
- Your brand identity requires visual consistency

## Key Principles

1. **Respect the platform** — users spend 99% of their time in other apps
2. **Prioritize usability** — a familiar pattern beats a clever custom solution
3. **Test on both platforms** — what feels natural on iOS may feel foreign on Android
4. **Use adaptive layouts** — design once, adapt to platform conventions in code
5. **Accessibility first** — both platforms provide excellent a11y tools; use them
