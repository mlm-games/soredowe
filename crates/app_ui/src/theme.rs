use domain::config::ThemeMode;
use repose_core::*;
use std::cell::Cell;

#[derive(Clone, Copy)]
struct Palette {
    bg: &'static str,
    surface: &'static str,
    surface_alt: &'static str,
    surface_hover: &'static str,
    card_border: &'static str,
    card_surface: &'static str,
    text_primary: &'static str,
    text_secondary: &'static str,
    text_muted: &'static str,
    text_dimmed: &'static str,
    accent: &'static str,
    accent_hover: &'static str,
    accent_pressed: &'static str,
    accent_bg: &'static str,
    accent_border: &'static str,
    accent_ink: &'static str,
    success: &'static str,
    success_bg: &'static str,
    success_border: &'static str,
    danger: &'static str,
    danger_bg: &'static str,
    danger_border: &'static str,
    warning: &'static str,
    warning_bg: &'static str,
    aur: &'static str,
    aur_bg: &'static str,
    flatpak: &'static str,
    flatpak_bg: &'static str,
    appimage: &'static str,
    appimage_bg: &'static str,
    repo: &'static str,
    repo_bg: &'static str,
    installed: &'static str,
    installed_bg: &'static str,
    log_bg: &'static str,
    log_text: &'static str,
    log_muted: &'static str,
    overlay: &'static str,
    scrim: &'static str,
}

const LIGHT: Palette = Palette {
    bg: "#F3F6FA",
    surface: "#FAFBFD",
    surface_alt: "#EDF1F6",
    surface_hover: "#E7ECF3",
    card_border: "#D7DEE8",
    card_surface: "#F0F3F8",
    text_primary: "#192330",
    text_secondary: "#3D4958",
    text_muted: "#566273",
    text_dimmed: "#647181",
    accent: "#2E64D4",
    accent_hover: "#2859BE",
    accent_pressed: "#214B9F",
    accent_bg: "#E7EEFC",
    accent_border: "#A9C0F0",
    accent_ink: "#F8FAFF",
    success: "#187153",
    success_bg: "#E3F2EC",
    success_border: "#A5D4C4",
    danger: "#B43A4B",
    danger_bg: "#FBE9EC",
    danger_border: "#E4A8B0",
    warning: "#9A6519",
    warning_bg: "#FFF3DA",
    aur: "#8B5A18",
    aur_bg: "#FFF1D5",
    flatpak: "#2B64A8",
    flatpak_bg: "#E6EFFA",
    appimage: "#A64B2A",
    appimage_bg: "#F9E9E2",
    repo: "#176A58",
    repo_bg: "#E1F1EC",
    installed: "#566273",
    installed_bg: "#E9EDF2",
    log_bg: "#18212D",
    log_text: "#C7D1DE",
    log_muted: "#8190A1",
    overlay: "#192330",
    scrim: "#596579",
};

const DARK: Palette = Palette {
    bg: "#0D1117",
    surface: "#121820",
    surface_alt: "#1A222C",
    surface_hover: "#202A36",
    card_border: "#293440",
    card_surface: "#1B232D",
    text_primary: "#EDF2F7",
    text_secondary: "#C2CBD5",
    text_muted: "#929EAA",
    text_dimmed: "#788593",
    accent: "#79A9FF",
    accent_hover: "#8CB7FF",
    accent_pressed: "#6393E8",
    accent_bg: "#1C2D4A",
    accent_border: "#355B91",
    accent_ink: "#08111F",
    success: "#5CC9A2",
    success_bg: "#123429",
    success_border: "#2C6B55",
    danger: "#FF8290",
    danger_bg: "#3A1B22",
    danger_border: "#7A3540",
    warning: "#E8B45E",
    warning_bg: "#352A17",
    aur: "#E7B85E",
    aur_bg: "#342B19",
    flatpak: "#78B7F4",
    flatpak_bg: "#172D44",
    appimage: "#E89B78",
    appimage_bg: "#39251D",
    repo: "#6BC7A9",
    repo_bg: "#15342B",
    installed: "#AAB4C0",
    installed_bg: "#242D37",
    log_bg: "#090D12",
    log_text: "#C3CEDA",
    log_muted: "#788593",
    overlay: "#EDF2F7",
    scrim: "#05080C",
};

thread_local! {
    static ACTIVE_THEME: Cell<ThemeMode> = const { Cell::new(ThemeMode::Dark) };
}

pub fn activate_theme(mode: ThemeMode) {
    ACTIVE_THEME.with(|active| active.set(mode));
}

fn palette() -> Palette {
    if ACTIVE_THEME.with(Cell::get) == ThemeMode::Dark {
        DARK
    } else {
        LIGHT
    }
}

macro_rules! color_fn {
    ($name:ident, $field:ident) => {
        pub fn $name() -> Color {
            Color::from_hex(palette().$field)
        }
    };
}

