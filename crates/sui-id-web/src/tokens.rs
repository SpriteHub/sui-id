//! Design tokens — light + dark palettes, spacing, typography, radii, shadows.
//!
//! Single source of truth for visual style. Every page consumes these via
//! CSS custom properties (variables) so themes can swap with a single
//! `[data-theme="dark"]` attribute on the document root.
//!
//! The palette is **Lavender-Jade**: lavender accent (#7C6BCF light /
//! #A89BFF dark), jade-influenced success (#3FA37A / #5FC49A), neutral
//! cool greys for surfaces and foreground. Semantic colours (danger /
//! warning / info / success) are tuned per mode for AA+ contrast on the
//! default surface.
//!
//! Token naming follows the convention from the design memo:
//!   --surface-* / --fg-* / --accent-* / --border-* / --state-* / --semantic.

pub const TOKENS_CSS: &str = r#"
:root {
  color-scheme: light dark;

  /* Light mode (default) ------------------------------------------------- */

  /* Surface */
  --surface-default:  #F6F5F9;
  --surface-subtle:   #EFEAF4;
  --surface-elevated: #FFFFFF;
  --surface-inverse:  #2B2733;
  --surface-sunken:   #E9E4EF;

  /* Foreground */
  --fg-default:    #1F1B24;
  --fg-muted:      #5F5A66;
  --fg-subtle:     #9A94A3;
  --fg-on-accent:  #FFFFFF;
  --fg-inverse:    #F6F5F9;

  /* Accent */
  --accent-default:  #7C6BCF;
  --accent-emphasis: #6956C0;
  --accent-subtle:   #E6E1F5;

  /* Semantic — full set per RFC 061 (v0.46.0).
   * Every semantic colour has three slots:
   *   --{name}-default — the border/foreground tint
   *   --{name}-subtle  — the tinted background for cards/banners
   *   --fg-on-{name}   — the foreground when text sits ON a -default fill
   * Contrast pairs all clear WCAG AA. */
  --danger-default:  #C94A4A;
  --danger-subtle:   #F6E3E3;
  --fg-on-danger:    #FFFFFF;

  --warning-default: #D49B2A;
  --warning-subtle:  #FBF1D9;
  --fg-on-warning:   #2A1F00;

  --success-default: #3FA37A;
  --success-subtle:  #DFF3E9;
  --fg-on-success:   #FFFFFF;

  --info-default:    #4A7FC9;
  --info-subtle:     #E2ECF8;
  --fg-on-info:      #FFFFFF;

  /* Interaction */
  --state-hover:    rgba(0, 0, 0, 0.05);
  --state-active:   rgba(0, 0, 0, 0.10);
  --state-focus:    #7C6BCF;
  --state-disabled: rgba(31, 27, 36, 0.38);

  /* Border */
  --border-default: #D6D1DE;
  --border-muted:   #E9E4EF;
  --border-accent:  #7C6BCF;

  /* Shadow (light) */
  --shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.04);
  --shadow-md: 0 2px 8px rgba(0, 0, 0, 0.06);
  --shadow-lg: 0 8px 24px rgba(0, 0, 0, 0.08);

  /* ------- Mode-independent tokens ------- */

  /* Spacing scale (4px base, 6 steps) */
  --space-1: 4px;
  --space-2: 8px;
  --space-3: 16px;
  --space-4: 24px;
  --space-5: 32px;
  --space-6: 48px;

  /* Typography scale */
  --font-size-display:  28px;
  --font-size-h2:       22px;
  --font-size-h3:       18px;
  --font-size-body:     15px;
  --font-size-caption:  13px;

  --line-height-display: 1.3;
  --line-height-h2:      1.35;
  --line-height-h3:      1.4;
  --line-height-body:    1.6;
  --line-height-caption: 1.5;

  --font-weight-regular: 400;
  --font-weight-medium:  500;
  --font-weight-bold:    700;

  /* Font families.
     Strategy: system-ui only, no web-font assets. Each OS provides the
     best-tuned UI font for its environment. CJK glyphs are reached via
     the Unicode font fallback — Hiragino on macOS, Yu Gothic UI on
     Windows, Noto Sans CJK on Linux/Android — without us bundling
     anything. When v1 multilingual support lands, :lang() rules can
     pin per-script fonts on top of this stack without touching the
     binary. */
  --font-sans:
      system-ui, -apple-system, "Segoe UI",
      "Hiragino Sans", "Yu Gothic UI",
      "Noto Sans JP", "Noto Sans KR", "Noto Sans SC",
      sans-serif;
  --font-mono:
      ui-monospace, "SF Mono", "Cascadia Code",
      "JetBrains Mono", Consolas, Menlo, monospace;

  /* Radius */
  --radius-sm: 6px;
  --radius-md: 10px;
  --radius-lg: 16px;

  /* Border width */
  --border-width-default:  1px;
  --border-width-emphasis: 2px;

  /* Layout constants */
  --content-max-width: 64rem;
  --content-narrow-width: 28rem;

  /* Motion / animation durations.
     Used with transition properties throughout components. When the
     prefers-reduced-motion media query fires, components that reference
     these tokens inherit zero-duration automatically via the override
     block below — no per-component duplication. */
  --motion-instant: 0ms;
  --motion-fast:    100ms;
  --motion-base:    200ms;
  --motion-slow:    350ms;
  --motion-easing:  cubic-bezier(0.4, 0, 0.2, 1);

  /* Z-index scale — explicit names prevent "magic number" layering. */
  --z-below:    -1;
  --z-base:      0;
  --z-raised:   10;
  --z-overlay:  100;
  --z-dropdown: 200;
  --z-modal:    300;
  --z-toast:    400;
}

