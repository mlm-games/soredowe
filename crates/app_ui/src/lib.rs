use crate::state::{Action, AppState, Route, SortMode, Store};
use crate::theme::*;
use crate::widgets::*;
use domain::config::ThemeMode;
use domain::{PackageDetails, PackageSummary, Stage};
use repose_core::*;
use repose_material::material3::dialog::{Dialog, DialogProperties, DialogState};
use repose_material::material3::{
    Card, CardConfig, Checkbox, CheckboxConfig, CircularProgressIndicator,
    CircularProgressIndicatorConfig, DividerConfig, DropdownMenu, DropdownMenuConfig,
    DropdownMenuEntry, DropdownMenuItem, HorizontalDivider, LinearProgressIndicator,
    LinearProgressIndicatorConfig, MenuState, NavItem, NavRailItem, NavigationBar,
    NavigationBarConfig, NavigationRail, NavigationRailConfig, SegmentConfig, SegmentedButton,
    SegmentedButtonConfig, Surface, SurfaceConfig, Switch, SwitchConfig, VerticalDivider,
};
use repose_material::{Icon, Symbol, material_symbols};
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

material_symbols! {
    APPS: '\u{e5c3}',
    CHECK: '\u{e5ca}',
    CHECK_CIRCLE: '\u{e86c}',
    CHEVRON_LEFT: '\u{e5cb}',
    CLOSE: '\u{e5cd}',
    DARK_MODE: '\u{e51c}',
    FILE_OPEN: '\u{eaf3}',
    HOME: '\u{e88a}',
    INFO: '\u{e88e}',
    INVENTORY: '\u{e1a1}',
    KEYBOARD_ARROW_DOWN: '\u{e313}',
    LIGHT_MODE: '\u{e518}',
    PACKAGE: '\u{e48f}',
    PACKAGE_2: '\u{f569}',
    REFRESH: '\u{e5d5}',
    SEARCH: '\u{e8b6}',
    SEARCH_OFF: '\u{ea76}',
    SETTINGS: '\u{e8b8}',
    SORT: '\u{e164}',
    SYSTEM_UPDATE_ALT: '\u{e8d7}',
    TERMINAL: '\u{eb8e}',
}

pub mod state;
pub mod theme;
pub mod widgets;

fn page_padding() -> Dp {
    match window_size_class().width {
        WidthClass::Compact => SPACE_4,
        WidthClass::Medium => SPACE_6,
        WidthClass::Expanded => SPACE_8,
    }
}

fn compact_layout() -> bool {
    !window_size_class().is_at_least_medium_width()
}

fn page_header(
    title: &str,
    subtitle: Option<&str>,
    navigation: Option<View>,
    actions: Vec<View>,
) -> View {
    let compact = compact_layout();
    let title_content = Column(Modifier::new().weight(1.0).min_width(Dp(0.0))).child((
        Text(title.to_string())
            .size(if compact { FONT_MD } else { FONT_LG })
            .line_height(if compact { LINE_BASE } else { Sp(24.0) })
            .font_weight(FontWeight::SEMI_BOLD)
            .color(text_primary())
            .single_line()
            .overflow_ellipsize(),
        if !compact {
            subtitle
                .map(|text| {
                    Text(text.to_string())
                        .size(FONT_SM)
                        .color(text_muted())
                        .single_line()
                        .overflow_ellipsize()
                })
                .unwrap_or(Box(Modifier::new()))
        } else {
            Box(Modifier::new())
        },
    ));
    let navigation = navigation
        .map(|view| {
            Box(Modifier::new()
                .flex_shrink(0.0)
                .padding_values(PaddingValues {
                    left: Dp(0.0),
                    right: SPACE_4,
                    top: Dp(0.0),
                    bottom: Dp(0.0),
                }))
            .child(view)
        })
        .unwrap_or(Box(Modifier::new()));
    let actions = Row(Modifier::new().flex_shrink(0.0).gap(SPACE_2)).child(actions);

    Row(Modifier::new()
        .fill_max_width()
        .height(HEADER_HEIGHT)
        .padding_values(PaddingValues {
            left: if compact { SPACE_3 } else { SPACE_5 },
            right: if compact { SPACE_3 } else { SPACE_5 },
            top: Dp(0.0),
            bottom: Dp(0.0),
        })
        .background(surface())
        .align_items(AlignItems::CENTER))
    .child((navigation, title_content, actions))
}

fn home_header(store: &Rc<Store>, s: &AppState) -> View {
    let compact = compact_layout();
    let mut actions = Vec::new();

    if s.in_upgrades_view {
        if !s.results.is_empty() {
            actions.push(primary_button("Update all", {
                let store = store.clone();
                move || store.dispatch(Action::UpgradeAll)
            }));
        }
    } else if compact {
        actions.push(tooltip_icon_button(
            Symbols::FILE_OPEN,
            "Install local file",
            {
                let store = store.clone();
                move || store.dispatch(Action::PickFile)
            },
            false,
        ));
    } else {
        actions.push(secondary_icon_button(Symbols::FILE_OPEN, "Install file", {
            let store = store.clone();
            move || store.dispatch(Action::PickFile)
        }));
    }

    actions.push(tooltip_icon_button(
        Symbols::REFRESH,
        "Refresh package data",
        {
            let store = store.clone();
            move || store.dispatch(Action::Refresh)
        },
        false,
    ));

    if s.in_upgrades_view {
        page_header(
            "Updates",
            Some("Review available changes before applying them."),
            None,
            actions,
        )
    } else {
        page_header(
            "Discover packages",
            Some("Search every enabled source from one place."),
            None,
            actions,
        )
    }
}

fn search_section(store: &Rc<Store>, s: &AppState) -> View {
    let search_state = remember_state_with_key("package_search", || {
        TextFieldState::with_text(s.query.clone())
    });
    let initial = s.query.clone();

    Column(Modifier::new().fill_max_width()).child(
        Row(Modifier::new()
            .fill_max_width()
            .align_items(AlignItems::CENTER))
        .child((
            Row(Modifier::new()
                .flex_grow(1.0)
                .height(SEARCH_HEIGHT)
                .background(surface())
                .border(Dp(1.0), card_border(), R_MD)
                .clip_rounded(R_MD)
                .align_items(AlignItems::CENTER)
                .state_colors(StateColors {
                    default: Color::TRANSPARENT,
                    hovered: surface_alt().with_alpha_f32(0.45),
                    focused: surface_alt().with_alpha_f32(0.65),
                    pressed: surface_alt().with_alpha_f32(0.65),
                    disabled: Color::TRANSPARENT,
                    dragged: surface_alt().with_alpha_f32(0.65),
                })
                .padding_values(PaddingValues {
                    left: SPACE_3,
                    right: SPACE_3,
                    top: Dp(0.0),
                    bottom: Dp(0.0),
                }))
            .child((
                Icon(Symbols::SEARCH)
                    .size(Sp(20.0))
                    .color(text_muted())
                    .single_line(),
                Box(Modifier::new().width(SPACE_2)),
                repose_ui::BasicTextField(
                    search_state,
                    Modifier::new()
                        .flex_grow(1.0)
                        .height(SEARCH_HEIGHT)
                        .semantics(
                            Semantics::new(Role::TextField)
                                .with_label("Search packages")
                                .with_value(initial),
                        ),
                    "Name, description, or repository",
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
                ),
            )),
            Box(Modifier::new().width(SPACE_2)),
            search_button({
                let store = store.clone();
                move || store.dispatch(Action::Search)
            }),
        )),
    )
}

