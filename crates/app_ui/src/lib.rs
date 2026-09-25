use crate::state::{Action, AppState, Route, SortMode, Store};
use crate::theme::*;
use crate::widgets::*;
use domain::{PackageDetails, PackageSummary, Source};
use repose_core::*;
use repose_material::material3::dialog::{Dialog, DialogProperties, DialogState};
use repose_material::material3::{
    Button, ButtonConfig, Card, CardConfig, CenterAlignedTopAppBar, Checkbox, CheckboxConfig,
    ChipColors, ChipConfig, DividerConfig, FilterChip, HorizontalDivider, LinearProgressIndicator,
    LinearProgressIndicatorConfig, ListItem, ListItemConfig, SegmentConfig, SegmentedButton,
    SegmentedButtonConfig, Surface, SurfaceConfig, Switch, SwitchConfig, TopAppBarColors,
    TopAppBarConfig,
};
use repose_material::{Icon, material_symbols};

material_symbols! {
    CHEVRON_LEFT: '\u{e5cb}',
    SEARCH: '\u{e8b6}',
}
use repose_navigation::{
    EntryScope, NavDisplay, NavTransition, Navigator, remember_back_stack, renderer,
};
use repose_ui::{
    TextStyle,
    lazy::LazyColumn,
    lazy_states::LazyColumnState,
    scroll::{ScrollArea, remember_scroll_state},
    *,
};
use std::rc::Rc;

pub mod state;
pub mod theme;
pub mod widgets;

const LOG_HEIGHT_DP: f32 = 180.0;

fn top_bar(store: &Rc<Store>, s: &AppState) -> View {
    let title = if s.in_upgrades_view {
        "Upgrades"
    } else {
        "Soredowe"
    };

    CenterAlignedTopAppBar(
        Text(title)
            .size(FONT_2XL)
            .color(Color::from_hex(TEXT_PRIMARY)),
        None,
        None,
        vec![
            if s.in_upgrades_view {
                if !s.results.is_empty() {
                    success_button("Upgrade all", {
                        let store = store.clone();
                        move || store.dispatch(Action::UpgradeAll)
                    })
                } else {
                    Box(Modifier::new())
                }
            } else {
                primary_button("Upgrades", {
                    let store = store.clone();
                    move || store.dispatch(Action::Upgrades)
                })
            },
            Space(Modifier::new().width(Dp(8.0))),
            secondary_button("Install File", {
                let store = store.clone();
                move || store.dispatch(Action::PickFile)
            }),
            Space(Modifier::new().width(Dp(8.0))),
            secondary_button("Refresh", {
                let store = store.clone();
                move || store.dispatch(Action::Refresh)
            }),
            Space(Modifier::new().width(Dp(8.0))),
            secondary_button("Settings", {
                let store = store.clone();
                move || store.dispatch(Action::OpenSettings)
            }),
        ],
        TopAppBarConfig {
            colors: TopAppBarColors {
                container_color: Color::TRANSPARENT,
                scrolled_container_color: Color::TRANSPARENT,
                ..Default::default()
            },
            ..Default::default()
        },
    )
}

