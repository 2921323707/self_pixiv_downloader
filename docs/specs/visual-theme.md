# Visual Theme Specification

Requirements: `REQ-THEME-001` through `REQ-THEME-005`

## Product Aesthetic

Pixiv is centered on anime, illustration, character art, color, and personal taste. The UI should honor that by being image-first and emotionally polished, while still behaving like a practical local tool.

Target feel:

- Image-first, not form-first.
- Light and expressive, but not visually noisy.
- Dense enough for managing many images.
- Anime/Pixiv-compatible without copying Pixiv branding.
- Mature about R18/NSFW controls.

Avoid:

- Pure marketing landing-page layout.
- Overly dark, low-contrast dashboards that make image color hard to judge.
- One-note palettes dominated by only purple, beige, brown, or slate.
- Decorative elements that compete with thumbnails.
- In-app tutorial text explaining obvious controls.

## Theme System

Each theme defines:

- `background`
- `surface`
- `surface_elevated`
- `text_primary`
- `text_secondary`
- `border`
- `accent`
- `accent_soft`
- `success`
- `warning`
- `danger`
- `r18`
- `thumbnail_placeholder`
- `shadow`

Theme should be represented as CSS variables at app root.

## Five V1 Themes

### 1. Sakura Light

Use case: bright anime gallery, friendly first-run experience.

Palette direction:

- Warm white background.
- Soft pink accent.
- Ink-like dark text.
- Mint or cyan secondary accent for controls/status.
- Red/ruby reserved for R18/danger.

Personality: clean, soft, optimistic.

### 2. Cyan Studio

Selected default theme.

Use case: cool illustration workstation.

Palette direction:

- Neutral light gray background.
- Cyan/blue accent.
- Charcoal text.
- Lime or coral status accents.
- R18 uses magenta/ruby, not the primary blue.

Personality: precise, modern, creator-tool oriented.

Default token direction:

| Token | Value |
| --- | --- |
| `background` | `#f4f8fb` |
| `surface` | `#ffffff` |
| `surface_elevated` | `#ffffff` |
| `text_primary` | `#17232b` |
| `text_secondary` | `#60717b` |
| `border` | `#d8e7ee` |
| `accent` | `#0096b7` |
| `accent_soft` | `#e8f7fb` |
| `success` | `#35a853` |
| `warning` | `#e6a100` |
| `danger` | `#d64a67` |
| `r18` | `#d64a67` |
| `thumbnail_placeholder` | `#dbeaf0` |

Layout direction:

- Top navigation is acceptable for desktop if it preserves gallery width.
- Forms and task panels should use compact 8-10px radius surfaces.
- Gallery cards should use restrained chrome so cyan accents do not compete with image colors.
- Coral should be reserved for important actions or secondary highlights, not used as a dominant page wash.

### 3. Night Gallery

Use case: comfortable late-night browsing.

Palette direction:

- Deep neutral background, not pure black.
- Off-white text.
- Teal or electric blue accent.
- Amber warning.
- R18 uses rose.

Personality: immersive, calm, image-cinema-like.

### 4. Pop Canvas

Use case: expressive anime collection browsing.

Palette direction:

- Clean white background.
- Saturated but controlled coral accent.
- Secondary violet/cyan only for small UI accents.
- Stronger badges and chips.

Personality: playful, energetic, but still organized.

### 5. Mono Ink

Use case: users who want images to dominate all color.

Palette direction:

- White or near-white background.
- Black/gray text and borders.
- Single restrained blue accent.
- Status colors remain semantic.
- R18 uses explicit red/rose.

Personality: gallery-wall, minimal, focused.

## Layout Density

| Area | Density |
| --- | --- |
| Homepage | Medium, image-led dashboard |
| Download Center | Medium-high, form/tool layout |
| Task Panel | High, table/list optimized |
| Gallery | Adaptive, image grid first |
| Settings | Medium, grouped controls |

## Component Guidance

### Navigation

- Use compact nav items with icons and labels.
- Active route should be obvious through accent color and background.
- Mobile navigation must not cover gallery content permanently.

### Buttons

- Primary actions use filled accent.
- Secondary actions use subtle surface/border.
- Destructive actions use danger color.
- Icon buttons should use recognizable icons and tooltips.

### Forms

- Inputs should be calm and readable.
- Secrets use masked input by default.
- R18 policy uses segmented control or radio cards with explicit labels.
- Quantity controls use numeric input plus stepper when useful.

### Image Cards

- Cards should have small radius, stable dimensions, and minimal chrome.
- Thumbnail area must not shift after image load.
- Badges should sit at edges without hiding important image content.
- Hover controls should be compact and translucent.

### Task Indicators

- Progress bar width reflects `progress_done + progress_failed` against total.
- `completed_with_errors` uses warning state, not full danger state.
- Logs use monospace only for IDs/technical fragments, not entire UI text.

### R18/NSFW Visual Handling

R18 content is a visibility state, not just a category badge.

Policies:

- `exclude`: do not request or display R18 content.
- `include_blurred`: display blurred thumbnails with clear category badge.
- `include_visible`: display normally with category badge.
- `only_r18`: search/download/display only R18 content where supported.

The selected R18 policy should be visible on pages where it affects results.

## Typography

- Use a modern sans-serif UI font.
- Avoid negative letter spacing.
- Do not scale font size directly with viewport width.
- Large display text is reserved for image-first homepage moments; forms and panels use compact headings.

## Motion

Allowed:

- Subtle carousel transitions.
- Progress animations.
- Modal fade/scale.
- Hover reveal on image cards.

Avoid:

- Large decorative animations.
- Motion that distracts from images.
- Infinite shimmer after content is loaded.

## First-Run Theme Preview

First run should present five theme previews using real layout fragments:

- Mini gallery grid.
- Task status chip.
- Primary action button.
- R18 badge sample.

The preview should show how the theme affects image browsing, not just color swatches.