fn package_filter_chip(label: &'static str, selected: bool, on_click: impl Fn() + 'static) -> View {
    let background = if selected { accent_bg() } else { surface() };
    let foreground = if selected { accent() } else { text_secondary() };
    let border = if selected {
        accent_border()
    } else {
        card_border()
    };
    Box(Modifier::new()
        .min_height(Dp(40.0))
        .padding_values(PaddingValues {
            left: SPACE_3,
            right: SPACE_3,
            top: Dp(0.0),
            bottom: Dp(0.0),
        })
        .background(background)
        .state_colors(StateColors {
            default: Color::TRANSPARENT,
            hovered: surface_hover(),
            focused: surface_hover(),
            pressed: surface_alt(),
            disabled: Color::TRANSPARENT,
            dragged: surface_hover(),
        })
        .border(Dp(1.0), border, R_SM)
        .clip_rounded(R_SM)
        .semantics(
            Semantics::new(Role::Checkbox)
                .with_label(label)
                .with_checked(selected),
        )
        .clickable()
        .on_click(on_click)
        .align_items(AlignItems::CENTER))
    .child(
        Row(Modifier::new().align_items(AlignItems::CENTER).gap(SPACE_1)).child((
            if selected {
                Icon(Symbols::CHECK)
                    .size(Sp(15.0))
                    .color(foreground)
                    .single_line()
            } else {
                Box(Modifier::new().width(Dp(15.0)))
            },
            Text(label)
                .size(FONT_SM)
                .font_weight(FontWeight::MEDIUM)
                .color(foreground)
                .single_line(),
        )),
    )
}

fn sort_item(
    label: &'static str,
    mode: SortMode,
    selected: SortMode,
    store: Rc<Store>,
) -> DropdownMenuEntry {
    let mut item = DropdownMenuItem::new(label, move || {
        store.dispatch(Action::SetSort(mode));
    });
    if selected == mode {
        item = item.trailing_icon(
            Icon(Symbols::CHECK)
                .size(Sp(18.0))
                .color(accent())
                .single_line(),
        );
    }
    DropdownMenuEntry::Item(item)
}

fn sort_menu(store: &Rc<Store>, s: &AppState) -> View {
    let state = remember(MenuState::new);
    let label = match s.sort {
        SortMode::Popularity => "Popular",
        SortMode::NameAsc => "A–Z",
        SortMode::NameDesc => "Z–A",
    };
    let menu_state = state.clone();
    let trigger = secondary_icon_button(Symbols::SORT, label, move || menu_state.open());
    let entries = vec![
        sort_item("Popular", SortMode::Popularity, s.sort, store.clone()),
        sort_item("Name A–Z", SortMode::NameAsc, s.sort, store.clone()),
        sort_item("Name Z–A", SortMode::NameDesc, s.sort, store.clone()),
    ];
    DropdownMenu(
        state,
        Modifier::new().align_self(AlignSelf::FLEX_START),
        trigger,
        entries,
        DropdownMenuConfig {
            container_color: surface(),
            item_text_color: text_secondary(),
            disabled_item_text_color: text_dimmed(),
            divider_color: card_border(),
            min_width: Dp(176.0),
            item_height: Dp(42.0),
            max_width: Dp(220.0),
            border: Some((Dp(1.0), card_border(), R_MD)),
            shape_radius: Some(R_MD),
            tonal_elevation: Dp(6.0),
            ..Default::default()
        },
    )
}

fn filter_bar(store: &Rc<Store>, s: &AppState) -> View {
    let chips = Row(Modifier::new()
        .fill_max_width()
        .gap(SPACE_2)
        .flex_wrap(FlexWrap::Wrap))
    .child(vec![
        package_filter_chip("Repo", s.filter_repo, {
            let store = store.clone();
            move || store.dispatch(Action::ToggleFilterRepo)
        }),
        package_filter_chip("AUR", s.filter_aur, {
            let store = store.clone();
            move || store.dispatch(Action::ToggleFilterAur)
        }),
        package_filter_chip("Flatpak", s.filter_flatpak, {
            let store = store.clone();
            move || store.dispatch(Action::ToggleFilterFlatpak)
        }),
        package_filter_chip("AppImage", s.filter_appimage, {
            let store = store.clone();
            move || store.dispatch(Action::ToggleFilterAppImage)
        }),
        package_filter_chip("Installed", s.filter_installed, {
            let store = store.clone();
            move || store.dispatch(Action::ToggleFilterInstalled)
        }),
    ]);

    if compact_layout() {
        Column(Modifier::new().fill_max_width().gap(SPACE_2)).child((
            Text("Sources")
                .size(FONT_SM)
                .font_weight(FontWeight::SEMI_BOLD)
                .color(text_secondary())
                .single_line(),
            chips,
            sort_menu(store, s),
        ))
    } else {
        Row(Modifier::new()
            .fill_max_width()
            .gap(SPACE_4)
            .align_items(AlignItems::FLEX_START))
        .child((
            Column(Modifier::new().flex_grow(1.0).gap(SPACE_2)).child((
                Text("Sources")
                    .size(FONT_SM)
                    .font_weight(FontWeight::SEMI_BOLD)
                    .color(text_secondary())
                    .single_line(),
                chips,
            )),
            sort_menu(store, s),
        ))
    }
}

fn pkg_action(store: &Rc<Store>, pkg: &PackageSummary, upgrades_mode: bool) -> View {
    let id = pkg.id.clone();
    if upgrades_mode {
        let store = store.clone();
        success_button("Update", move || {
            store.dispatch(Action::Upgrade(id.clone()))
        })
    } else if pkg.installed {
        let store = store.clone();
        danger_button("Remove", move || store.dispatch(Action::Remove(id.clone())))
    } else {
        let store = store.clone();
        success_button("Install", move || {
            store.dispatch(Action::Install(id.clone()))
        })
    }
}

fn pkg_row(store: Rc<Store>, pkg: PackageSummary, selected: bool, upgrades_mode: bool) -> View {
    let background = if selected { accent_bg() } else { surface() };
    let source = pkg.id.source;
    let name = pkg.id.name.clone();
    let description = pkg.description.clone();
    let version = pkg.version.clone();
    let installed = pkg.installed;
    let action_width = if compact_layout() {
        Dp(92.0)
    } else {
        Dp(104.0)
    };

    Row(Modifier::new()
        .fill_max_width()
        .height(Dp(LIST_ITEM_HEIGHT))
        .padding_values(PaddingValues {
            left: SPACE_3,
            right: SPACE_3,
            top: SPACE_2,
            bottom: SPACE_2,
        })
        .background(background)
        .state_colors(StateColors {
            default: Color::TRANSPARENT,
            hovered: surface_hover().with_alpha_f32(0.55),
            focused: surface_hover().with_alpha_f32(0.55),
            pressed: surface_hover(),
            disabled: Color::TRANSPARENT,
            dragged: surface_hover(),
        })
        .border(
            Dp(1.0),
            if selected {
                accent_border()
            } else {
                card_border()
            },
            R_SM,
        )
        .clip_rounded(R_SM)
        .clickable()
        .on_click({
            let store = store.clone();
            let id = pkg.id.clone();
            move || store.dispatch(Action::Select(id.clone()))
        })
        .align_items(AlignItems::CENTER))
    .child((
        source_icon(source, Dp(40.0)).modifier(Modifier::new().flex_shrink(0.0)),
        Box(Modifier::new().width(SPACE_3).flex_shrink(0.0)),
        Column(Modifier::new().weight(1.0).min_width(Dp(0.0)).gap(Dp(3.0))).child((
            Text(name)
                .size(FONT_BASE)
                .line_height(LINE_BASE)
                .font_weight(FontWeight::SEMI_BOLD)
                .color(text_primary())
                .single_line()
                .overflow_ellipsize()
                .modifier(Modifier::new().fill_max_width()),
            Text(description)
                .size(FONT_SM)
                .line_height(Sp(17.0))
                .color(text_muted())
                .max_lines(2)
                .overflow_ellipsize()
                .modifier(Modifier::new().fill_max_width().min_width(Dp(0.0))),
            Row(Modifier::new()
                .min_width(Dp(0.0))
                .align_items(AlignItems::CENTER)
                .gap(SPACE_2))
            .child((
                source_badge(&pkg),
                Text(format!("v{}", version.trim_start_matches('v')))
                    .size(FONT_XS)
                    .font_family(FONT_MONO)
                    .color(text_dimmed())
                    .single_line()
                    .overflow_ellipsize()
                    .modifier(Modifier::new().flex_shrink(1.0).min_width(Dp(0.0))),
            )),
        )),
        Box(Modifier::new().width(SPACE_3).flex_shrink(0.0)),
        Column(
            Modifier::new()
                .width(action_width)
                .flex_shrink(0.0)
                .align_self_center()
                .align_items(AlignItems::CENTER)
                .gap(Dp(4.0)),
        )
        .child((
            if installed {
                installed_badge()
            } else {
                Box(Modifier::new())
            },
            pkg_action(&store, &pkg, upgrades_mode),
        )),
    ))
}