fn search_section(store: &Rc<Store>, s: &AppState) -> View {
    Column(
        Modifier::new()
            .fill_max_width()
            .padding_values(PaddingValues {
                left: Dp(0.0),
                right: Dp(0.0),
                top: Dp(8.0),
                bottom: Dp(8.0),
            }),
    )
    .child((
        Row(Modifier::new().fill_max_width()).child((
            Row(Modifier::new()
                .flex_grow(1.0)
                .fill_max_width()
                .height(Dp(40.0))
                .background(Color::from_hex(CARD_BG))
                .border(Dp(1.0), Color::from_hex(CARD_BORDER), R_MD)
                .clip_rounded(R_MD)
                .align_items(AlignItems::CENTER)
                .padding_values(PaddingValues {
                    left: Dp(12.0),
                    right: Dp(12.0),
                    top: Dp(0.0),
                    bottom: Dp(0.0),
                }))
            .child((
                Icon(Symbols::SEARCH)
                    .size(Sp(18.0))
                    .color(Color::from_hex(TEXT_MUTED))
                    .single_line(),
                Space(Modifier::new().width(Dp(8.0))),
                {
                    let search_state = remember_state(TextFieldState::new);
                    repose_ui::BasicTextField(
                        search_state,
                        Modifier::new()
                            .flex_grow(1.0)
                            .fill_max_width()
                            .height(Dp(40.0))
                            .padding_values(PaddingValues {
                                left: Dp(0.0),
                                right: Dp(0.0),
                                top: Dp(0.0),
                                bottom: Dp(0.0),
                            })
                            .semantics(Semantics {
                                role: Role::TextField,
                                label: Some("Search field".into()),
                                focused: false,
                                enabled: true,
                                selectable_group: false,
                                checked: None,
                                selected: None,
                                value: None,
                            }),
                        "Search packages",
                        TextFieldConfig {
                            line_limits: TextFieldLineLimits::SingleLine,
                            on_change: Some(Rc::new({
                                let store = store.clone();
                                move |text: String| store.dispatch(Action::SetQuery(text))
                            })),
                            on_submit: Some(Rc::new({
                                let store = store.clone();
                                move |text: String| {
                                    store.dispatch(Action::SetQuery(text));
                                    store.dispatch(Action::Search);
                                }
                            })),
                            ..Default::default()
                        },
                    )
                },
            )),
            Space(Modifier::new().width(Dp(8.0))),
            secondary_button("Search", {
                let store = store.clone();
                move || store.dispatch(Action::Search)
            }),
        )),
        Space(Modifier::new().height(Dp(8.0))),
        {
            let chip_cfg = ChipConfig {
                colors: ChipColors {
                    container_color: Color::from_hex(CARD_BORDER),
                    label_color: Color::from_hex(TEXT_MUTED),
                    leading_icon_color: Color::from_hex(TEXT_MUTED),
                    trailing_icon_color: Color::from_hex(TEXT_MUTED),
                    disabled_container_color: Color::from_hex(CARD_BORDER),
                    disabled_label_color: Color::from_hex(TEXT_DIMMED),
                    disabled_leading_icon_color: Color::from_hex(TEXT_DIMMED),
                    disabled_trailing_icon_color: Color::from_hex(TEXT_DIMMED),
                    selected_container_color: Color::from_hex(SEL_BG),
                    selected_label_color: Color::from_hex(TEXT_PRIMARY),
                    selected_leading_icon_color: Color::from_hex(TEXT_PRIMARY),
                    selected_trailing_icon_color: Color::from_hex(TEXT_PRIMARY),
                    disabled_selected_container_color: Color::from_hex(SEL_BG),
                },
                border_color: Color::from_hex(CARD_BORDER),
                selected_border_color: Color::from_hex(BLUE_BORDER),
                ..Default::default()
            };
            Row(Modifier::new()
                .fill_max_width()
                .align_items(AlignItems::CENTER))
            .child(vec![
                FilterChip(
                    s.filter_repo,
                    {
                        let store = store.clone();
                        move || store.dispatch(Action::ToggleFilterRepo)
                    },
                    Text("Repo").size(FONT_SM),
                    None,
                    None,
                    ChipConfig { ..chip_cfg.clone() },
                ),
                Space(Modifier::new().width(Dp(6.0))),
                FilterChip(
                    s.filter_aur,
                    {
                        let store = store.clone();
                        move || store.dispatch(Action::ToggleFilterAur)
                    },
                    Text("AUR").size(FONT_SM),
                    None,
                    None,
                    ChipConfig { ..chip_cfg.clone() },
                ),
                Space(Modifier::new().width(Dp(6.0))),
                FilterChip(
                    s.filter_flatpak,
                    {
                        let store = store.clone();
                        move || store.dispatch(Action::ToggleFilterFlatpak)
                    },
                    Text("Flatpak").size(FONT_SM),
                    None,
                    None,
                    ChipConfig { ..chip_cfg.clone() },
                ),
                Space(Modifier::new().width(Dp(6.0))),
                FilterChip(
                    s.filter_appimage,
                    {
                        let store = store.clone();
                        move || store.dispatch(Action::ToggleFilterAppImage)
                    },
                    Text("AppImage").size(FONT_SM),
                    None,
                    None,
                    ChipConfig { ..chip_cfg.clone() },
                ),
                Space(Modifier::new().width(Dp(6.0))),
                FilterChip(
                    s.filter_installed,
                    {
                        let store = store.clone();
                        move || store.dispatch(Action::ToggleFilterInstalled)
                    },
                    Text("Installed only").size(FONT_SM),
                    None,
                    None,
                    ChipConfig { ..chip_cfg },
                ),
                Spacer(),
                SegmentedButton(
                    &[match s.sort {
                        SortMode::Popularity => 0,
                        SortMode::NameAsc => 1,
                        SortMode::NameDesc => 2,
                    }],
                    vec![
                        SegmentConfig {
                            label: "Popular".into(),
                            icon: None,
                            on_click: Rc::new({
                                let store = store.clone();
                                move || store.dispatch(Action::SetSort(SortMode::Popularity))
                            }),
                            enabled: true,
                            interaction_source: None,
                        },
                        SegmentConfig {
                            label: "A-Z".into(),
                            icon: None,
                            on_click: Rc::new({
                                let store = store.clone();
                                move || store.dispatch(Action::SetSort(SortMode::NameAsc))
                            }),
                            enabled: true,
                            interaction_source: None,
                        },
                        SegmentConfig {
                            label: "Z-A".into(),
                            icon: None,
                            on_click: Rc::new({
                                let store = store.clone();
                                move || store.dispatch(Action::SetSort(SortMode::NameDesc))
                            }),
                            enabled: true,
                            interaction_source: None,
                        },
                    ],
                    SegmentedButtonConfig {
                        selected_container_color: Color::from_hex(SEL_BG),
                        selected_content_color: Color::from_hex(TEXT_PRIMARY),
                        unselected_content_color: Color::from_hex(TEXT_MUTED),
                        border_color: Color::from_hex(CARD_BORDER),
                        ..Default::default()
                    },
                ),
            ])
        },
    ))
}