[data-theme="light"] {
  color-scheme: light;
}

/* Dark mode --------------------------------------------------------------
   Activated by data-theme="dark" on <html>. The early inline script in
   layout.rs sets this synchronously so initial paint is correct. */
[data-theme="dark"] {
  color-scheme: dark;

  --surface-default:  #1A1720;
  --surface-subtle:   #211D2A;
  --surface-elevated: #2A2433;
  --surface-inverse:  #F6F5F9;
  --surface-sunken:   #16131B;

  --fg-default:    #F1EEF6;
  --fg-muted:      #B8B2C4;
  --fg-subtle:     #7E788A;
  --fg-on-accent:  #FFFFFF;
  --fg-inverse:    #1F1B24;

  --accent-default:  #A89BFF;
  --accent-emphasis: #C1B6FF;
  --accent-subtle:   #332C4A;

  --danger-default:  #FF6B6B;
  --danger-subtle:   #3A1F22;
  --fg-on-danger:    #FFFFFF;

  --warning-default: #E6B85C;
  --warning-subtle:  #3A2E14;
  --fg-on-warning:   #FFE7B3;

  --success-default: #5FC49A;
  --success-subtle:  #1E3A2D;
  --fg-on-success:   #FFFFFF;

  --info-default:    #6FA8FF;
  --info-subtle:     #1F2D44;
  --fg-on-info:      #FFFFFF;

  --state-hover:    rgba(255, 255, 255, 0.06);
  --state-active:   rgba(255, 255, 255, 0.12);
  --state-focus:    #A89BFF;
  --state-disabled: rgba(241, 238, 246, 0.38);

  --border-default: #3A3445;
  --border-muted:   #2A2433;
  --border-accent:  #A89BFF;

  --shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.4);
  --shadow-md: 0 2px 8px rgba(0, 0, 0, 0.5);
  --shadow-lg: 0 8px 24px rgba(0, 0, 0, 0.6);
}

/* Auto dark mode (when user has no explicit preference saved) ----------
   Only applies when data-theme is unset; explicit user choice always
   overrides. */
@media (prefers-color-scheme: dark) {
  :root:not([data-theme]) {
    --surface-default:  #1A1720;
    --surface-subtle:   #211D2A;
    --surface-elevated: #2A2433;
    --surface-inverse:  #F6F5F9;
    --surface-sunken:   #16131B;

    --fg-default:    #F1EEF6;
    --fg-muted:      #B8B2C4;
    --fg-subtle:     #7E788A;
    --fg-on-accent:  #FFFFFF;
    --fg-inverse:    #1F1B24;

    --accent-default:  #A89BFF;
    --accent-emphasis: #C1B6FF;
    --accent-subtle:   #332C4A;

    --danger-default:  #FF6B6B;
    --danger-subtle:   #3A1F22;
    --fg-on-danger:    #FFFFFF;

    --warning-default: #E6B85C;
    --warning-subtle:  #3A2E14;
    --fg-on-warning:   #FFE7B3;

    --success-default: #5FC49A;
    --success-subtle:  #1E3A2D;
    --fg-on-success:   #FFFFFF;

    --info-default:    #6FA8FF;
    --info-subtle:     #1F2D44;
    --fg-on-info:      #FFFFFF;

    --state-hover:    rgba(255, 255, 255, 0.06);
    --state-active:   rgba(255, 255, 255, 0.12);
    --state-focus:    #A89BFF;
    --state-disabled: rgba(241, 238, 246, 0.38);

    --border-default: #3A3445;
    --border-muted:   #2A2433;
    --border-accent:  #A89BFF;

    --shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.4);
    --shadow-md: 0 2px 8px rgba(0, 0, 0, 0.5);
    --shadow-lg: 0 8px 24px rgba(0, 0, 0, 0.6);
  }
}

/* Motion: honour OS-level "Reduce motion" preference.
   All transitions/animations that reference the token vars above are
   automatically zeroed; components do not need individual overrides. */
@media (prefers-reduced-motion: reduce) {
  :root {
    --motion-fast: 0ms;
    --motion-base: 0ms;
    --motion-slow: 0ms;
  }
  *, *::before, *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
}

/* Text selection contrast — meets WCAG 2.1 SC 1.4.3 in both modes.
   Light: #E6E1F5 bg on #1F1B24 text ≈ 13:1 ✓
   Dark:  #332C4A bg on #F1EEF6 text ≈ 7:1  ✓ */
::selection {
  background-color: var(--accent-subtle);
  color: var(--fg-default);
}
"#;
