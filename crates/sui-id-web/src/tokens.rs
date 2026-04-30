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

  /* Semantic */
  --danger-default:  #C94A4A;
  --danger-subtle:   #F6E3E3;
  --warning-default: #D49B2A;
  --success-default: #3FA37A;
  --info-default:    #4A7FC9;

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
  --warning-default: #E6B85C;
  --success-default: #5FC49A;
  --info-default:    #6FA8FF;

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
    --warning-default: #E6B85C;
    --success-default: #5FC49A;
    --info-default:    #6FA8FF;

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
"#;