fn pkg_action(store: &Rc<Store>, pkg: &PackageSummary, upgrades_mode: bool) -> View {
    let id = pkg.id.clone();
    if upgrades_mode {
        let s = store.clone();
        success_button("Upgrade", move || s.dispatch(Action::Upgrade(id.clone())))
    } else if pkg.installed {
        let s = store.clone();
        danger_button("Remove", move || s.dispatch(Action::Remove(id.clone())))
    } else {
        let s = store.clone();
        success_button("Install", move || s.dispatch(Action::Install(id.clone())))
    }
}

fn pkg_row(store: Rc<Store>, pkg: PackageSummary, _selected: bool, upgrades_mode: bool) -> View {
    let (bg, border) = match pkg.id.source {
        Source::Aur => (AUR_BG, AUR_BORDER),
        Source::Flatpak => (FLATPAK_BG, FLATPAK_BORDER),
        Source::AppImage => (APPIMAGE_BG, APPIMAGE_BORDER),
        Source::Repo => (CARD_BG, CARD_BORDER),
    };

    Row(Modifier::new()
        .fill_max_width()
        .padding(Dp(12.0))
        .margin_vertical(Dp(3.0))
        .background(Color::from_hex(bg))
        .border(Dp(1.0), Color::from_hex(border), R_MD)
        .state_colors(StateColors {
            default: Color::TRANSPARENT,
            hovered: Color::from_hex(STATE_OVERLAY).with_alpha_f32(0.02),
            focused: Color::from_hex(STATE_OVERLAY).with_alpha_f32(0.02),
            pressed: Color::from_hex(STATE_OVERLAY).with_alpha_f32(0.10),
            disabled: Color::TRANSPARENT,
            dragged: Color::from_hex(STATE_OVERLAY).with_alpha_f32(0.10),
        })
        .clip_rounded(R_MD)
        .clickable()
        .on_pointer_down({
            let store = store.clone();
            let id = pkg.id.clone();
            move |_| store.dispatch(Action::Select(id.clone()))
        }))
    .child((
        Column(Modifier::new().flex_grow(1.0)).child((
            Row(Modifier::new().align_items(AlignItems::CENTER)).child((
                Text(pkg.id.name.clone())
                    .size(FONT_LG)
                    .color(Color::from_hex(TEXT_PRIMARY)),
                Space(Modifier::new().width(Dp(8.0))),
                source_badge(&pkg),
                Space(Modifier::new().width(Dp(4.0))),
                if pkg.installed {
                    installed_badge()
                } else {
                    Box(Modifier::new())
                },
            )),
            Space(Modifier::new().height(Dp(4.0))),
            Text(pkg.description.clone())
                .size(FONT_SM)
                .color(Color::from_hex(TEXT_MUTED))
                .max_lines(1)
                .overflow_ellipsize()
                .modifier(Modifier::new().max_width(Dp(480.0))),
        )),
        Column(Modifier::new().align_self_center()).child((
            Text(pkg.version.clone())
                .size(FONT_XS)
                .color(Color::from_hex(TEXT_DIMMED))
                .modifier(
                    Modifier::new()
                        .align_self_center()
                        .padding_values(PaddingValues {
                            left: Dp(0.0),
                            right: Dp(0.0),
                            top: Dp(0.0),
                            bottom: Dp(4.0),
                        }),
                ),
            pkg_action(&store, &pkg, upgrades_mode),
        )),
    ))
}