fn results_heading(s: &AppState) -> View {
    let count = s.results.len();
    let title = if s.in_upgrades_view {
        if count == 1 {
            "1 update available".to_string()
        } else {
            format!("{count} updates available")
        }
    } else if s.query.trim().is_empty() {
        if count == 1 {
            "1 package".to_string()
        } else {
            format!("{count} packages")
        }
    } else {
        format!("{count} matches for “{}”", s.query.trim())
    };

    Row(Modifier::new()
        .fill_max_width()
        .align_items(AlignItems::CENTER))
    .child((
        Text(title)
            .size(FONT_SM)
            .font_weight(FontWeight::SEMI_BOLD)
            .color(text_secondary())
            .single_line()
            .overflow_ellipsize()
            .modifier(Modifier::new().flex_grow(1.0)),
        Text(match s.sort {
            SortMode::Popularity => "Popular",
            SortMode::NameAsc => "A–Z",
            SortMode::NameDesc => "Z–A",
        })
        .size(FONT_XS)
        .font_family(FONT_MONO)
        .color(text_dimmed())
        .single_line(),
    ))
}

fn empty_results(store: &Rc<Store>, s: &AppState) -> View {
    let (symbol, title, body, action) = if s.in_upgrades_view {
        (
            Symbols::CHECK_CIRCLE,
            "You’re up to date",
            "Soredowe checked the enabled package sources and found no available updates.",
            None,
        )
    } else if s.query.trim().is_empty() {
        (
            Symbols::SEARCH,
            "Search your software sources",
            "Look for packages across enabled repositories, AUR, Flatpak, and AppImage sources.",
            Some(secondary_icon_button(
                Symbols::FILE_OPEN,
                "Install local file",
                {
                    let store = store.clone();
                    move || store.dispatch(Action::PickFile)
                },
            )),
        )
    } else {
        (
            Symbols::SEARCH_OFF,
            "No matching packages",
            "Try a shorter package name or enable another source.",
            None,
        )
    };

    Box(Modifier::new()
        .fill_max_width()
        .weight(1.0)
        .background(surface())
        .border(Dp(1.0), card_border(), R_MD)
        .clip_rounded(R_MD))
    .child(empty_state(symbol, title, body, action))
}

fn results_list(store: &Rc<Store>, s: &AppState) -> View {
    if s.results.is_empty() {
        return empty_results(store, s);
    }

    let store_for_rows = store.clone();
    let selected = s.selected.clone();
    let upgrades_mode = s.in_upgrades_view;
    let results = s.results.clone();

    LazyColumn(
        results,
        LIST_ITEM_HEIGHT,
        |pkg: &PackageSummary| {
            let mut hash = 17_u64;
            for byte in pkg.id.name.bytes() {
                hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
            }
            hash = hash.wrapping_mul(31).wrapping_add(pkg.id.source as u64);
            if let Some(repo) = pkg.id.repo.as_ref() {
                for byte in repo.bytes() {
                    hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
                }
            }
            hash
        },
        move |pkg: PackageSummary, _| {
            let is_selected = selected.as_ref().is_some_and(|id| *id == pkg.id);
            pkg_row(store_for_rows.clone(), pkg, is_selected, upgrades_mode)
        },
        LazyColumnConfig {
            modifier: Modifier::new()
                .fill_max_width()
                .weight(1.0)
                .min_width(Dp(0.0))
                .min_height(Dp(0.0))
                .padding(SPACE_1)
                .background(bg())
                .clip_rounded(R_MD),
            state: remember_with_key("pkg_scroll", LazyColumnState::new),
            animate_spec: None::<repose_core::animation::AnimationSpec>,
            ..Default::default()
        },
    )
}

fn stage_name(stage: Stage) -> &'static str {
    match stage {
        Stage::Queued => "Queued",
        Stage::Refreshing => "Refreshing package data",
        Stage::Searching => "Searching sources",
        Stage::Resolving => "Resolving dependencies",
        Stage::Downloading => "Downloading packages",
        Stage::Building => "Building packages",
        Stage::Installing => "Installing packages",
        Stage::Removing => "Removing packages",
        Stage::Verifying => "Verifying changes",
        Stage::Cleaning => "Cleaning up",
        Stage::Finished => "Finished",
        Stage::Failed => "Failed",
    }
}

fn log_panel(s: &AppState) -> View {
    if !s.log_expanded {
        return Box(Modifier::new());
    }
    let scroll = remember_scroll_state("log_panel");
    ScrollArea(
        Modifier::new()
            .fill_max_width()
            .height(LOG_HEIGHT)
            .flex_shrink(0.0)
            .padding(SPACE_4)
            .background(log_bg())
            .clip_rounded(R_MD),
        scroll,
        Text(s.progress_log.clone())
            .size(FONT_SM)
            .line_height(Sp(18.0))
            .font_family(FONT_MONO)
            .color(log_text())
            .modifier(Modifier::new().fill_max_width()),
    )
}

