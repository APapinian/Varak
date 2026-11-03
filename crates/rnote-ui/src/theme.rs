// Varak themes.
//
// The "violet" theme follows the Violet desktop palette at
// `$XDG_STATE_HOME/violet/palette.json` (written by `violet-palette apply`) and reloads live
// when the wallpaper palette changes. The other themes are static.

// Imports
use adw::prelude::*;
use gtk4::{CssProvider, gdk, gio, glib};
use std::cell::RefCell;
use std::path::PathBuf;
use tracing::{debug, error};

/// Palette with the same keys as the Violet desktop `palette.json`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Palette {
    pub(crate) dark: bool,
    /// Window background.
    pub(crate) background: String,
    /// Views, header bars.
    pub(crate) surface: String,
    /// Cards, popovers.
    pub(crate) raised: String,
    pub(crate) border: String,
    /// Accent used for text and highlights.
    pub(crate) accent: String,
    /// Foreground.
    pub(crate) pale: String,
    pub(crate) muted: String,
    pub(crate) dim: String,
    /// Foreground on top of `glow`.
    pub(crate) ink: String,
    /// Saturated accent, used as the accent background.
    pub(crate) glow: String,
}

fn p(dark: bool, c: [&str; 10]) -> Palette {
    Palette {
        dark,
        background: c[0].into(),
        surface: c[1].into(),
        raised: c[2].into(),
        border: c[3].into(),
        accent: c[4].into(),
        pale: c[5].into(),
        muted: c[6].into(),
        dim: c[7].into(),
        ink: c[8].into(),
        glow: c[9].into(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Theme {
    /// Plain libadwaita, no overrides.
    System,
    /// Live Violet desktop palette.
    Violet,
    VioletLight,
    CatppuccinMocha,
    RosePine,
    Nord,
    GruvboxDark,
    Paper,
}

impl Theme {
    pub(crate) const ALL: [Theme; 8] = [
        Theme::System,
        Theme::Violet,
        Theme::VioletLight,
        Theme::CatppuccinMocha,
        Theme::RosePine,
        Theme::Nord,
        Theme::GruvboxDark,
        Theme::Paper,
    ];

    pub(crate) fn id(self) -> &'static str {
        match self {
            Theme::System => "system",
            Theme::Violet => "violet",
            Theme::VioletLight => "violet-light",
            Theme::CatppuccinMocha => "catppuccin-mocha",
            Theme::RosePine => "rose-pine",
            Theme::Nord => "nord",
            Theme::GruvboxDark => "gruvbox-dark",
            Theme::Paper => "paper",
        }
    }

    pub(crate) fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|t| t.id() == id)
    }

    pub(crate) fn palette(self) -> Option<Palette> {
        let pal = match self {
            Theme::System => return None,
            Theme::Violet => violet_desktop_palette().unwrap_or_else(violet_fallback),
            #[rustfmt::skip]
            Theme::VioletLight => p(false, ["#f4effa", "#fcfaff", "#ebe3f6", "#d4c8e6", "#6b4c99", "#2a1d3f", "#5d536e", "#8b819b", "#fbf7ff", "#7b56b8"]),
            #[rustfmt::skip]
            Theme::CatppuccinMocha => p(true, ["#181825", "#1e1e2e", "#313244", "#45475a", "#cba6f7", "#cdd6f4", "#a6adc8", "#6c7086", "#11111b", "#cba6f7"]),
            #[rustfmt::skip]
            Theme::RosePine => p(true, ["#191724", "#1f1d2e", "#26233a", "#403d52", "#c4a7e7", "#e0def4", "#908caa", "#6e6a86", "#191724", "#c4a7e7"]),
            #[rustfmt::skip]
            Theme::Nord => p(true, ["#2e3440", "#3b4252", "#434c5e", "#4c566a", "#88c0d0", "#eceff4", "#d8dee9", "#7b88a1", "#2e3440", "#88c0d0"]),
            #[rustfmt::skip]
            Theme::GruvboxDark => p(true, ["#1d2021", "#282828", "#3c3836", "#504945", "#fabd2f", "#ebdbb2", "#a89984", "#7c6f64", "#1d2021", "#d79921"]),
            #[rustfmt::skip]
            Theme::Paper => p(false, ["#efe6d2", "#f8f2e4", "#e7dcc3", "#d3c4a5", "#8a5a2b", "#3b3226", "#6b5e4b", "#9a8c74", "#fff8ec", "#9c6631"]),
        };
        Some(pal)
    }
}