fn results_list(store: &Rc<Store>, s: &AppState) -> View {
    if s.results.is_empty() {
        return empty_state("No results");
    }

    let store = store.clone();
    let selected = s.selected.clone();
    let upgrades_mode = s.in_upgrades_view;
    let results = s.results.clone();

    LazyColumn(
        results,
        76.0,
        |pkg: &PackageSummary| {
            let mut h: u64 = 0;
            for b in pkg.id.name.bytes() {
                h = h.wrapping_mul(31).wrapping_add(b as u64);
            }
            h = h.wrapping_mul(31).wrapping_add(pkg.id.source as u64);
            if let Some(ref r) = pkg.id.repo {
                for b in r.bytes() {
                    h = h.wrapping_mul(31).wrapping_add(b as u64);
                }
            }
            h
        },
        move |pkg: PackageSummary, _| {
            let is_sel = selected.as_ref().is_some_and(|id| *id == pkg.id);
            pkg_row(store.clone(), pkg, is_sel, upgrades_mode)
        },
        LazyColumnConfig {
            modifier: Modifier::new()
                .fill_max_width()
                .weight(1.0)
                .clip_rounded(R_SM)
                .padding(Dp(4.0)),
            state: remember_with_key("pkg_scroll", LazyColumnState::new),
            animate_spec: None::<repose_core::animation::AnimationSpec>,
            ..Default::default()
        },
    )
}