fn activity_panel(store: &Rc<Store>, s: &AppState) -> View {
    let has_log = !s.progress_log.trim().is_empty();
    let body = if let Some(stage) = s.active_stage {
        Row(Modifier::new()
            .fill_max_width()
            .height(STATUS_HEIGHT)
            .padding_values(PaddingValues {
                left: SPACE_4,
                right: SPACE_3,
                top: SPACE_2,
                bottom: SPACE_2,
            })
            .align_items(AlignItems::CENTER)
            .gap(SPACE_3))
        .child((
            CircularProgressIndicator(
                None,
                CircularProgressIndicatorConfig {
                    modifier: Modifier::new().size(Dp(24.0), Dp(24.0)),
                    color: accent(),
                    track_color: accent_bg(),
                    ..Default::default()
                },
            ),
            Column(Modifier::new().flex_grow(1.0).gap(Dp(3.0))).child((
                Row(Modifier::new().align_items(AlignItems::CENTER).gap(SPACE_2)).child((
                    Text(stage_name(stage))
                        .size(FONT_SM)
                        .font_weight(FontWeight::SEMI_BOLD)
                        .color(text_primary())
                        .single_line(),
                    if let Some(percent) = s.progress_pct {
                        Text(format!("{:.0}%", percent * 100.0))
                            .size(FONT_XS)
                            .font_family(FONT_MONO)
                            .color(accent())
                            .single_line()
                    } else {
                        Box(Modifier::new())
                    },
                )),
                if let Some(last) = s.progress_log.lines().last() {
                    Text(last.to_string())
                        .size(FONT_XS)
                        .color(text_muted())
                        .single_line()
                        .overflow_ellipsize()
                } else {
                    Text("Working…")
                        .size(FONT_XS)
                        .color(text_muted())
                        .single_line()
                },
            )),
            Box(Modifier::new().flex_grow(1.0)).child(LinearProgressIndicator(
                s.progress_pct,
                LinearProgressIndicatorConfig {
                    color: accent(),
                    track_color: accent_bg(),
                    ..Default::default()
                },
            )),
            if has_log {
                quiet_button("Hide log", {
                    let store = store.clone();
                    move || store.dispatch(Action::ToggleLog)
                })
            } else {
                Box(Modifier::new())
            },
        ))
    } else {
        Row(Modifier::new()
            .fill_max_width()
            .height(Dp(48.0))
            .padding_values(PaddingValues {
                left: SPACE_4,
                right: SPACE_3,
                top: Dp(0.0),
                bottom: Dp(0.0),
            })
            .align_items(AlignItems::CENTER)
            .gap(SPACE_2))
        .child((
            Icon(Symbols::CHECK_CIRCLE)
                .size(Sp(18.0))
                .color(success())
                .single_line(),
            Text("Ready")
                .size(FONT_SM)
                .font_weight(FontWeight::MEDIUM)
                .color(text_secondary())
                .single_line(),
            Box(Modifier::new().flex_grow(1.0)),
            if has_log {
                (
                    Text(
                        s.progress_log
                            .lines()
                            .last()
                            .unwrap_or("Activity available")
                            .to_string(),
                    )
                    .size(FONT_XS)
                    .color(text_dimmed())
                    .single_line()
                    .overflow_ellipsize()
                    .modifier(Modifier::new().max_width(Dp(360.0))),
                    quiet_button("Activity", {
                        let store = store.clone();
                        move || store.dispatch(Action::ToggleLog)
                    }),
                )
            } else {
                (Box(Modifier::new()), Box(Modifier::new()))
            },
        ))
    };

    Column(Modifier::new().fill_max_width()).child((
        log_panel(s),
        Box(Modifier::new().height(SPACE_1)),
        Box(Modifier::new()
            .fill_max_width()
            .background(surface())
            .border(
                Dp(1.0),
                card_border(),
                if s.log_expanded { R_MD } else { Dp(0.0) },
            )
            .clip_rounded(R_MD))
        .child(body),
    ))
}

fn home_view(store: Rc<Store>) -> View {
    let s = store.state.get();
    let padding = page_padding();
    Column(Modifier::new().fill_max_size().background(bg())).child((
        home_header(&store, &s),
        HorizontalDivider(DividerConfig {
            color: card_border(),
            thickness: Dp(1.0),
            ..Default::default()
        }),
        Column(
            Modifier::new()
                .fill_max_width()
                .weight(1.0)
                .min_height(Dp(0.0))
                .padding_values(PaddingValues {
                    left: padding,
                    right: padding,
                    top: if s.in_upgrades_view { SPACE_4 } else { SPACE_5 },
                    bottom: SPACE_3,
                })
                .gap(if s.in_upgrades_view { SPACE_3 } else { SPACE_4 }),
        )
        .child((
            if !s.in_upgrades_view {
                search_section(&store, &s)
            } else {
                Box(Modifier::new())
            },
            filter_bar(&store, &s),
            results_heading(&s),
            results_list(&store, &s),
            activity_panel(&store, &s),
        )),
    ))
}

fn metadata_table(rows: Vec<View>) -> View {
    if rows.is_empty() {
        return Box(Modifier::new());
    }
    let mut children = Vec::new();
    for (index, row) in rows.into_iter().enumerate() {
        if index > 0 {
            children.push(HorizontalDivider(DividerConfig {
                color: card_border(),
                thickness: Dp(1.0),
                ..Default::default()
            }));
        }
        children.push(row);
    }
    Card(
        CardConfig {
            container_color: surface(),
            border: Some((Dp(1.0), card_border())),
            shape_radius: R_MD,
            ..Default::default()
        },
        || Column(Modifier::new().fill_max_width()).with_children(children),
    )
}

fn details_body(details: &PackageDetails) -> View {
    let mut children = Vec::new();
    let summary = &details.summary;

    children.push(
        Row(Modifier::new()
            .fill_max_width()
            .align_items(AlignItems::CENTER)
            .gap(SPACE_4))
        .child((
            source_icon(summary.id.source, Dp(58.0)),
            Column(Modifier::new().flex_grow(1.0).gap(SPACE_2)).child((
                Text(summary.id.name.clone())
                    .size(FONT_XL)
                    .line_height(Sp(32.0))
                    .font_weight(FontWeight::SEMI_BOLD)
                    .color(text_primary()),
                Row(Modifier::new().gap(SPACE_2).align_items(AlignItems::CENTER)).child((
                    source_badge(summary),
                    if summary.installed {
                        installed_badge()
                    } else {
                        Box(Modifier::new())
                    },
                    Text(format!("v{}", summary.version.trim_start_matches('v')))
                        .size(FONT_SM)
                        .font_family(FONT_MONO)
                        .color(text_muted())
                        .single_line(),
                )),
            )),
        )),
    );

    children.push(Box(Modifier::new().height(SPACE_5)));
    children.push(
        Text(summary.description.clone())
            .size(FONT_MD)
            .line_height(Sp(25.0))
            .color(text_secondary())
            .modifier(Modifier::new().fill_max_width()),
    );

    let long_description = details.description.as_deref().unwrap_or("");
    if !long_description.is_empty() && long_description != summary.description {
        children.push(Box(Modifier::new().height(SPACE_5)));
        children.push(
            Text(long_description.to_string())
                .size(FONT_BASE)
                .line_height(LINE_BODY)
                .color(text_secondary())
                .modifier(Modifier::new().fill_max_width()),
        );
    }

    let mut info_rows = Vec::new();
    if let Some(homepage) = details
        .homepage
        .as_deref()
        .filter(|value| !value.is_empty())
    {
        info_rows.push(detail_row("Homepage", homepage));
    }
    if let Some(license) = details.license.as_deref().filter(|value| !value.is_empty()) {
        info_rows.push(detail_row("License", license));
    }
    if let Some(developer) = details
        .developer
        .as_deref()
        .filter(|value| !value.is_empty())
    {
        info_rows.push(detail_row("Developer", developer));
    }
    if let Some(maintainer) = details
        .maintainer
        .as_deref()
        .filter(|value| !value.is_empty())
    {
        info_rows.push(detail_row("Maintainer", maintainer));
    }
    if let Some(size) = details.size_install {
        info_rows.push(detail_row("Install size", &format_bytes(size)));
    }
    if let Some(size) = details.size_download {
        info_rows.push(detail_row("Download size", &format_bytes(size)));
    }

    if !info_rows.is_empty() {
        children.push(Box(Modifier::new().height(SPACE_6)));
        children.push(
            Text("Package information")
                .size(FONT_MD)
                .font_weight(FontWeight::SEMI_BOLD)
                .color(text_primary())
                .single_line(),
        );
        children.push(Box(Modifier::new().height(SPACE_2)));
        children.push(metadata_table(info_rows));
    }

    if !details.depends.is_empty() {
        children.push(Box(Modifier::new().height(SPACE_6)));
        children.push(tag_list("Dependencies", &details.depends));
    }
    if !details.opt_depends.is_empty() {
        children.push(Box(Modifier::new().height(SPACE_5)));
        children.push(tag_list("Optional dependencies", &details.opt_depends));
    }

    Column(Modifier::new().fill_max_width()).with_children(children)
}

fn detail_loading() -> View {
    Column(
        Modifier::new()
            .fill_max_width()
            .padding(SPACE_10)
            .align_items(AlignItems::CENTER)
            .gap(SPACE_3),
    )
    .child((
        CircularProgressIndicator(
            None,
            CircularProgressIndicatorConfig {
                modifier: Modifier::new().size(Dp(30.0), Dp(30.0)),
                color: accent(),
                track_color: accent_bg(),
                ..Default::default()
            },
        ),
        Text("Loading package details…")
            .size(FONT_BASE)
            .color(text_muted())
            .single_line(),
    ))
}

