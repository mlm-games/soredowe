use crate::Symbols;
use crate::theme::*;
use domain::{PackageSummary, Source};
use repose_core::*;
use repose_material::material3::ThemedColor;
use repose_material::material3::{
    Button, ButtonConfig, FilledIconButton, IconButton, IconButtonColors, IconButtonConfig,
    OutlinedButton, OutlinedIconButton, TextButton, TooltipAnchorPosition, TooltipBox,
    TooltipConfig, TooltipState,
};
use repose_material::{Icon, Symbol};
use repose_ui::{TextStyle, *};

fn badge(label: &str, fg: Color, bg: Color) -> View {
    Text(label.to_string())
        .size(FONT_XS)
        .line_height(Sp(15.0))
        .font_weight(FontWeight::MEDIUM)
        .color(fg)
        .modifier(
            Modifier::new()
                .padding_values(PaddingValues {
                    left: Dp(7.0),
                    right: Dp(7.0),
                    top: Dp(3.0),
                    bottom: Dp(3.0),
                })
                .background(bg)
                .clip_rounded(R_XS),
        )
}

pub fn aur_badge() -> View {
    badge("AUR", aur(), aur_bg())
}

pub fn repo_badge() -> View {
    badge("Repo", repo(), repo_bg())
}

pub fn flatpak_badge() -> View {
    badge("Flatpak", flatpak(), flatpak_bg())
}

pub fn appimage_badge() -> View {
    badge("AppImage", appimage(), appimage_bg())
}

pub fn installed_badge() -> View {
    badge("Installed", installed(), installed_bg())
}

pub fn source_badge(pkg: &PackageSummary) -> View {
    match pkg.id.source {
        Source::Aur => aur_badge(),
        Source::Flatpak => flatpak_badge(),
        Source::AppImage => appimage_badge(),
        Source::Repo => match pkg.id.repo.as_deref() {
            Some(repo) if !repo.is_empty() => badge(repo, text_muted(), card_surface()),
            _ => repo_badge(),
        },
    }
}

pub fn count_badge(count: usize) -> View {
    badge(&count.to_string(), accent(), accent_bg())
}

fn source_colors(source: Source) -> (Symbol, Color, Color) {
    match source {
        Source::Repo => (Symbols::INVENTORY, repo(), repo_bg()),
        Source::Aur => (Symbols::TERMINAL, aur(), aur_bg()),
        Source::Flatpak => (Symbols::APPS, flatpak(), flatpak_bg()),
        Source::AppImage => (Symbols::PACKAGE, appimage(), appimage_bg()),
    }
}

pub fn source_icon(source: Source, size: Dp) -> View {
    let (symbol, fg, bg) = source_colors(source);
    Box(Modifier::new()
        .size(size, size)
        .background(bg)
        .clip_rounded(R_MD)
        .align_items(AlignItems::CENTER)
        .justify_content(JustifyContent::CENTER))
    .child(Icon(symbol).size(Sp(size.0 * 0.48)).color(fg).single_line())
}

fn state_colors(hovered: Color, pressed: Color) -> StateColors {
    StateColors {
        default: Color::TRANSPARENT,
        hovered,
        focused: hovered,
        pressed,
        disabled: Color::TRANSPARENT,
        dragged: pressed,
    }
}

pub fn primary_button(label: &str, on_click: impl Fn() + 'static) -> View {
    Button(
        Modifier::new(),
        on_click,
        ButtonConfig {
            container_color: Some(accent()),
            content_color: Some(accent_ink()),
            state_colors: state_colors(accent_hover(), accent_pressed()),
            height: CONTROL_HEIGHT,
            shape_radius: R_SM,
            content_padding: Some(PaddingValues {
                left: Dp(16.0),
                right: Dp(16.0),
                top: Dp(0.0),
                bottom: Dp(0.0),
            }),
            ..Default::default()
        },
        || {
            Text(label)
                .size(FONT_BASE)
                .font_weight(FontWeight::SEMI_BOLD)
                .single_line()
        },
    )
}

pub fn secondary_button(label: &str, on_click: impl Fn() + 'static) -> View {
    OutlinedButton(
        Modifier::new(),
        on_click,
        ButtonConfig {
            container_color: Some(Color::TRANSPARENT),
            content_color: Some(text_secondary()),
            state_colors: state_colors(surface_alt(), surface_hover()),
            border: Some((Dp(1.0), card_border(), R_SM)),
            height: CONTROL_HEIGHT,
            shape_radius: R_SM,
            content_padding: Some(PaddingValues {
                left: Dp(14.0),
                right: Dp(14.0),
                top: Dp(0.0),
                bottom: Dp(0.0),
            }),
            ..Default::default()
        },
        || {
            Text(label)
                .size(FONT_BASE)
                .font_weight(FontWeight::MEDIUM)
                .single_line()
        },
    )
}