fn detail_overlay(store: Rc<Store>, s: &AppState) -> View {
    let Some(sel_id) = &s.selected else {
        return Box(Modifier::new());
    };

    let summary = match s.results.iter().find(|p| &p.id == sel_id) {
        Some(p) => p.clone(),
        None => return Box(Modifier::new()),
    };

    let detail_section: View = if let Some(det) = &s.detail {
        details_body(det)
    } else {
        Text("Loading details")
            .size(FONT_SM)
            .color(Color::from_hex(TEXT_DIMMED))
            .modifier(Modifier::new().padding(Dp(8.0)))
    };

    let scroll = remember_scroll_state(format!(
        "detail:{}:{:?}",
        summary.id.name, summary.id.source
    ));
    scroll.set_show_scrollbar(false);

    ScrollArea(
        Modifier::new().fill_max_size().padding(Dp(24.0)),
        scroll,
        Column(Modifier::new().fill_max_size().align_self_center()).child((
            Row(Modifier::new()
                .fill_max_width()
                .align_items(AlignItems::CENTER))
            .child((
                secondary_icon_button(Symbols::CHEVRON_LEFT, "Back", {
                    let store = store.clone();
                    move || store.dispatch(Action::ClearSelection)
                }),
                Spacer(),
                pkg_action(&store, &summary, s.in_upgrades_view),
            )),
            Space(Modifier::new().height(Dp(20.0))),
            Row(Modifier::new().fill_max_width()).child(
                Column(Modifier::new().flex_grow(1.0)).child((
                    Row(Modifier::new().align_items(AlignItems::CENTER)).child((
                        Text(summary.id.name.clone())
                            .size(Sp(28.0))
                            .color(Color::from_hex(TEXT_PRIMARY)),
                        Space(Modifier::new().width(Dp(12.0))),
                        source_badge(&summary),
                        if summary.installed {
                            installed_badge()
                        } else {
                            Box(Modifier::new())
                        },
                    )),
                    Space(Modifier::new().height(Dp(6.0))),
                    Text(summary.description.clone())
                        .size(FONT_BASE)
                        .color(Color::from_hex(TEXT_SECONDARY))
                        .max_lines(8)
                        .overflow_ellipsize(),
                    Space(Modifier::new().height(Dp(4.0))),
                    Text(format!("v{}", summary.version.trim_start_matches('v')))
                        .size(FONT_SM)
                        .color(Color::from_hex(TEXT_DIMMED)),
                )),
            ),
            Space(Modifier::new().height(Dp(16.0))),
            HorizontalDivider(DividerConfig {
                color: Color::from_hex(CARD_BORDER),
                ..Default::default()
            }),
            Space(Modifier::new().height(Dp(12.0))),
            detail_section,
        )),
    )
}

fn details_body(det: &PackageDetails) -> View {
    let mut rows: Vec<View> = Vec::new();

    let long_desc = det.description.as_deref().unwrap_or("");
    if !long_desc.is_empty() && long_desc != det.summary.description {
        rows.push(
            Text(long_desc.to_string())
                .size(FONT_BASE)
                .color(Color::from_hex(TEXT_SECONDARY))
                .modifier(Modifier::new().margin_vertical(Dp(8.0))),
        );
    }

    let mut info_rows: Vec<View> = Vec::new();
    if let Some(ref h) = det.homepage {
        if !h.is_empty() {
            info_rows.push(detail_row("Homepage", h));
        }
    }
    if let Some(ref l) = det.license {
        info_rows.push(detail_row("License", l));
    }
    if let Some(ref d) = det.developer {
        info_rows.push(detail_row("Developer", d));
    }
    if let Some(ref m) = det.maintainer {
        info_rows.push(detail_row("Maintainer", m));
    }
    if let Some(s) = det.size_install {
        info_rows.push(detail_row("Install size", &format_bytes(s)));
    }
    if let Some(s) = det.size_download {
        info_rows.push(detail_row("Download size", &format_bytes(s)));
    }
    if !info_rows.is_empty() {
        rows.push(Card(
            CardConfig {
                container_color: Color::from_hex(CARD_BG),
                border: Some((Dp(1.0), Color::from_hex(CARD_BORDER))),
                shape_radius: R_MD,
                ..Default::default()
            },
            || Column(Modifier::new().fill_max_width().padding(Dp(12.0))).child(info_rows),
        ));
    }

    if !det.depends.is_empty() {
        rows.push(tag_list("Dependencies", &det.depends));
    }
    if !det.opt_depends.is_empty() {
        rows.push(tag_list("Optional deps", &det.opt_depends));
    }

    Column(Modifier::new().fill_max_width()).child(rows)
}