fn detail_page(store: Rc<Store>, s: &AppState) -> View {
    let Some(summary) = s
        .selected
        .as_ref()
        .and_then(|selected| s.results.iter().find(|pkg| &pkg.id == selected))
        .cloned()
    else {
        return Box(Modifier::new().fill_max_size().background(bg()));
    };

    let back_store = store.clone();
    let back_action = if compact_layout() {
        tooltip_icon_button(
            Symbols::CHEVRON_LEFT,
            "Packages",
            move || back_store.dispatch(Action::ClearSelection),
            false,
        )
    } else {
        let back_store = store.clone();
        secondary_icon_button(Symbols::CHEVRON_LEFT, "Packages", move || {
            back_store.dispatch(Action::ClearSelection)
        })
    };
    let detail_scroll_key = format!("detail:{}:{:?}", summary.id.name, summary.id.source);
    let scroll = remember_scroll_state(detail_scroll_key);
    scroll.set_show_scrollbar(false);
    let padding = page_padding();
    let body = if let Some(details) = s.detail.as_ref() {
        details_body(details)
    } else {
        detail_loading()
    };

    let detail_pane = Column(Modifier::new().fill_max_size().background(bg())).child((
        page_header(
            if compact_layout() {
                "Details"
            } else {
                "Package details"
            },
            None,
            Some(back_action),
            vec![pkg_action(&store, &summary, s.in_upgrades_view)],
        ),
        HorizontalDivider(DividerConfig {
            color: card_border(),
            thickness: Dp(1.0),
            ..Default::default()
        }),
        Box(Modifier::new().fill_max_width().weight(1.0)).child(ScrollArea(
            Modifier::new().fill_max_size().padding(padding),
            scroll,
            Box(Modifier::new()
                .fill_max_width()
                .max_width(Dp(860.0))
                .align_self_center())
            .child(body),
        )),
    ));

    if window_size_class().is_expanded_width() && !s.results.is_empty() {
        Row(Modifier::new().fill_max_size()).child((
            Column(
                Modifier::new()
                    .width(Dp(360.0))
                    .fill_max_height()
                    .background(bg())
                    .padding_values(PaddingValues {
                        left: SPACE_3,
                        right: SPACE_3,
                        top: SPACE_4,
                        bottom: SPACE_3,
                    })
                    .gap(SPACE_2),
            )
            .child((results_heading(s), results_list(&store, s))),
            VerticalDivider(DividerConfig {
                color: card_border(),
                thickness: Dp(1.0),
                ..Default::default()
            }),
            Box(Modifier::new()
                .weight(1.0)
                .min_width(Dp(0.0))
                .fill_max_height())
            .child(detail_pane),
        ))
    } else {
        detail_pane
    }
}

fn section_heading(title: &str, description: &str) -> View {
    Column(Modifier::new().fill_max_width().gap(SPACE_1)).child((
        Text(title.to_string())
            .size(FONT_MD)
            .line_height(LINE_BASE)
            .font_weight(FontWeight::SEMI_BOLD)
            .color(text_primary())
            .single_line(),
        Text(description.to_string())
            .size(FONT_SM)
            .line_height(Sp(18.0))
            .color(text_muted())
            .modifier(Modifier::new().fill_max_width()),
    ))
}

fn theme_selector(store: Rc<Store>, selected: ThemeMode) -> View {
    let dark_store = store.clone();
    let light_store = store;
    let index = match selected {
        ThemeMode::Dark => 0,
        ThemeMode::Light => 1,
    };
    let selector = SegmentedButton(
        &[index],
        vec![
            SegmentConfig {
                label: "Dark".into(),
                icon: Some(
                    Icon(Symbols::DARK_MODE)
                        .size(Sp(17.0))
                        .color(text_secondary())
                        .single_line(),
                ),
                on_click: Rc::new(move || dark_store.dispatch(Action::SetTheme(ThemeMode::Dark))),
                enabled: true,
                interaction_source: None,
            },
            SegmentConfig {
                label: "Light".into(),
                icon: Some(
                    Icon(Symbols::LIGHT_MODE)
                        .size(Sp(18.0))
                        .color(text_secondary())
                        .single_line(),
                ),
                on_click: Rc::new(move || light_store.dispatch(Action::SetTheme(ThemeMode::Light))),
                enabled: true,
                interaction_source: None,
            },
        ],
        SegmentedButtonConfig {
            selected_container_color: accent_bg(),
            selected_content_color: accent(),
            unselected_content_color: text_muted(),
            border_color: card_border(),
            ..Default::default()
        },
    );
    let copy = Column(Modifier::new().gap(SPACE_1)).child((
        Text("Theme")
            .size(FONT_BASE)
            .font_weight(FontWeight::SEMI_BOLD)
            .color(text_primary())
            .single_line(),
        Text("Dark is used by default. Choose Light for the pale workspace.")
            .size(FONT_SM)
            .line_height(Sp(18.0))
            .color(text_muted()),
    ));
    let content = if compact_layout() {
        Column(Modifier::new().fill_max_width().gap(SPACE_3)).child((copy, selector))
    } else {
        Row(Modifier::new()
            .fill_max_width()
            .align_items(AlignItems::CENTER)
            .gap(SPACE_6))
        .child((Box(Modifier::new().flex_grow(1.0)).child(copy), selector))
    };
    settings_card(vec![Box(Modifier::new().padding(SPACE_4)).child(content)])
}

fn setting_icon(symbol: Symbol) -> View {
    Box(Modifier::new()
        .size(Dp(38.0), Dp(38.0))
        .background(surface_alt())
        .clip_rounded(R_MD)
        .align_items(AlignItems::CENTER)
        .justify_content(JustifyContent::CENTER))
    .child(
        Icon(symbol)
            .size(Sp(19.0))
            .color(text_secondary())
            .single_line(),
    )
}

fn settings_switch_row(
    store: Rc<Store>,
    symbol: Symbol,
    title: &'static str,
    description: &'static str,
    enabled: bool,
    backend: &'static str,
) -> View {
    let switch_store = store.clone();
    let switch_backend = backend.to_string();
    Row(Modifier::new()
        .fill_max_width()
        .min_height(Dp(76.0))
        .padding_values(PaddingValues {
            left: SPACE_4,
            right: SPACE_4,
            top: SPACE_3,
            bottom: SPACE_3,
        })
        .align_items(AlignItems::CENTER)
        .gap(SPACE_3))
    .child((
        setting_icon(symbol),
        Column(Modifier::new().flex_grow(1.0).gap(Dp(2.0))).child((
            Row(Modifier::new().align_items(AlignItems::CENTER).gap(SPACE_2)).child((
                Icon(symbol)
                    .size(Sp(17.0))
                    .color(text_muted())
                    .single_line(),
                Text(title)
                    .size(FONT_BASE)
                    .font_weight(FontWeight::SEMI_BOLD)
                    .color(text_primary())
                    .single_line(),
            )),
            Text(description)
                .size(FONT_SM)
                .line_height(Sp(18.0))
                .color(text_muted())
                .max_lines(2)
                .overflow_ellipsize(),
        )),
        Switch(
            enabled,
            move |value| {
                switch_store.dispatch(Action::SetBackendEnabled {
                    backend: switch_backend.clone(),
                    enabled: value,
                })
            },
            SwitchConfig {
                checked_track_color: accent(),
                unchecked_track_color: card_border(),
                checked_thumb_color: accent_ink(),
                unchecked_thumb_color: surface(),
                ..Default::default()
            },
        ),
    ))
}