/// Built-in hyrule-violet palette, used when the desktop palette file is missing.
fn violet_fallback() -> Palette {
    p(
        true,
        [
            "#121017", "#1a171f", "#272130", "#483e55", "#ceaffe", "#ebe3f7", "#aba2b9", "#756c84",
            "#2a1d3f", "#8a62c6",
        ],
    )
}

fn violet_palette_path() -> PathBuf {
    std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .unwrap_or_else(|| glib::home_dir().join(".local/state"))
        .join("violet")
        .join("palette.json")
}

fn is_hex_color(s: &str) -> bool {
    s.len() == 7 && s.starts_with('#') && s[1..].chars().all(|c| c.is_ascii_hexdigit())
}

fn violet_desktop_palette() -> Option<Palette> {
    let bytes = std::fs::read(violet_palette_path()).ok()?;
    let json: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    let fallback = violet_fallback();
    let get = |key: &str, default: &str| -> String {
        json.get(key)
            .and_then(|v| v.as_str())
            .filter(|s| is_hex_color(s))
            .unwrap_or(default)
            .to_string()
    };
    Some(Palette {
        dark: true,
        background: get("background", &fallback.background),
        surface: get("surface", &fallback.surface),
        raised: get("raised", &fallback.raised),
        border: get("border", &fallback.border),
        accent: get("accent", &fallback.accent),
        pale: get("pale", &fallback.pale),
        muted: get("muted", &fallback.muted),
        dim: get("dim", &fallback.dim),
        ink: get("ink", &fallback.ink),
        glow: get("glow", &fallback.glow),
    })
}

/// Generates the libadwaita CSS variable overrides for a palette.
pub(crate) fn palette_css(pal: &Palette) -> String {
    let Palette {
        background,
        surface,
        raised,
        border,
        accent,
        pale,
        muted,
        dim,
        ink,
        glow,
        ..
    } = pal;
    // On dark palettes the saturated glow carries light text; on light palettes the deep accent
    // carries the pale `ink`.
    let (accent_bg, accent_fg) = if pal.dark {
        (glow.as_str(), "#ffffff")
    } else {
        (accent.as_str(), ink.as_str())
    };
    format!(
        r#":root {{
    --accent-bg-color: {accent_bg};
    --accent-fg-color: {accent_fg};
    --accent-color: {accent};
    --window-bg-color: {background};
    --window-fg-color: {pale};
    --view-bg-color: {surface};
    --view-fg-color: {pale};
    --headerbar-bg-color: {surface};
    --headerbar-fg-color: {pale};
    --headerbar-backdrop-color: {background};
    --headerbar-border-color: {border};
    --sidebar-bg-color: {background};
    --sidebar-fg-color: {pale};
    --sidebar-backdrop-color: {background};
    --sidebar-border-color: {border};
    --secondary-sidebar-bg-color: {surface};
    --secondary-sidebar-fg-color: {pale};
    --secondary-sidebar-backdrop-color: {surface};
    --card-bg-color: {raised};
    --card-fg-color: {pale};
    --popover-bg-color: {raised};
    --popover-fg-color: {pale};
    --dialog-bg-color: {surface};
    --dialog-fg-color: {pale};
    --thumbnail-bg-color: {raised};
    --thumbnail-fg-color: {pale};
    --border-color: {border};
}}

.canvas_scroller {{
    background-color: {background};
}}

.dim-label, label.dim-label {{
    color: {muted};
}}