fn status_bar(store: &Rc<Store>, s: &AppState) -> View {
    let last = s.progress_log.lines().last();
    let stage_label = s.active_stage.map(|st| format!("{:?}", st));

    let indicator = if let Some(stage) = &stage_label {
        Row(Modifier::new()
            .fill_max_width()
            .align_items(AlignItems::CENTER))
        .child((
            Text(stage.as_str())
                .size(FONT_XS)
                .color(Color::from_hex(INDIGO))
                .modifier(Modifier::new().padding(Dp(4.0)).width(Dp(100.0))),
            if let Some(pct) = s.progress_pct {
                LinearProgressIndicator(
                    Some(pct),
                    LinearProgressIndicatorConfig {
                        color: Color::from_hex(BLUE_BORDER),
                        track_color: Color::from_hex(CARD_BORDER),
                        ..Default::default()
                    },
                )
            } else {
                LinearProgressIndicator(
                    None,
                    LinearProgressIndicatorConfig {
                        color: Color::from_hex(INDIGO),
                        track_color: Color::from_hex(CARD_BORDER),
                        ..Default::default()
                    },
                )
            },
            Space(Modifier::new().width(Dp(4.0))),
            if let Some(pct) = s.progress_pct {
                Text(format!("{:.0}%", pct * 100.0))
                    .size(FONT_XS)
                    .color(Color::from_hex(TEXT_DIMMED))
                    .modifier(Modifier::new().width(Dp(40.0)))
            } else {
                Box(Modifier::new().width(Dp(40.0)))
            },
        ))
    } else {
        Box(Modifier::new().height(Dp(6.0)))
    };

    Column(Modifier::new().fill_max_width().margin_vertical(Dp(4.0))).child((
        indicator,
        Card(
            CardConfig {
                container_color: Color::from_hex(CARD_BG),
                border: Some((Dp(1.0), Color::from_hex(CARD_BORDER))),
                shape_radius: R_MD,
                ..Default::default()
            },
            || {
                Row(Modifier::new()
                    .fill_max_width()
                    .padding_values(PaddingValues {
                        left: Dp(10.0),
                        right: Dp(10.0),
                        top: Dp(10.0),
                        bottom: Dp(10.0),
                    }))
                .child((
                    if let Some(last) = last {
                        Text(last.to_string())
                            .size(FONT_SM)
                            .color(Color::from_hex(TEXT_MUTED))
                            .modifier(Modifier::new().flex_grow(1.0).align_self_center())
                    } else {
                        Box(Modifier::new().flex_grow(1.0))
                    },
                    secondary_button(
                        if s.log_expanded {
                            "Hide log"
                        } else {
                            "Show log"
                        },
                        {
                            let store = store.clone();
                            move || store.dispatch(Action::ToggleLog)
                        },
                    ),
                ))
            },
        ),
    ))
}
fn log_panel(s: &AppState) -> View {
    if !s.log_expanded {
        return Box(Modifier::new());
    }

    let scroll = remember_scroll_state("log_panel");

    ScrollArea(
        Modifier::new()
            .fill_max_width()
            .height(Dp(LOG_HEIGHT_DP))
            .flex_shrink(0.0)
            .padding(Dp(12.0))
            .margin_vertical(Dp(4.0))
            .background(Color::from_hex(CARD_BG))
            .border(Dp(1.0), Color::from_hex(CARD_BORDER), R_MD)
            .clip_rounded(R_MD),
        scroll,
        Column(Modifier::new().fill_max_width()).child(
            Text(s.progress_log.clone())
                .size(Sp(12.0))
                .color(Color::from_hex(LOG_TEXT))
                .modifier(Modifier::new().fill_max_width()),
        ),
    )
}