fn settings_card(children: Vec<View>) -> View {
    Card(
        CardConfig {
            container_color: surface(),
            border: Some((Dp(1.0), card_border())),
            shape_radius: R_MD,
            ..Default::default()
        },
        || Column(Modifier::new().fill_max_width()).with_children(children),
    )
}

fn settings_notice() -> View {
    Row(Modifier::new()
        .fill_max_width()
        .padding_values(PaddingValues {
            left: SPACE_4,
            right: SPACE_4,
            top: SPACE_3,
            bottom: SPACE_3,
        })
        .background(accent_bg())
        .border(Dp(1.0), accent_border(), R_MD)
        .clip_rounded(R_MD)
        .align_items(AlignItems::CENTER)
        .gap(SPACE_3))
    .child((
        Icon(Symbols::INFO)
            .size(Sp(20.0))
            .color(accent())
            .single_line(),
        Text("Disabled package sources take effect after restarting Soredowe.")
            .size(FONT_SM)
            .line_height(Sp(18.0))
            .color(text_secondary())
            .modifier(Modifier::new().flex_grow(1.0)),
    ))
}

fn upgrade_scope_row(store: Rc<Store>) -> View {
    let state = remember(DialogState::new);
    let dialog_state = state.clone();
    let dialog_store = store.clone();

    let mut scope_labels = Vec::new();
    if store.state.get().settings.upgrade_repo {
        scope_labels.push("Repo");
    }
    if store.state.get().settings.upgrade_aur {
        scope_labels.push("AUR");
    }
    if store.state.get().settings.upgrade_flatpak {
        scope_labels.push("Flatpak");
    }
    if store.state.get().settings.upgrade_appimage {
        scope_labels.push("AppImage");
    }
    let scope = if scope_labels.is_empty() {
        "No sources selected".to_string()
    } else {
        scope_labels.join(" · ")
    };

    let open_state = state.clone();
    let row = Row(Modifier::new()
        .fill_max_width()
        .min_height(Dp(76.0))
        .padding_values(PaddingValues {
            left: SPACE_4,
            right: SPACE_4,
            top: SPACE_3,
            bottom: SPACE_3,
        })
        .align_items(AlignItems::CENTER)
        .gap(SPACE_3))
    .child((
        Box(Modifier::new()
            .size(Dp(38.0), Dp(38.0))
            .background(accent_bg())
            .clip_rounded(R_MD)
            .align_items(AlignItems::CENTER)
            .justify_content(JustifyContent::CENTER))
        .child(
            Icon(Symbols::SYSTEM_UPDATE_ALT)
                .size(Sp(20.0))
                .color(accent())
                .single_line(),
        ),
        Column(Modifier::new().flex_grow(1.0).gap(Dp(2.0))).child((
            Text("Sources included in “Update all”")
                .size(FONT_BASE)
                .font_weight(FontWeight::SEMI_BOLD)
                .color(text_primary())
                .single_line(),
            Text(scope)
                .size(FONT_SM)
                .color(text_muted())
                .single_line()
                .overflow_ellipsize(),
        )),
        secondary_button("Choose", move || open_state.show()),
    ));

    let dialog = Dialog(
        state.clone(),
        Modifier::new(),
        DialogProperties::default(),
        Column(
            Modifier::new()
                .width(if compact_layout() {
                    Dp(280.0)
                } else {
                    Dp(420.0)
                })
                .padding(SPACE_6)
                .background(surface())
                .border(Dp(1.0), card_border(), R_LG)
                .clip_rounded(R_LG)
                .gap(SPACE_4),
        )
        .with_children(vec![
            Row(Modifier::new()
                .fill_max_width()
                .align_items(AlignItems::CENTER))
            .child((
                Text("Update all sources")
                    .size(FONT_LG)
                    .line_height(Sp(24.0))
                    .font_weight(FontWeight::SEMI_BOLD)
                    .color(text_primary())
                    .modifier(Modifier::new().flex_grow(1.0)),
                tooltip_icon_button(
                    Symbols::CLOSE,
                    "Close",
                    {
                        let state = dialog_state.clone();
                        move || state.dismiss()
                    },
                    false,
                ),
            )),
            Text("Choose which package sources are included when updating every package.")
                .size(FONT_BASE)
                .line_height(LINE_BODY)
                .color(text_muted()),
            upgrade_checkbox("Repository packages", "repo", &dialog_store),
            upgrade_checkbox("AUR packages", "aur", &dialog_store),
            upgrade_checkbox("Flatpak packages", "flatpak", &dialog_store),
            upgrade_checkbox("AppImage packages", "appimage", &dialog_store),
            Row(Modifier::new().fill_max_width()).child((
                Box(Modifier::new().flex_grow(1.0)),
                primary_button("Done", {
                    let state = state.clone();
                    move || state.dismiss()
                }),
            )),
        ]),
    );

    Column(Modifier::new().fill_max_width()).child((row, dialog))
}

fn upgrade_checkbox(label: &'static str, backend: &'static str, store: &Rc<Store>) -> View {
    let enabled = match backend {
        "repo" => store.state.get().settings.upgrade_repo,
        "aur" => store.state.get().settings.upgrade_aur,
        "flatpak" => store.state.get().settings.upgrade_flatpak,
        "appimage" => store.state.get().settings.upgrade_appimage,
        _ => false,
    };
    let callback_store = store.clone();
    let key = backend.to_string();
    Row(Modifier::new()
        .fill_max_width()
        .align_items(AlignItems::CENTER))
    .child((
        Checkbox(
            enabled,
            move |value| {
                callback_store.dispatch(Action::SetUpgradeAllSource {
                    backend: key.clone(),
                    enabled: value,
                })
            },
            CheckboxConfig {
                checked_color: accent(),
                unchecked_color: surface(),
                checkmark_color: accent_ink(),
                checked_border_color: accent(),
                unchecked_border_color: card_border(),
                state_colors: StateColors {
                    default: Color::TRANSPARENT,
                    hovered: accent_bg(),
                    focused: accent_bg(),
                    pressed: accent_border(),
                    disabled: Color::TRANSPARENT,
                    dragged: accent_bg(),
                },
                ..Default::default()
            },
        ),
        Text(label)
            .size(FONT_BASE)
            .color(text_secondary())
            .single_line(),
    ))
}

