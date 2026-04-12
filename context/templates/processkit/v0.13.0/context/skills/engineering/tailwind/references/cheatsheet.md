# Tailwind CSS v4 — Utility Class Cheatsheet

Quick reference for the most-used utility classes. Tailwind v4 uses CSS-first
configuration with `@theme` and supports arbitrary spacing values natively.

## Layout

| Purpose          | Classes                                                  |
|------------------|----------------------------------------------------------|
| Display          | `block` `inline-block` `flex` `inline-flex` `grid` `hidden` `contents` |
| Position         | `static` `relative` `absolute` `fixed` `sticky`         |
| Flex direction   | `flex-row` `flex-col` `flex-row-reverse` `flex-col-reverse` |
| Flex wrap        | `flex-wrap` `flex-nowrap`                                |
| Flex grow/shrink | `grow` `shrink` `grow-0` `shrink-0` `basis-1/2`         |
| Justify content  | `justify-start` `justify-center` `justify-between` `justify-end` `justify-around` |
| Align items      | `items-start` `items-center` `items-end` `items-stretch` `items-baseline` |
| Align self       | `self-auto` `self-start` `self-center` `self-end`        |
| Grid template    | `grid-cols-1` ... `grid-cols-12` `grid-rows-1` ... `grid-rows-6` |
| Grid span        | `col-span-1` ... `col-span-full` `row-span-1` ... `row-span-full` |
| Gap              | `gap-0` `gap-1` ... `gap-8` `gap-x-4` `gap-y-4`        |
| Overflow         | `overflow-auto` `overflow-hidden` `overflow-scroll` `overflow-visible` |
| Z-index          | `z-0` `z-10` `z-20` `z-30` `z-40` `z-50` `z-auto`      |

## Spacing

| Purpose     | Classes                                                       |
|-------------|---------------------------------------------------------------|
| Padding     | `p-0` `p-1` ... `p-8` `px-4` `py-2` `pt-4` `pr-4` `pb-4` `pl-4` |
| Margin      | `m-0` `m-1` ... `m-8` `mx-auto` `my-4` `mt-4` `mr-4` `mb-4` `ml-4` `-mt-4` |
| Space between | `space-x-4` `space-y-4`                                    |

> In v4, any integer works: `p-13`, `mt-17`, `gap-22` — no config needed.

## Sizing

| Purpose    | Classes                                                        |
|------------|----------------------------------------------------------------|
| Width      | `w-0` `w-px` `w-1` ... `w-96` `w-auto` `w-full` `w-screen` `w-fit` `w-min` `w-max` `w-1/2` `w-1/3` `w-2/3` `w-1/4` |
| Max width  | `max-w-xs` `max-w-sm` `max-w-md` `max-w-lg` `max-w-xl` ... `max-w-7xl` `max-w-full` `max-w-prose` |
| Min width  | `min-w-0` `min-w-full` `min-w-min` `min-w-max`                |
| Height     | `h-0` `h-1` ... `h-96` `h-auto` `h-full` `h-screen` `h-fit` `h-min` `h-max` |
| Min height | `min-h-0` `min-h-full` `min-h-screen`                         |
| Max height | `max-h-0` ... `max-h-96` `max-h-full` `max-h-screen`          |

## Typography

| Purpose       | Classes                                                     |
|---------------|-------------------------------------------------------------|
| Font size     | `text-xs` `text-sm` `text-base` `text-lg` `text-xl` `text-2xl` ... `text-9xl` |
| Font weight   | `font-thin` `font-light` `font-normal` `font-medium` `font-semibold` `font-bold` `font-extrabold` |
| Font style    | `italic` `not-italic`                                       |
| Font family   | `font-sans` `font-serif` `font-mono`                       |
| Text align    | `text-left` `text-center` `text-right` `text-justify`      |
| Text color    | `text-black` `text-white` `text-gray-500` `text-{color}-{shade}` |
| Line height   | `leading-none` `leading-tight` `leading-snug` `leading-normal` `leading-relaxed` `leading-loose` |
| Letter spacing| `tracking-tighter` `tracking-tight` `tracking-normal` `tracking-wide` `tracking-wider` `tracking-widest` |
| Text decoration| `underline` `overline` `line-through` `no-underline`       |
| Text transform| `uppercase` `lowercase` `capitalize` `normal-case`          |
| Text overflow | `truncate` `text-ellipsis` `text-clip`                      |
| Whitespace    | `whitespace-normal` `whitespace-nowrap` `whitespace-pre`     |

## Backgrounds