pub fn search_button(on_click: impl Fn() + 'static) -> View {
    OutlinedIconButton(
        Icon(Symbols::SEARCH).size(Sp(20.0)).single_line(),
        on_click,
        IconButtonConfig {
            modifier: Modifier::new()
                .semantics(Semantics::new(Role::Button).with_label("Search packages")),
            colors: IconButtonColors {
                container_color: Color::TRANSPARENT,
                content_color: text_secondary(),
                disabled_container_color: Color::TRANSPARENT,
                disabled_content_color: text_dimmed(),
            },
            container_size: Some(SEARCH_HEIGHT),
            shape_radius: Some(R_MD),
            ..Default::default()
        },
    )
}

pub fn secondary_icon_button(symbol: Symbol, label: &str, on_click: impl Fn() + 'static) -> View {
    OutlinedButton(
        Modifier::new(),
        on_click,
        ButtonConfig {
            container_color: Some(Color::TRANSPARENT),
            content_color: Some(text_secondary()),
            state_colors: state_colors(surface_alt(), surface_hover()),
            border: Some((Dp(1.0), card_border(), R_SM)),
            height: CONTROL_HEIGHT,
            shape_radius: R_SM,
            content_padding: Some(PaddingValues {
                left: Dp(12.0),
                right: Dp(12.0),
                top: Dp(0.0),
                bottom: Dp(0.0),
            }),
            ..Default::default()
        },
        || {
            Row(Modifier::new().gap(SPACE_2).align_items(AlignItems::CENTER)).child((
                Icon(symbol)
                    .size(Sp(18.0))
                    .color(text_secondary())
                    .single_line(),
                Text(label).size(FONT_BASE).single_line(),
            ))
        },
    )
}

pub fn quiet_button(label: &str, on_click: impl Fn() + 'static) -> View {
    TextButton(
        Modifier::new(),
        on_click,
        ButtonConfig {
            content_color: Some(text_secondary()),
            state_colors: state_colors(surface_alt(), surface_hover()),
            height: Dp(36.0),
            shape_radius: R_SM,
            content_padding: Some(PaddingValues {
                left: Dp(10.0),
                right: Dp(10.0),
                top: Dp(0.0),
                bottom: Dp(0.0),
            }),
            ..Default::default()
        },
        || {
            Text(label)
                .size(FONT_SM)
                .font_weight(FontWeight::MEDIUM)
                .single_line()
        },
    )
}

pub fn success_button(label: &str, on_click: impl Fn() + 'static) -> View {
    Button(
        Modifier::new(),
        on_click,
        ButtonConfig {
            container_color: Some(accent()),
            content_color: Some(accent_ink()),
            state_colors: state_colors(accent_hover(), accent_pressed()),
            height: Dp(36.0),
            shape_radius: R_SM,
            content_padding: Some(PaddingValues {
                left: Dp(14.0),
                right: Dp(14.0),
                top: Dp(0.0),
                bottom: Dp(0.0),
            }),
            ..Default::default()
        },
        || {
            Text(label)
                .size(FONT_SM)
                .font_weight(FontWeight::SEMI_BOLD)
                .single_line()
        },
    )
}

pub fn danger_button(label: &str, on_click: impl Fn() + 'static) -> View {
    Button(
        Modifier::new(),
        on_click,
        ButtonConfig {
            container_color: Some(danger_bg()),
            content_color: Some(danger()),
            state_colors: state_colors(danger_border(), danger()),
            border: Some((Dp(1.0), danger_border(), R_SM)),
            height: Dp(36.0),
            shape_radius: R_SM,
            content_padding: Some(PaddingValues {
                left: Dp(14.0),
                right: Dp(14.0),
                top: Dp(0.0),
                bottom: Dp(0.0),
            }),
            ..Default::default()
        },
        || {
            Text(label)
                .size(FONT_SM)
                .font_weight(FontWeight::SEMI_BOLD)
                .single_line()
        },
    )
}