color_fn!(bg, bg);
color_fn!(surface, surface);
color_fn!(surface_alt, surface_alt);
color_fn!(surface_hover, surface_hover);
color_fn!(card_border, card_border);
color_fn!(card_surface, card_surface);
color_fn!(text_primary, text_primary);
color_fn!(text_secondary, text_secondary);
color_fn!(text_muted, text_muted);
color_fn!(text_dimmed, text_dimmed);
color_fn!(accent, accent);
color_fn!(accent_hover, accent_hover);
color_fn!(accent_pressed, accent_pressed);
color_fn!(accent_bg, accent_bg);
color_fn!(accent_border, accent_border);
color_fn!(accent_ink, accent_ink);
color_fn!(success, success);
color_fn!(success_bg, success_bg);
color_fn!(success_border, success_border);
color_fn!(danger, danger);
color_fn!(danger_bg, danger_bg);
color_fn!(danger_border, danger_border);
color_fn!(warning, warning);
color_fn!(warning_bg, warning_bg);
color_fn!(aur, aur);
color_fn!(aur_bg, aur_bg);
color_fn!(flatpak, flatpak);
color_fn!(flatpak_bg, flatpak_bg);
color_fn!(appimage, appimage);
color_fn!(appimage_bg, appimage_bg);
color_fn!(repo, repo);
color_fn!(repo_bg, repo_bg);
color_fn!(installed, installed);
color_fn!(installed_bg, installed_bg);
color_fn!(log_bg, log_bg);
color_fn!(log_text, log_text);
color_fn!(log_muted, log_muted);
color_fn!(overlay, overlay);
color_fn!(scrim, scrim);

pub fn material_theme() -> Theme {
    let colors = ColorScheme {
        primary: accent(),
        on_primary: accent_ink(),
        primary_container: accent_bg(),
        on_primary_container: accent(),
        secondary: text_secondary(),
        on_secondary: surface(),
        secondary_container: surface_alt(),
        on_secondary_container: text_primary(),
        tertiary: repo(),
        on_tertiary: surface(),
        tertiary_container: repo_bg(),
        on_tertiary_container: repo(),
        error: danger(),
        on_error: surface(),
        error_container: danger_bg(),
        on_error_container: danger(),
        background: bg(),
        on_background: text_primary(),
        surface: surface(),
        on_surface: text_primary(),
        surface_variant: surface_alt(),
        on_surface_variant: text_secondary(),
        surface_container_lowest: bg(),
        surface_container_low: surface(),
        surface_container: surface_alt(),
        surface_container_high: surface_hover(),
        surface_container_highest: card_surface(),
        surface_bright: surface(),
        surface_dim: bg(),
        surface_tint: accent(),
        inverse_surface: text_primary(),
        inverse_on_surface: bg(),
        inverse_primary: accent(),
        outline: card_border(),
        outline_variant: surface_hover(),
        scrim: scrim(),
        shadow: log_bg(),
        focus: accent(),
    };
    Theme {
        colors,
        focus: accent(),
        scrollbar_track: Color::TRANSPARENT,
        scrollbar_thumb: card_border(),
        button_bg: accent(),
        button_bg_hover: accent_hover(),
        button_bg_pressed: accent_pressed(),
        ..Theme::default()
    }
}

pub const R_XS: Dp = Dp(4.0);
pub const R_SM: Dp = Dp(6.0);
pub const R_MD: Dp = Dp(8.0);
pub const R_LG: Dp = Dp(12.0);
pub const R_PILL: Dp = Dp(999.0);

pub const SPACE_1: Dp = Dp(4.0);
pub const SPACE_2: Dp = Dp(8.0);
pub const SPACE_3: Dp = Dp(12.0);
pub const SPACE_4: Dp = Dp(16.0);
pub const SPACE_5: Dp = Dp(20.0);
pub const SPACE_6: Dp = Dp(24.0);
pub const SPACE_8: Dp = Dp(32.0);
pub const SPACE_10: Dp = Dp(40.0);
pub const SPACE_12: Dp = Dp(48.0);

pub const CONTROL_HEIGHT: Dp = Dp(42.0);
pub const SEARCH_HEIGHT: Dp = Dp(52.0);
pub const HEADER_HEIGHT: Dp = Dp(68.0);
pub const STATUS_HEIGHT: Dp = Dp(56.0);
pub const LOG_HEIGHT: Dp = Dp(180.0);
pub const LIST_ITEM_HEIGHT: f32 = 88.0;

pub const FONT_XS: Sp = Sp(11.0);
pub const FONT_SM: Sp = Sp(12.0);
pub const FONT_BASE: Sp = Sp(14.0);
pub const FONT_MD: Sp = Sp(16.0);
pub const FONT_LG: Sp = Sp(20.0);
pub const FONT_XL: Sp = Sp(28.0);
pub const FONT_DISPLAY: Sp = Sp(34.0);
pub const FONT_MONO: &str = "monospace";
pub const LINE_TIGHT: Sp = Sp(20.0);
pub const LINE_BASE: Sp = Sp(21.0);
pub const LINE_BODY: Sp = Sp(20.0);
pub const LINE_DISPLAY: Sp = Sp(38.0);
