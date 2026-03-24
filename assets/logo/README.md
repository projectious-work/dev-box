# aibox — Logo Package

App icon for aibox, a projectious.work project.
Terminal brackets `< >` with ✨ sparkle stars — the AI inside the container.

## Final specification

- **Center star:** #E05232 (accent orange), opacity 100%, r=0.22
- **Companion star:** #E05232, opacity 87%, r=0.075, position (0.62, 0.6)
- **Star cut-out ring:** Background-colored stroke (stencil technique from parent brand)
- **Brackets:** Background-colored, stroke-width 0.05×, arms at 0.26/0.74
- **Box:** Rounded rectangle, rx=0.1×, midnight on light / slate-blue on dark
- **Star tips contained within bracket top/bottom lines** (y=0.28 to y=0.72)

## Structure

```
aibox-package/
├── svg/                         # Vector sources
│   ├── aibox-icon-light.svg     # For light backgrounds
│   ├── aibox-icon-dark.svg      # For dark backgrounds
│   ├── aibox-favicon.svg        # With midnight background (for favicons)
│   ├── aibox-mono-black.svg     # Single color: black
│   ├── aibox-mono-white.svg     # Single color: white (slate inner)
│   ├── aibox-mono-gray.svg      # Single color: gray (print)
│   └── aibox-mono-midnight.svg  # Single color: midnight
├── png-1x/                      # Native resolution (16–1024px)
├── png-2x/                      # Retina (2× pixel dimensions)
├── png-3x/                      # Super-retina (3× pixel dimensions)
└── favicon/                     # Web favicon set
    ├── favicon.ico              # Multi-res ICO (16+32+48)
    ├── favicon-16px.png
    ├── favicon-32px.png
    ├── favicon-48px.png
    ├── favicon-64px.png
    ├── favicon-180px.png
    ├── apple-touch-icon.png     # 180×180
    ├── android-chrome-192.png   # PWA icon
    └── android-chrome-512.png   # PWA splash

## HTML favicon snippet

```html
<link rel="icon" type="image/x-icon" href="/favicon.ico">
<link rel="icon" type="image/png" sizes="32x32" href="/favicon-32px.png">
<link rel="icon" type="image/png" sizes="16x16" href="/favicon-16px.png">
<link rel="apple-touch-icon" sizes="180x180" href="/apple-touch-icon.png">
<link rel="manifest" href="/site.webmanifest">
```

## Brand relationship

aibox is branded as "aibox — a projectious.work project" and uses:
- projectious.work color palette (midnight #1d3352, accent #E05232, slate #546a82)
- Background-colored cut-out ring technique (stencil DNA from parent brand)
- Same font system (Plus Jakarta Sans / Source Sans 3 / IBM Plex Mono)