fn settings_view(store: Rc<Store>, _s: &AppState) -> View {
    let settings = store.state.get().settings;
    let scroll = remember_scroll_state("settings_scroll");
    scroll.set_show_scrollbar(false);
    let padding = page_padding();
    let back_store = store.clone();

    let source_rows = vec![
        settings_switch_row(
            store.clone(),
            Symbols::INVENTORY,
            "Repository packages",
            "Arch Linux repositories such as core, extra, and multilib.",
            settings.enable_repo,
            "repo",
        ),
        HorizontalDivider(DividerConfig {
            color: card_border(),
            thickness: Dp(1.0),
            ..Default::default()
        }),
        settings_switch_row(
            store.clone(),
            Symbols::TERMINAL,
            "AUR packages",
            "Community packages from the Arch User Repository.",
            settings.enable_aur,
            "aur",
        ),
        HorizontalDivider(DividerConfig {
            color: card_border(),
            thickness: Dp(1.0),
            ..Default::default()
        }),
        settings_switch_row(
            store.clone(),
            Symbols::APPS,
            "Flatpak packages",
            "Sandboxed packages from configured Flatpak remotes.",
            settings.enable_flatpak,
            "flatpak",
        ),
        HorizontalDivider(DividerConfig {
            color: card_border(),
            thickness: Dp(1.0),
            ..Default::default()
        }),
        settings_switch_row(
            store.clone(),
            Symbols::PACKAGE_2,
            "AppImage packages",
            "Portable, self-contained Linux applications.",
            settings.enable_appimage,
            "appimage",
        ),
    ];

    let update_store = store.clone();
    let back_action = if compact_layout() {
        tooltip_icon_button(
            Symbols::CHEVRON_LEFT,
            "Back",
            move || {
                if let Some(navigator) = back_store.navigator.borrow().as_ref() {
                    navigator.pop();
                }
            },
            false,
        )
    } else {
        secondary_icon_button(Symbols::CHEVRON_LEFT, "Back", move || {
            if let Some(navigator) = back_store.navigator.borrow().as_ref() {
                navigator.pop();
            }
        })
    };
    Column(Modifier::new().fill_max_size().background(bg())).child((
        page_header(
            "Settings",
            Some("Choose which package sources Soredowe uses."),
            None,
            // Some(back_action),
            Vec::new(),
        ),
        HorizontalDivider(DividerConfig {
            color: card_border(),
            thickness: Dp(1.0),
            ..Default::default()
        }),
        ScrollArea(
            Modifier::new().fill_max_size().padding(padding),
            scroll,
            Column(
                Modifier::new()
                    .fill_max_width()
                    .max_width(Dp(760.0))
                    .align_self_center(),
            )
            .with_children(vec![
                section_heading(
                    "Appearance",
                    "Choose how Soredowe looks. Your choice is saved immediately.",
                ),
                Box(Modifier::new().height(SPACE_3)),
                theme_selector(store.clone(), settings.theme),
                Box(Modifier::new().height(SPACE_8)),
                section_heading(
                    "Package sources",
                    "Disabled sources are removed from search, updates, and package operations.",
                ),
                Box(Modifier::new().height(SPACE_3)),
                settings_card(source_rows),
                Box(Modifier::new().height(SPACE_8)),
                section_heading(
                    "Updates",
                    "Control which sources participate in the “Update all” operation.",
                ),
                Box(Modifier::new().height(SPACE_3)),
                settings_card(vec![upgrade_scope_row(update_store)]),
                Box(Modifier::new().height(SPACE_5)),
                settings_notice(),
                Box(Modifier::new().height(SPACE_8)),
            ]),
        ),
    ))
}

fn brand_mark(size: Dp) -> View {
    Box(Modifier::new()
        .size(size, size)
        .background(accent())
        .clip_rounded(R_MD)
        .align_items(AlignItems::CENTER)
        .justify_content(JustifyContent::CENTER))
    .child(
        Text("S")
            .size(Sp(size.0 * 0.48))
            .font_weight(FontWeight::BOLD)
            .color(accent_ink())
            .single_line(),
    )
}

fn drawer_nav_item(
    symbol: Symbol,
    label: &'static str,
    selected: bool,
    badge: Option<View>,
    on_click: Rc<dyn Fn()>,
) -> View {
    let background = if selected {
        accent_bg()
    } else {
        Color::TRANSPARENT
    };
    let foreground = if selected { accent() } else { text_muted() };
    let callback = on_click.clone();
    Box(Modifier::new()
        .fill_max_width()
        .min_height(Dp(44.0))
        .padding_values(PaddingValues {
            left: SPACE_2,
            right: SPACE_2,
            top: Dp(6.0),
            bottom: Dp(6.0),
        })
        .background(background)
        .state_colors(StateColors {
            default: Color::TRANSPARENT,
            hovered: surface_alt(),
            focused: surface_alt(),
            pressed: surface_hover(),
            disabled: Color::TRANSPARENT,
            dragged: surface_hover(),
        })
        .clip_rounded(R_SM)
        .semantics(
            Semantics::new(Role::Tab)
                .with_label(label)
                .with_selected(selected),
        )
        .clickable()
        .on_click(move || callback()))
    .child(
        Row(Modifier::new().align_items(AlignItems::CENTER).gap(SPACE_3)).child((
            Icon(symbol).size(Sp(20.0)).color(foreground).single_line(),
            Text(label)
                .size(FONT_BASE)
                .font_weight(if selected {
                    FontWeight::SEMI_BOLD
                } else {
                    FontWeight::MEDIUM
                })
                .color(if selected { accent() } else { text_secondary() })
                .single_line()
                .modifier(Modifier::new().flex_grow(1.0)),
            badge.unwrap_or(Box(Modifier::new())),
        )),
    )
}

fn selected_nav_index(route: &Route, state: &AppState) -> usize {
    match route {
        Route::Home | Route::Detail(_) => {
            if state.in_upgrades_view {
                1
            } else {
                0
            }
        }
        Route::Settings => 2,
    }
}

fn navigation_actions(store: Rc<Store>) -> [Rc<dyn Fn()>; 3] {
    let discover_store = store.clone();
    let discover = Rc::new(move || {
        if discover_store.state.get().in_upgrades_view {
            discover_store.dispatch(Action::Search);
        }
        if let Some(navigator) = discover_store.navigator.borrow().as_ref() {
            navigator.clear_and_push(Route::Home);
        }
    });

    let updates_store = store.clone();
    let updates = Rc::new(move || {
        updates_store.dispatch(Action::Upgrades);
        if let Some(navigator) = updates_store.navigator.borrow().as_ref() {
            navigator.clear_and_push(Route::Home);
        }
    });

    let settings_store = store;
    let settings = Rc::new(move || {
        if let Some(navigator) = settings_store.navigator.borrow().as_ref() {
            navigator.clear_and_push(Route::Settings);
        }
    });

    [discover, updates, settings]
}

fn navigation_drawer(selected: usize, update_count: usize, actions: [Rc<dyn Fn()>; 3]) -> View {
    Column(
        Modifier::new()
            .fill_max_size()
            .padding_values(PaddingValues {
                left: SPACE_3,
                right: SPACE_3,
                top: SPACE_4,
                bottom: SPACE_3,
            })
            .gap(SPACE_2),
    )
    .child((
        Row(Modifier::new()
            .fill_max_width()
            .padding(SPACE_2)
            .gap(SPACE_3))
        .child((
            brand_mark(Dp(36.0)),
            Column(Modifier::new().flex_grow(1.0)).child((
                Text("Soredowe")
                    .size(FONT_MD)
                    .font_weight(FontWeight::BOLD)
                    .color(text_primary())
                    .single_line(),
                Text("Package manager")
                    .size(FONT_XS)
                    .color(text_muted())
                    .single_line(),
            )),
        )),
        Box(Modifier::new()
            .fill_max_width()
            .height(Dp(1.0))
            .background(card_border())),
        Box(Modifier::new().height(SPACE_2)),
        drawer_nav_item(
            Symbols::SEARCH,
            "Discover",
            selected == 0,
            None,
            actions[0].clone(),
        ),
        drawer_nav_item(
            Symbols::SYSTEM_UPDATE_ALT,
            "Updates",
            selected == 1,
            if update_count > 0 {
                Some(count_badge(update_count))
            } else {
                None
            },
            actions[1].clone(),
        ),
        drawer_nav_item(
            Symbols::SETTINGS,
            "Settings",
            selected == 2,
            None,
            actions[2].clone(),
        ),
        Box(Modifier::new().flex_grow(1.0)),
    ))
}

