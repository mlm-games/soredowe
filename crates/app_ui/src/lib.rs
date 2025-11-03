use crate::state::{Action, SortMode, Store};
use domain::{PackageSummary, Source};
use repose_core::*;
use repose_ui::*;
use std::{cell::RefCell, rc::Rc};

pub mod state;

// Simple badges
fn badge(text: &str, bg: Color) -> View {
    TextColor(Text(text.to_string()), Color::from_hex("#EEEEEE")).modifier(
        Modifier::new()
            .background(bg)
            .clip_rounded(4.0)
            .padding(6.0),
    )
}

// Filter chip
fn chip(label: &str, on: bool, on_toggle: impl Fn() + 'static) -> View {
    Button(label, on_toggle).modifier(
        Modifier::new()
            .padding(4.0)
            .background(if on {
                Color::from_hex("#2A8F6A")
            } else {
                Color::from_hex("#2A2A2A")
            })
            .clip_rounded(6.0),
    )
}

// Row separator
fn separator() -> View {
    Box(Modifier::new()
        .size(1.0, 1.0)
        .background(Color::from_hex("#2A2A2A")))
}

// Package row
fn pkg_row(store: Rc<Store>, pkg: PackageSummary, selected: bool) -> View {
    let is_aur = pkg.id.source == Source::Aur;
    Row(Modifier::new()
        .padding(10.0)
        .background(if selected {
            Color::from_hex("#244E74")
        } else if is_aur {
            Color::from_hex("#1A2030")
        } else {
            Color::from_hex("#1E1E1E")
        })
        .border(1.0, Color::from_hex("#333333"), 8.0)
        .clip_rounded(8.0)
        .clickable()
        .on_pointer_down({
            let store = store.clone();
            let id = pkg.id.clone();
            move |_| store.dispatch(Action::Select(id.clone()))
        }))
    .child((
        // Left col: name + meta
        Column(Modifier::new().flex_grow(1.0)).child((
            Row(Modifier::new()).child((
                Text(pkg.id.name.clone()).modifier(Modifier::new().padding(2.0)),
                if is_aur {
                    badge("AUR", Color::from_hex("#6B46C1"))
                } else {
                    badge("Repo", Color::from_hex("#2D6A4F"))
                },
                if pkg.installed {
                    badge("Installed", Color::from_hex("#4B5563"))
                } else {
                    Box(Modifier::new())
                },
            )),
            TextColor(
                TextSize(Text(pkg.description.clone()), 12.0),
                Color::from_hex("#AAAAAA"),
            )
            .modifier(Modifier::new().padding(2.0)),
        )),
        // Right col: actions
        Row(Modifier::new()).child(
            Button(if pkg.installed { "Remove" } else { "Install" }, {
                let store = store.clone();
                let id = pkg.id.clone();
                move || {
                    if pkg.installed {
                        store.dispatch(Action::Remove(id.clone()))
                    } else {
                        store.dispatch(Action::Install(id.clone()))
                    }
                }
            })
            .modifier(Modifier::new().padding(6.0)),
        ),
    ))
}

// Details card (right pane)
fn details_card(store: Rc<Store>) -> View {
    let s = store.state.get();
    let results = s.results.clone();
    let selected = s.selected.clone();
    let Some(id) = &s.selected else {
        return Column(Modifier::new().padding(16.0)).child(TextColor(
            Text("Select a package to see details"),
            Color::from_hex("#AAAAAA"),
        ));
    };
    // Find summary in current results (lightweight until details endpoint is used)
    let pkg = results.into_iter().find(|p| &p.id == id);
    if let Some(pkg) = pkg {
        Column(
            Modifier::new()
                .padding(16.0)
                .background(Color::from_hex("#1B1B1B"))
                .border(1.0, Color::from_hex("#333333"), 10.0)
                .clip_rounded(10.0),
        )
        .child((
            Row(Modifier::new()).child((
                TextSize(Text(pkg.id.name.clone()), 18.0),
                if pkg.id.source == Source::Aur {
                    badge("AUR", Color::from_hex("#6B46C1"))
                } else {
                    badge("Repo", Color::from_hex("#2D6A4F"))
                },
                if pkg.installed {
                    badge("Installed", Color::from_hex("#4B5563"))
                } else {
                    Box(Modifier::new())
                },
            )),
            TextColor(Text(pkg.description.clone()), Color::from_hex("#BBBBBB"))
                .modifier(Modifier::new().padding(6.0)),
            Row(Modifier::new().padding(8.0)).child((
                Button(if pkg.installed { "Remove" } else { "Install" }, {
                    let store = store.clone();
                    let id = pkg.id.clone();
                    move || {
                        if pkg.installed {
                            store.dispatch(Action::Remove(id.clone()))
                        } else {
                            store.dispatch(Action::Install(id.clone()))
                        }
                    }
                }),
                Button("Clear selection", {
                    let store = store.clone();
                    move || store.dispatch(Action::ClearSelection)
                })
                .modifier(Modifier::new().padding(6.0)),
            )),
        ))
    } else {
        Column(Modifier::new().padding(16.0)).child(TextColor(
            Text("No details available"),
            Color::from_hex("#AAAAAA"),
        ))
    }
}