fn settings_view(store: Rc<Store>, s: &AppState) -> View {
    let settings = &s.settings;

    let section = |title: &str| {
        Text(title.to_string())
            .size(FONT_SM)
            .color(Color::from_hex(TEXT_DIMMED))
            .modifier(Modifier::new().padding_values(PaddingValues {
                left: Dp(4.0),
                right: Dp(0.0),
                top: Dp(16.0),
                bottom: Dp(8.0),
            }))
    };

    let section_gap = || Space(Modifier::new().height(Dp(4.0)));

    let store_for_backend = store.clone();
    let backend_item =
        move |label: &str, desc: &str, badge: View, enabled: bool, backend_key: &str| {
            let key = backend_key.to_string();

            ListItem(
                label,
                Some(desc.into()),
                None,
                Some(badge),
                Some(Switch(
                    enabled,
                    {
                        let s = store_for_backend.clone();
                        let k = key.clone();
                        move |v| {
                            s.dispatch(Action::SetBackendEnabled {
                                backend: k.clone(),
                                enabled: v,
                            })
                        }
                    },
                    SwitchConfig {
                        checked_track_color: Color::from_hex(SEL_BG),
                        unchecked_track_color: Color::from_hex(CARD_BORDER),
                        checked_thumb_color: Color::from_hex(TEXT_PRIMARY),
                        unchecked_thumb_color: Color::from_hex(TEXT_MUTED),
                        ..Default::default()
                    },
                )),
                Some(Rc::new({
                    let s = store_for_backend.clone();
                    let k = key;
                    move || {
                        s.dispatch(Action::SetBackendEnabled {
                            backend: k.clone(),
                            enabled: !enabled,
                        })
                    }
                })),
                None,
                ListItemConfig {
                    shape_radius: R_MD,
                    ..Default::default()
                },
            )
        };

    let scroll = remember_scroll_state("settings_scroll");
    scroll.set_show_scrollbar(false);
    let store_clone = store.clone();

    let upgrade_dialog_state = remember(DialogState::new);

    ScrollArea(
        Modifier::new().fill_max_size().padding(Dp(24.0)),
        scroll,
        Column(Modifier::new().fill_max_size().align_self_center()).with_children(vec![
            Row(Modifier::new()
                .fill_max_width()
                .align_items(AlignItems::CENTER))
            .child((
                secondary_icon_button(Symbols::CHEVRON_LEFT, "Back", move || {
                    if let Some(ref nav) = *store_clone.navigator.borrow() {
                        nav.pop();
                    }
                }),
                Spacer(),
            )),
            Space(Modifier::new().height(Dp(8.0))),
            section("Backends"),
            backend_item(
                "Repo packages",
                "Packages from Arch Linux repositories (core, extra, multilib)",
                repo_badge(),
                settings.enable_repo,
                "repo",
            ),
            section_gap(),
            backend_item(
                "AUR packages",
                "Community packages from the Arch User Repository",
                aur_badge(),
                settings.enable_aur,
                "aur",
            ),
            backend_item(
                "Flatpak packages",
                "Sandboxed packages from Flathub and other remotes",
                flatpak_badge(),
                settings.enable_flatpak,
                "flatpak",
            ),
            section_gap(),
            backend_item(
                "AppImage packages",
                "Portable self-contained Linux applications",
                appimage_badge(),
                settings.enable_appimage,
                "appimage",
            ),
            Space(Modifier::new().height(Dp(8.0))),
            {
                let d = upgrade_dialog_state.clone();
                ListItem(
                    "Upgrade all sources",
                    Some("Choose which backends to include when upgrading all packages".into()),
                    None,
                    None,
                    None,
                    Some(Rc::new(move || d.show())),
                    None,
                    ListItemConfig {
                        shape_radius: R_MD,
                        ..Default::default()
                    },
                )
            },
            Dialog(
                upgrade_dialog_state.clone(),
                Modifier::new(),
                DialogProperties::default(),
                Column(Modifier::new().padding(Dp(20.0)).min_width(Dp(320.0))).with_children(vec![
                    Text("Upgrade all sources".to_string())
                        .size(FONT_LG)
                        .font_weight(FontWeight::BOLD)
                        .color(Color::from_hex(TEXT_PRIMARY)),
                    Space(Modifier::new().height(Dp(4.0))),
                    Text("Select backends to include")
                        .size(FONT_XS)
                        .color(Color::from_hex(TEXT_MUTED)),
                    Space(Modifier::new().height(Dp(16.0))),
                    upgrade_checkbox("Repository (pacman)", "repo", settings.upgrade_repo, &store),
                    upgrade_checkbox("AUR", "aur", settings.upgrade_aur, &store),
                    upgrade_checkbox("Flatpak", "flatpak", settings.upgrade_flatpak, &store),
                    upgrade_checkbox("AppImage", "appimage", settings.upgrade_appimage, &store),
                    Space(Modifier::new().height(Dp(20.0))),
                    Row(Modifier::new().fill_max_width()).child((Spacer(), {
                        let d = upgrade_dialog_state.clone();
                        Button(
                            Modifier::new(),
                            move || d.dismiss(),
                            ButtonConfig {
                                container_color: Some(Color::from_hex(SEL_BG)),
                                content_color: Some(Color::from_hex(TEXT_PRIMARY)),
                                ..Default::default()
                            },
                            || Text("Done".to_string()),
                        )
                    })),
                ]),
            ),
            Space(Modifier::new().height(Dp(12.0))),
            section("Info"),
            Card(
                CardConfig {
                    container_color: Color::from_hex(CARD_BG),
                    border: Some((Dp(1.0), Color::from_hex(CARD_BORDER))),
                    shape_radius: R_MD,
                    ..Default::default()
                },
                || {
                    Row(Modifier::new().fill_max_width().padding(Dp(12.0))).child(
                        Text("Disabled backends take effect after restart.")
                            .size(FONT_XS)
                            .color(Color::from_hex(TEXT_DIMMED)),
                    )
                },
            ),
            Space(Modifier::new().height(Dp(40.0))),
        ]),
    )
}