fn navigation_rail(selected: usize, update_count: usize, actions: [Rc<dyn Fn()>; 3]) -> View {
    NavigationRail(
        selected,
        vec![
            NavRailItem {
                icon: Icon(Symbols::SEARCH).size(Sp(22.0)).single_line(),
                label: "Discover".to_string(),
                on_click: actions[0].clone(),
                badge: None,
                enabled: true,
                interaction_source: None,
            },
            NavRailItem {
                icon: Icon(Symbols::SYSTEM_UPDATE_ALT)
                    .size(Sp(22.0))
                    .single_line(),
                label: "Updates".to_string(),
                on_click: actions[1].clone(),
                badge: if update_count > 0 {
                    Some(count_badge(update_count))
                } else {
                    None
                },
                enabled: true,
                interaction_source: None,
            },
            NavRailItem {
                icon: Icon(Symbols::SETTINGS).size(Sp(22.0)).single_line(),
                label: "Settings".to_string(),
                on_click: actions[2].clone(),
                badge: None,
                enabled: true,
                interaction_source: None,
            },
        ],
        Some(brand_mark(Dp(36.0))),
        None,
        NavigationRailConfig {
            modifier: Modifier::new().background(surface()),
            container_color: surface(),
            selected_icon_color: accent(),
            selected_text_color: accent(),
            unselected_icon_color: text_muted(),
            unselected_text_color: text_muted(),
            indicator_color: accent_bg(),
            width: Dp(76.0),
            item_radius: R_MD,
            indicator_opacity: 1.0,
            item_spacing: SPACE_2,
            indicator_width: Dp(48.0),
            indicator_height: Dp(32.0),
        },
    )
}

fn navigation_bar(selected: usize, update_count: usize, actions: [Rc<dyn Fn()>; 3]) -> View {
    NavigationBar(
        selected,
        vec![
            NavItem {
                icon: Icon(Symbols::SEARCH).size(Sp(21.0)).single_line(),
                label: "Search".to_string(),
                on_click: actions[0].clone(),
                enabled: true,
                interaction_source: None,
            },
            NavItem {
                icon: if update_count > 0 {
                    Box(Modifier::new().width(Dp(24.0)).height(Dp(24.0))).child((
                        Icon(Symbols::SYSTEM_UPDATE_ALT)
                            .size(Sp(21.0))
                            .single_line(),
                        Box(Modifier::new().absolute().offset(
                            Some(Dp(12.0)),
                            Some(Dp(0.0)),
                            None,
                            None,
                        ))
                        .child(count_badge(update_count)),
                    ))
                } else {
                    Icon(Symbols::SYSTEM_UPDATE_ALT)
                        .size(Sp(21.0))
                        .single_line()
                },
                label: "Updates".to_string(),
                on_click: actions[1].clone(),
                enabled: true,
                interaction_source: None,
            },
            NavItem {
                icon: Icon(Symbols::SETTINGS).size(Sp(21.0)).single_line(),
                label: "Settings".to_string(),
                on_click: actions[2].clone(),
                enabled: true,
                interaction_source: None,
            },
        ],
        NavigationBarConfig {
            modifier: Modifier::new()
                .height(Dp(68.0))
                .border(Dp(1.0), card_border(), Dp(0.0)),
            container_color: surface(),
            content_color: text_secondary(),
            selected_icon_color: accent(),
            selected_text_color: accent(),
            unselected_icon_color: text_muted(),
            unselected_text_color: text_muted(),
            indicator_color: accent_bg(),
            height: Dp(68.0),
            tonal_elevation: Dp(0.0),
            indicator_opacity: 1.0,
            indicator_radius: R_PILL,
            item_spacing: SPACE_2,
            indicator_width: Dp(56.0),
            indicator_height: Dp(30.0),
        },
    )
}

fn adaptive_navigation(store: Rc<Store>, route: Route, content: View) -> View {
    let state = store.state.get();
    let selected = selected_nav_index(&route, &state);
    let update_count = if state.in_upgrades_view {
        state.results.len()
    } else {
        0
    };
    let actions = navigation_actions(store.clone());

    match window_size_class().width {
        WidthClass::Expanded => Row(Modifier::new().fill_max_size()).child((
            Box(Modifier::new()
                .width(Dp(224.0))
                .fill_max_height()
                .background(surface()))
            .child(navigation_drawer(selected, update_count, actions)),
            Box(Modifier::new()
                .weight(1.0)
                .min_width(Dp(0.0))
                .fill_max_height()
                .background(bg()))
            .child(content),
        )),
        WidthClass::Medium => Row(Modifier::new().fill_max_size()).child((
            Box(Modifier::new().fill_max_height()).child(navigation_rail(
                selected,
                update_count,
                actions,
            )),
            Box(Modifier::new()
                .fill_max_height()
                .width(Dp(1.0))
                .background(card_border())),
            Box(Modifier::new()
                .weight(1.0)
                .min_width(Dp(0.0))
                .fill_max_height())
            .child(content),
        )),
        WidthClass::Compact => Column(Modifier::new().fill_max_size()).child((
            Box(Modifier::new()
                .weight(1.0)
                .min_width(Dp(0.0))
                .fill_max_width())
            .child(content),
            Box(Modifier::new().height(Dp(1.0)).background(card_border())),
            navigation_bar(selected, update_count, actions),
        )),
    }
}

pub fn root_view(store: Rc<Store>) -> View {
    activate_theme(store.state.get().settings.theme);
    with_theme(material_theme(), || {
        let stack = remember_back_stack(Route::Home);
        *store.navigator.borrow_mut() = Some(Navigator {
            stack: (*stack).clone(),
        });
        let route = stack
            .top()
            .map(|(_, route, _, _)| route)
            .unwrap_or(Route::Home);

        let render_store = store.clone();
        let nav_display = NavDisplay(
            stack,
            renderer(move |scope: &EntryScope<Route>| {
                let state = render_store.state.get();
                match scope.key() {
                    Route::Home => home_view(render_store.clone()),
                    Route::Detail(_) => detail_page(render_store.clone(), &state),
                    Route::Settings => settings_view(render_store.clone(), &state),
                }
            }),
            None,
            NavTransition {
                slide_px: 24.0,
                fade: true,
                spec: AnimationSpec::fast(),
            },
        );

        Surface(
            SurfaceConfig {
                modifier: Modifier::new().fill_max_size().background(bg()),
                color: bg(),
                content_color: text_primary(),
                ..Default::default()
            },
            || adaptive_navigation(store.clone(), route, nav_display),
        )
    })
}

#[cfg(test)]
mod layout_tests {
    use super::*;
    use domain::config::Settings;
    use domain::{PackageId, Source};
    use std::collections::HashMap;

    #[test]
    fn long_package_description_stays_inside_adaptive_shell() {
        let (tx, _rx) = crossbeam_channel::unbounded();
        let store = Rc::new(Store::new(tx, None, Settings::default()));
        let package = PackageSummary {
            id: PackageId {
                name: "package-with-a-long-name".into(),
                source: Source::Aur,
                repo: Some("core".into()),
            },
            version: "1.0.0".into(),
            description: "supercalifragilisticexpialidocious-supercalifragilisticexpialidocious-supercalifragilisticexpialidocious-supercalifragilisticexpialidocious".into(),
            installed: false,
            popular: Some(1),
            last_updated: None,
        };
        let view = root_view(store.clone());
        let _ = repose_ui::layout_and_paint(
            &view,
            (1200, 800),
            &HashMap::new(),
            &Default::default(),
            None,
        );
        store.state.update(|state| {
            state.raw_results = vec![package.clone()];
            state.results = vec![package];
        });
        let view = root_view(store);

        for size in [(1200, 800), (900, 800), (600, 800)] {
            let (scene, _, _) = repose_ui::layout_and_paint(
                &view,
                size,
                &HashMap::new(),
                &Default::default(),
                None,
            );
            let overflow: Vec<_> = scene
                .nodes
                .iter()
                .filter_map(|node| match node {
                    SceneNode::Rect { rect, .. } | SceneNode::Text { rect, .. } => Some(*rect),
                    _ => None,
                })
                .filter(|rect| rect.x < -0.5 || rect.x + rect.w > size.0 as f32 + 0.5)
                .collect();
            assert!(overflow.is_empty(), "{size:?} overflow: {overflow:?}");
        }
    }
}