pub fn tooltip_icon_button(
    symbol: Symbol,
    label: &'static str,
    on_click: impl Fn() + 'static,
    filled: bool,
) -> View {
    let state = remember(TooltipState::new);
    let icon = Icon(symbol).size(Sp(20.0)).single_line();
    let colors = IconButtonColors {
        container_color: if filled { accent() } else { Color::TRANSPARENT },
        content_color: if filled {
            accent_ink()
        } else {
            text_secondary()
        },
        disabled_container_color: Color::TRANSPARENT,
        disabled_content_color: text_dimmed(),
    };
    let config = IconButtonConfig {
        modifier: Modifier::new().semantics(Semantics::new(Role::Button).with_label(label)),
        colors,
        container_size: Some(if filled { SEARCH_HEIGHT } else { Dp(40.0) }),
        shape_radius: Some(if filled { R_MD } else { R_SM }),
        ..Default::default()
    };
    let button = if filled {
        FilledIconButton(icon, on_click, config)
    } else {
        IconButton(icon, on_click, config)
    };
    TooltipBox(
        label,
        state,
        Modifier::new().align_self_center(),
        button,
        TooltipConfig {
            position: TooltipAnchorPosition::Below,
            container_color: ThemedColor::Set(text_primary()),
            content_color: ThemedColor::Set(surface()),
            ..Default::default()
        },
    )
}

pub fn empty_state(symbol: Symbol, title: &str, body: &str, action: Option<View>) -> View {
    Column(
        Modifier::new()
            .fill_max_width()
            .padding_values(PaddingValues {
                left: SPACE_6,
                right: SPACE_6,
                top: SPACE_10,
                bottom: SPACE_10,
            })
            .align_items(AlignItems::CENTER)
            .justify_content(JustifyContent::CENTER),
    )
    .child((
        Box(Modifier::new()
            .size(Dp(52.0), Dp(52.0))
            .background(accent_bg())
            .clip_rounded(R_LG)
            .align_items(AlignItems::CENTER)
            .justify_content(JustifyContent::CENTER))
        .child(Icon(symbol).size(Sp(26.0)).color(accent()).single_line()),
        Box(Modifier::new().height(SPACE_4)),
        Text(title.to_string())
            .size(FONT_MD)
            .line_height(LINE_BASE)
            .font_weight(FontWeight::SEMI_BOLD)
            .color(text_primary())
            .text_align(TextAlign::Center),
        Box(Modifier::new().height(SPACE_1)),
        Text(body.to_string())
            .size(FONT_BASE)
            .line_height(LINE_BODY)
            .color(text_muted())
            .text_align(TextAlign::Center)
            .max_lines(3)
            .modifier(Modifier::new().max_width(Dp(560.0))),
        if let Some(action) = action {
            Box(Modifier::new().height(SPACE_5)).child(action)
        } else {
            Box(Modifier::new())
        },
    ))
}

pub fn detail_row(label: &str, value: &str) -> View {
    if value.is_empty() {
        return Box(Modifier::new());
    }
    Row(Modifier::new()
        .fill_max_width()
        .min_height(Dp(42.0))
        .padding_values(PaddingValues {
            left: Dp(14.0),
            right: Dp(14.0),
            top: SPACE_2,
            bottom: SPACE_2,
        }))
    .child((
        Box(Modifier::new().width(Dp(124.0))).child(
            Text(label.to_string())
                .size(FONT_SM)
                .color(text_muted())
                .single_line(),
        ),
        Box(Modifier::new().flex_grow(1.0)).child(
            Text(value.to_string())
                .size(FONT_BASE)
                .line_height(LINE_BASE)
                .color(text_secondary())
                .max_lines(4)
                .overflow_ellipsize(),
        ),
    ))
}

pub fn tag_list(label: &str, items: &[String]) -> View {
    if items.is_empty() {
        return Box(Modifier::new());
    }
    Column(Modifier::new().fill_max_width()).child((
        Text(label.to_string())
            .size(FONT_BASE)
            .font_weight(FontWeight::SEMI_BOLD)
            .color(text_primary())
            .modifier(Modifier::new().padding_values(PaddingValues {
                left: Dp(0.0),
                right: Dp(0.0),
                top: Dp(0.0),
                bottom: SPACE_2,
            })),
        Row(Modifier::new()
            .fill_max_width()
            .gap(SPACE_1)
            .flex_wrap(FlexWrap::Wrap))
        .child(
            items
                .iter()
                .take(30)
                .map(|item| {
                    Text(item.clone())
                        .size(FONT_SM)
                        .line_height(Sp(17.0))
                        .color(text_secondary())
                        .modifier(
                            Modifier::new()
                                .padding_values(PaddingValues {
                                    left: SPACE_2,
                                    right: SPACE_2,
                                    top: Dp(5.0),
                                    bottom: Dp(5.0),
                                })
                                .background(card_surface())
                                .border(Dp(1.0), card_border(), R_XS)
                                .clip_rounded(R_XS),
                        )
                })
                .collect::<Vec<_>>(),
        ),
    ))
}

pub fn format_bytes(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.1} GiB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{:.1} MiB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.0} KiB", bytes as f64 / 1024.0)
    } else {
        format!("{bytes} B")
    }
}