fn upgrade_checkbox(label: &str, backend_key: &str, enabled: bool, store: &Rc<Store>) -> View {
    let key = backend_key.to_string();
    let store_cb = store.clone();
    Row(Modifier::new()
        .fill_max_width()
        .align_items(AlignItems::CENTER))
    .child((
        Checkbox(
            enabled,
            move |v| {
                store_cb.dispatch(Action::SetUpgradeAllSource {
                    backend: key.clone(),
                    enabled: v,
                })
            },
            CheckboxConfig {
                checked_color: Color::from_hex(SEL_BG),
                unchecked_color: Color::from_hex(CARD_BORDER),
                checkmark_color: Color::from_hex(TEXT_PRIMARY),
                checked_border_color: Color::from_hex(SEL_BG),
                unchecked_border_color: Color::from_hex(TEXT_MUTED),
                state_colors: StateColors {
                    default: Color::TRANSPARENT,
                    hovered: Color::from_hex(SEL_BG).with_alpha_f32(0.15),
                    focused: Color::from_hex(SEL_BG).with_alpha_f32(0.15),
                    pressed: Color::from_hex(SEL_BG).with_alpha_f32(0.25),
                    disabled: Color::TRANSPARENT,
                    dragged: Color::from_hex(SEL_BG).with_alpha_f32(0.25),
                },
                ..Default::default()
            },
        ),
        Text(label.to_string())
            .size(FONT_SM)
            .color(Color::from_hex(TEXT_PRIMARY)),
    ))
}

fn home_view(store: Rc<Store>) -> View {
    let s = store.state.get();
    Column(Modifier::new().fill_max_size()).child((
        top_bar(&store, &s),
        HorizontalDivider(DividerConfig {
            color: Color::from_hex(CARD_BORDER),
            ..Default::default()
        }),
        Column(
            Modifier::new()
                .fill_max_width()
                .flex_grow(1.0)
                .padding_values(PaddingValues {
                    left: Dp(16.0),
                    right: Dp(16.0),
                    top: Dp(0.0),
                    bottom: Dp(0.0),
                }),
        )
        .child((
            search_section(&store, &s),
            Space(Modifier::new().height(Dp(8.0))),
            results_list(&store, &s),
            status_bar(&store, &s),
            log_panel(&s),
        )),
    ))
}

pub fn root_view(store: Rc<Store>) -> View {
    let stack = remember_back_stack(Route::Home);
    *store.navigator.borrow_mut() = Some(Navigator {
        stack: (*stack).clone(),
    });

    Surface(
        SurfaceConfig {
            modifier: Modifier::new()
                .fill_max_size()
                .background(Color::from_hex(BG)),
            ..Default::default()
        },
        || {
            NavDisplay(
                stack.clone(),
                renderer(move |scope: &EntryScope<Route>| match scope.key() {
                    Route::Home => home_view(store.clone()),
                    Route::Detail(_) => {
                        let s = store.state.get();
                        detail_overlay(store.clone(), &s)
                    }
                    Route::Settings => {
                        let s = store.state.get();
                        settings_view(store.clone(), &s)
                    }
                }),
                None,
                NavTransition::default(),
            )
        },
    )
}