placeholder, .placeholder {{
    color: {dim};
}}

.modifiernode {{
    color: {glow};
}}

.modifiernode:hover {{
    color: {accent};
}}
"#
    )
}

#[derive(Default)]
struct ThemeState {
    current: Option<Theme>,
    provider: Option<CssProvider>,
    monitor: Option<gio::FileMonitor>,
}

thread_local! {
    static STATE: RefCell<ThemeState> = RefCell::new(ThemeState::default());
}

/// The theme that is currently applied.
pub(crate) fn current() -> Theme {
    STATE.with_borrow(|s| s.current.unwrap_or(Theme::System))
}

/// Applies the theme to the default display. `system_scheme` is the color scheme to restore when
/// switching back to the system theme.
pub(crate) fn apply(theme: Theme, system_scheme: adw::ColorScheme) {
    let Some(display) = gdk::Display::default() else {
        error!("Applying theme failed, no default display");
        return;
    };

    // Setting the color scheme emits notify handlers that call `current()`, so it must happen
    // outside of the state borrow.
    let scheme = STATE.with_borrow_mut(|state| {
        state.current = Some(theme);

        let scheme = match theme.palette() {
            None => {
                if let Some(provider) = state.provider.take() {
                    gtk4::style_context_remove_provider_for_display(&display, &provider);
                }
                system_scheme
            }
            Some(pal) => {
                let provider = state.provider.get_or_insert_with(|| {
                    let provider = CssProvider::new();
                    // Above the application stylesheet so the palette wins.
                    gtk4::style_context_add_provider_for_display(
                        &display,
                        &provider,
                        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION + 1,
                    );
                    provider
                });
                provider.load_from_string(&palette_css(&pal));
                if pal.dark {
                    adw::ColorScheme::ForceDark
                } else {
                    adw::ColorScheme::ForceLight
                }
            }
        };

        if theme == Theme::Violet {
            if state.monitor.is_none() {
                state.monitor = watch_violet_palette();
            }
        } else {
            state.monitor = None;
        }
        scheme
    });
    adw::StyleManager::default().set_color_scheme(scheme);
    debug!("Applied theme `{}`", theme.id());
}

fn watch_violet_palette() -> Option<gio::FileMonitor> {
    let file = gio::File::for_path(violet_palette_path());
    let monitor = match file.monitor_file(gio::FileMonitorFlags::WATCH_MOVES, gio::Cancellable::NONE)
    {
        Ok(m) => m,
        Err(e) => {
            error!("Watching the Violet palette failed, Err: {e:?}");
            return None;
        }
    };
    // palette.json is written atomically (rename), which shows up as a move or create.
    monitor.connect_changed(|_, _, _, event| {
        if matches!(
            event,
            gio::FileMonitorEvent::ChangesDoneHint
                | gio::FileMonitorEvent::Created
                | gio::FileMonitorEvent::MovedIn
                | gio::FileMonitorEvent::Renamed
        ) && current() == Theme::Violet
        {
            STATE.with_borrow(|state| {
                if let (Some(provider), Some(pal)) = (&state.provider, Theme::Violet.palette()) {
                    provider.load_from_string(&palette_css(&pal));
                }
            });
            debug!("Reloaded Violet desktop palette");
        }
    });
    Some(monitor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_ids_roundtrip() {
        for theme in Theme::ALL {
            assert_eq!(Theme::from_id(theme.id()), Some(theme));
        }
    }

    #[test]
    fn static_palettes_are_valid_hex() {
        for theme in Theme::ALL.into_iter().filter(|t| *t != Theme::Violet) {
            let Some(pal) = theme.palette() else { continue };
            for c in [
                &pal.background,
                &pal.surface,
                &pal.raised,
                &pal.border,
                &pal.accent,
                &pal.pale,
                &pal.muted,
                &pal.dim,
                &pal.ink,
                &pal.glow,
            ] {
                assert!(is_hex_color(c), "{theme:?}: {c}");
            }
        }
    }
}