pub fn root_view(store: Rc<Store>) -> View {
    let query_live = remember_with_key("query_live", || signal(String::new()));
    // Visual TextField; query comes from Store (see note at top)
    let _search_tf = remember_with_key("search_tf", || {
        Rc::new(RefCell::new(repose_ui::textfield::TextFieldState::new()))
    });

    let s = store.state.get();

    Surface(
        Modifier::new()
            .fill_max_size()
            .background(Color::from_hex("#0F1012")),
        Column(Modifier::new().padding(12.0)).child((
            // Header bar
            Row(Modifier::new().padding(8.0)).child((
                TextSize(Text("Heyday"), 20.0).modifier(Modifier::new().padding(8.0)),
                Spacer(),
                Button("Refresh", {
                    let store = store.clone();
                    move || store.dispatch(Action::Search)
                })
                .modifier(Modifier::new().padding(4.0)),
                Button("Upgrades", {
                    let store = store.clone();
                    move || store.dispatch(Action::Search) // placeholder for Upgrades flow
                })
                .modifier(Modifier::new().padding(4.0)),
            )),
            separator(),
            // Search row
            Row(Modifier::new().padding(8.0)).child((
                repose_ui::textfield::TextField(
                    "Search packages…",
                    Modifier::new()
                        .size(420.0, 36.0)
                        .background(Color::from_hex("#171717"))
                        .border(1.0, Color::from_hex("#3A3A3A"), 6.0)
                        .clip_rounded(6.0)
                        .semantics("Search field"),
                    {
                        let store = store.clone();
                        let query_live = query_live.clone();
                        move |text| {
                            query_live.set(text.clone());
                            store.dispatch(Action::SetQuery(text));
                        }
                    },
                ),
                // Search button: ensure store gets the very latest text, then search
                Button("Search", {
                    let store = store.clone();
                    let query_live = query_live.clone();
                    move || {
                        let q = query_live.get();
                        store.dispatch(Action::SetQuery(q));
                        store.dispatch(Action::Search);
                    }
                })
                .modifier(Modifier::new().padding(8.0)),
                // Filters
                chip("Repo", s.filter_repo, {
                    let store = store.clone();
                    move || store.dispatch(Action::ToggleFilterRepo)
                }),
                chip("AUR", s.filter_aur, {
                    let store = store.clone();
                    move || store.dispatch(Action::ToggleFilterAur)
                }),
                chip("Installed", s.filter_installed, {
                    let store = store.clone();
                    move || store.dispatch(Action::ToggleFilterInstalled)
                }),
                // Sort
                Row(Modifier::new().padding(6.0)).child((
                    Button("A–Z", {
                        let store = store.clone();
                        move || store.dispatch(Action::SetSort(SortMode::NameAsc))
                    }),
                    Button("Z–A", {
                        let store = store.clone();
                        move || store.dispatch(Action::SetSort(SortMode::NameDesc))
                    }),
                    Button("Popular", {
                        let store = store.clone();
                        move || store.dispatch(Action::SetSort(SortMode::Popularity))
                    }),
                )),
            )),
            // Body: responsive two-pane layout
            // Use Grid(6) and span (4|2) or (6) if narrow.
            {
                // crude breakpoint using item width heuristic
                let wide = true; // keep simple; grid will still fill width
                let left_span = if wide { 4 } else { 6 };
                let right_span = if wide { 2 } else { 6 };

                Grid(
                    6,
                    Modifier::new().fill_max_size().padding(6.0),
                    vec![
                        // Left: result list
                        Column(Modifier::new().grid_span(left_span, 1)).child(
                            if s.results.is_empty() {
                                Column(Modifier::new().padding(16.0)).child(TextColor(
                                    Text("No results. Try searching."),
                                    Color::from_hex("#888888"),
                                ))
                            } else {
                                repose_ui::lazy::LazyColumn(
                                    s.results.clone(),
                                    56.0,
                                    remember_with_key("scroll", || {
                                        repose_ui::lazy::LazyColumnState::new()
                                    }),
                                    Modifier::new().fill_max_size(),
                                    {
                                        let store = store.clone();
                                        move |pkg: PackageSummary, _| {
                                            let selected = s
                                                .selected
                                                .as_ref()
                                                .map_or(false, |id| *id == pkg.id);
                                            pkg_row(store.clone(), pkg, selected)
                                        }
                                    },
                                )
                            },
                        ),
                        // Right: details
                        Column(Modifier::new().grid_span(right_span, 1))
                            .child(details_card(store.clone())),
                    ],
                )
            },
            // Footer / status
            Row(Modifier::new().padding(8.0)).child((
                TextColor(TextSize(Text("Status"), 12.0), Color::from_hex("#888888")),
                TextColor(
                    Text(format!(
                        "  |  {}",
                        s.progress_log.lines().last().unwrap_or("")
                    )),
                    Color::from_hex("#A0A0A0"),
                )
                .modifier(Modifier::new().padding(4.0)),
                Spacer(),
                Button(
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
            )),
            if s.log_expanded {
                Box(Modifier::new()
                    .fill_max_size()
                    .size(0.0, 180.0)
                    .background(Color::from_hex("#101010"))
                    .border(1.0, Color::from_hex("#2A2A2A"), 6.0)
                    .clip_rounded(6.0))
                .child(
                    TextColor(
                        TextSize(Text(s.progress_log.clone()), 12.0),
                        Color::from_hex("#B0B0B0"),
                    )
                    .modifier(Modifier::new().padding(8.0)),
                )
            } else {
                Box(Modifier::new())
            },
        )),
    )
}