| Purpose       | Classes                                                     |
|---------------|-------------------------------------------------------------|
| Color         | `bg-white` `bg-black` `bg-gray-100` `bg-{color}-{shade}` `bg-transparent` |
| Opacity       | `bg-blue-500/50` `bg-black/75` (color-mix in v4)            |
| Gradient      | `bg-linear-to-r` `bg-linear-to-b` `bg-linear-45` `from-{color}` `via-{color}` `to-{color}` |
| Size          | `bg-auto` `bg-cover` `bg-contain`                           |
| Position      | `bg-center` `bg-top` `bg-bottom` `bg-left` `bg-right`      |
| Repeat        | `bg-repeat` `bg-no-repeat` `bg-repeat-x` `bg-repeat-y`     |

> v4 renamed `bg-gradient-*` to `bg-linear-*`.

## Borders

| Purpose       | Classes                                                     |
|---------------|-------------------------------------------------------------|
| Width         | `border` `border-0` `border-2` `border-4` `border-8` `border-t` `border-r` `border-b` `border-l` |
| Color         | `border-gray-200` `border-{color}-{shade}` `border-transparent` |
| Style         | `border-solid` `border-dashed` `border-dotted` `border-none` |
| Radius        | `rounded-none` `rounded-sm` `rounded` `rounded-md` `rounded-lg` `rounded-xl` `rounded-2xl` `rounded-full` |
| Divide        | `divide-x` `divide-y` `divide-gray-200`                    |
| Ring          | `ring` `ring-2` `ring-4` `ring-{color}-{shade}` `ring-offset-2` |

## Effects

| Purpose       | Classes                                                     |
|---------------|-------------------------------------------------------------|
| Shadow        | `shadow-sm` `shadow` `shadow-md` `shadow-lg` `shadow-xl` `shadow-2xl` `shadow-none` |
| Opacity       | `opacity-0` `opacity-25` `opacity-50` `opacity-75` `opacity-100` |
| Blur          | `blur-none` `blur-sm` `blur` `blur-md` `blur-lg`           |
| Backdrop blur  | `backdrop-blur-sm` `backdrop-blur` `backdrop-blur-md`      |

## Transitions and Animation

| Purpose       | Classes                                                     |
|---------------|-------------------------------------------------------------|
| Property      | `transition-none` `transition-all` `transition` `transition-colors` `transition-opacity` `transition-shadow` `transition-transform` |
| Duration      | `duration-75` `duration-100` `duration-150` `duration-200` `duration-300` `duration-500` `duration-700` `duration-1000` |
| Timing        | `ease-linear` `ease-in` `ease-out` `ease-in-out`           |
| Animation     | `animate-spin` `animate-ping` `animate-pulse` `animate-bounce` `animate-none` |
| Transform     | `scale-50` ... `scale-150` `rotate-45` `rotate-90` `translate-x-4` `translate-y-4` |

## Interactivity

| Purpose       | Classes                                                     |
|---------------|-------------------------------------------------------------|
| Cursor        | `cursor-pointer` `cursor-default` `cursor-not-allowed` `cursor-wait` |
| User select   | `select-none` `select-text` `select-all` `select-auto`     |
| Pointer events| `pointer-events-none` `pointer-events-auto`                |
| Scroll        | `scroll-smooth` `scroll-auto` `snap-start` `snap-center`   |

## Responsive Breakpoints

| Prefix | Min-width | Typical use             |
|--------|-----------|-------------------------|
| (none) | 0px       | Mobile-first base       |
| `sm:`  | 640px     | Large phones, landscape  |
| `md:`  | 768px     | Tablets                  |
| `lg:`  | 1024px    | Laptops                  |
| `xl:`  | 1280px    | Desktops                 |
| `2xl:` | 1536px    | Large screens            |

## State Variants

| Variant            | Trigger                              |
|--------------------|--------------------------------------|
| `hover:`           | Mouse hover                          |
| `focus:`           | Element focused                      |
| `focus-visible:`   | Keyboard focus (recommended over focus:) |
| `active:`          | Being clicked/pressed                |
| `disabled:`        | Disabled form element                |
| `first:` `last:`   | First/last child                    |
| `odd:` `even:`     | Odd/even children                   |
| `dark:`            | Dark mode                            |
| `motion-reduce:`   | Prefers reduced motion               |
| `@sm:` `@md:` etc. | Container query breakpoints (v4)    |
| `not-hover:`       | Inverse of hover (v4)               |

## Accessibility

| Purpose           | Classes                                              |
|--------------------|------------------------------------------------------|
| Screen reader only | `sr-only` `not-sr-only`                             |
| Focus ring         | `focus-visible:ring-2 focus-visible:ring-blue-500`  |
| Reduced motion     | `motion-reduce:transition-none`                     |
| Forced colors      | `forced-colors:border`                              |
