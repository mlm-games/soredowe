//! Each function here should only return a `View` and take only the data it needs.
//! No function here knows about `Store` or `Action`.

use crate::theme::*;
use domain::{PackageSummary, Source};
use repose_core::*;
use repose_material::material3::{
    Button, ButtonConfig, Card, CardConfig, FilledTonalButton, OutlinedButton,
};
use repose_material::{Icon, Symbol};
use repose_ui::{TextStyle, *};

fn badge(label: &str, fg: &str, bg: &str) -> View {
    Text(label.to_string())
        .size(FONT_XS)
        .color(Color::from_hex(fg))
        .modifier(
            Modifier::new()
                .padding_values(PaddingValues {
                    left: Dp(8.0),
                    right: Dp(8.0),
                    top: Dp(3.0),
                    bottom: Dp(3.0),
                })
                .background(Color::from_hex(bg))
                .clip_rounded(R_SM),
        )
}

pub fn aur_badge() -> View {
    badge("AUR", PURPLE, PURPLE_BG)
}
pub fn repo_badge() -> View {
    badge("Repo", TEAL, TEAL_BG)
}
pub fn flatpak_badge() -> View {
    badge("Flatpak", FLATPAK_BORDER, FLATPAK_BG)
}
pub fn appimage_badge() -> View {
    badge("AppImage", APPIMAGE_BORDER, APPIMAGE_BG)
}
pub fn installed_badge() -> View {
    badge("Installed", INDIGO, INDIGO_BG)
}
pub fn source_badge(pkg: &PackageSummary) -> View {
    match pkg.id.source {
        Source::Aur => aur_badge(),
        Source::Flatpak => flatpak_badge(),
        Source::AppImage => appimage_badge(),
        Source::Repo => {
            if let Some(repo) = &pkg.id.repo {
                badge(repo, TEXT_MUTED, CARD_SURFACE)
            } else {
                repo_badge()
            }
        }
    }
}

pub fn primary_button(label: &str, on_click: impl Fn() + 'static) -> View {
    FilledTonalButton(
        Modifier::new(),
        on_click,
        ButtonConfig {
            container_color: Some(Color::from_hex(SEL_BG)),
            content_color: Some(Color::from_hex(TEXT_PRIMARY)),
            ..Default::default()
        },
        || Text(label).size(FONT_BASE),
    )
}

pub fn secondary_button(label: &str, on_click: impl Fn() + 'static) -> View {
    OutlinedButton(
        Modifier::new(),
        on_click,
        ButtonConfig {
            container_color: Some(Color::from_hex(CARD_BORDER)),
            content_color: Some(Color::from_hex(TEXT_MUTED)),
            ..Default::default()
        },
        || Text(label).size(FONT_BASE),
    )
}

pub fn secondary_icon_button(symbol: Symbol, label: &str, on_click: impl Fn() + 'static) -> View {
    OutlinedButton(
        Modifier::new(),
        on_click,
        ButtonConfig {
            container_color: Some(Color::from_hex(CARD_BORDER)),
            content_color: Some(Color::from_hex(TEXT_MUTED)),
            ..Default::default()
        },
        || {
            Row(Modifier::new().gap(Dp(6.0)).align_items(AlignItems::CENTER)).child((
                Icon(symbol)
                    .size(Sp(18.0))
                    .color(Color::from_hex(TEXT_MUTED))
                    .single_line(),
                Text(label).size(FONT_BASE),
            ))
        },
    )
}

pub fn success_button(label: &str, on_click: impl Fn() + 'static) -> View {
    Button(
        Modifier::new(),
        on_click,
        ButtonConfig {
            container_color: Some(Color::from_hex(GREEN_BG)),
            content_color: Some(Color::from_hex(TEXT_PRIMARY)),
            ..Default::default()
        },
        || Text(label).size(FONT_BASE),
    )
}

pub fn danger_button(label: &str, on_click: impl Fn() + 'static) -> View {
    Button(
        Modifier::new(),
        on_click,
        ButtonConfig {
            container_color: Some(Color::from_hex(RED_BG)),
            content_color: Some(Color::from_hex(RED)),
            ..Default::default()
        },
        || Text(label).size(FONT_BASE),
    )
}

pub fn empty_state(title: &str) -> View {
    Column(Modifier::new().fill_max_width()).child(Card(
        CardConfig {
            container_color: Color::from_hex(CARD_BG),
            border: Some((Dp(1.0), Color::from_hex(CARD_BORDER))),
            shape_radius: R_LG,
            ..Default::default()
        },
        || {
            Column(Modifier::new().fill_max_width().padding(Dp(32.0))).child(
                Text(title)
                    .size(FONT_LG)
                    .color(Color::from_hex(TEXT_MUTED))
                    .modifier(Modifier::new().align_self_center()),
            )
        },
    ))
}

/// Labelled metadata row:  "Label    value"
pub fn detail_row(label: &str, value: &str) -> View {
    if value.is_empty() {
        return Box(Modifier::new());
    }
    Row(Modifier::new().padding_values(PaddingValues {
        left: Dp(0.0),
        right: Dp(0.0),
        top: Dp(4.0),
        bottom: Dp(4.0),
    }))
    .child((
        Text(label.to_string())
            .size(FONT_SM)
            .color(Color::from_hex(TEXT_DIMMED))
            .modifier(Modifier::new().width(Dp(110.0))),
        Text(value.to_string())
            .size(FONT_SM)
            .color(Color::from_hex(TEXT_SECONDARY))
            .max_lines(3)
            .overflow_ellipsize()
            .modifier(Modifier::new().flex_grow(1.0)),
    ))
}

/// Renders a tag list (e.g. dependencies) as a flowing set of pills.
pub fn tag_list(label: &str, items: &[String]) -> View {
    if items.is_empty() {
        return Box(Modifier::new());
    }
    Column(Modifier::new().padding_values(PaddingValues {
        left: Dp(0.0),
        right: Dp(0.0),
        top: Dp(8.0),
        bottom: Dp(4.0),
    }))
    .child((
        Text(label.to_string())
            .size(FONT_SM)
            .color(Color::from_hex(TEXT_DIMMED))
            .modifier(Modifier::new().padding_values(PaddingValues {
                left: Dp(0.0),
                right: Dp(0.0),
                top: Dp(0.0),
                bottom: Dp(6.0),
            })),
        // Simple wrapping: show them in rows. LazyColumn not needed for ≤30 deps.
        Row(Modifier::new().fill_max_width().flex_wrap(FlexWrap::Wrap)).child(
            items
                .iter()
                .take(30)
                .map(|dep| {
                    Text(dep.clone())
                        .size(FONT_XS)
                        .color(Color::from_hex(TEXT_SECONDARY))
                        .modifier(
                            Modifier::new()
                                .padding_values(PaddingValues {
                                    left: Dp(8.0),
                                    right: Dp(8.0),
                                    top: Dp(3.0),
                                    bottom: Dp(3.0),
                                })
                                .margin(Dp(2.0))
                                .background(Color::from_hex(CARD_SURFACE))
                                .clip_rounded(R_SM)
                                .border(Dp(1.0), Color::from_hex(CARD_BORDER), R_SM),
                        )
                })
                .collect::<Vec<_>>(),
        ),
    ))
}

pub fn format_bytes(b: u64) -> String {
    if b >= 1024 * 1024 {
        format!("{:.1} MiB", b as f64 / (1024.0 * 1024.0))
    } else if b >= 1024 {
        format!("{:.0} KiB", b as f64 / 1024.0)
    } else {
        format!("{b} B")
    }
}
