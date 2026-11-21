//! The status bar, the empty state, the split view, editor scrollbars, and icons.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use egui_workbench::egui::{self, Role, pos2};
use egui_workbench::metrics::STATUS_HEIGHT;
use egui_workbench::{Icons, MaterialIcons, StatusItem};

#[test]
fn the_status_bar_lays_out_left_and_right_cells() {
    let mut harness = common::harness([900.0, 200.0], (), |ui, ()| {
        egui::Panel::bottom("status").exact_size(STATUS_HEIGHT).show(ui, |ui| {
            let left = [
                StatusItem::new("…/docs/guide.md").tooltip("/home/me/docs/guide.md"),
                StatusItem::new("Saved"),
            ];
            let right = [
                StatusItem::new("UTF-8"),
                StatusItem::new("LF"),
                StatusItem::new("Large file").warning(),
            ];
            egui_workbench::show_status_bar(ui, &left, &right);
        });
    });
    harness.run();
    let x = |label: &str| harness.get_by_label(label).rect().center().x;
    assert!(x("…/docs/guide.md") < x("Saved"));
    assert!(
        x("UTF-8") > x("LF") && x("LF") > x("Large file"),
        "right cells run from the edge inward"
    );
    assert!(x("UTF-8") > 850.0);
    let (saved, utf8) = (
        harness.get_by_label("Saved").rect(),
        harness.get_by_label("UTF-8").rect(),
    );
    assert_eq!(saved.center().y, utf8.center().y, "all cells share one baseline");
    assert!(saved.top() > 150.0, "the bar is at the bottom: {saved:?}");
}

#[test]
fn the_empty_state_offers_its_action() {
    let mut harness = common::harness([800.0, 600.0], 0, |ui, clicks: &mut usize| {
        egui::CentralPanel::default().show(ui, |ui| {
            if egui_workbench::show_empty_state(ui, "App", "A native app.", "Open file", "Open a file to begin.") {
                *clicks += 1;
            }
        });
    });
    harness.run();
    let heading = harness.get_by_label("App").rect();
    assert!(
        heading.top() > 600.0 * 0.2,
        "the heading sits about a quarter down: {heading:?}"
    );
    assert!(harness.query_by_label("Open a file to begin.").is_some());
    harness.get_by_role_and_label(Role::Button, "Open file").click();
    harness.run();
    assert_eq!(*harness.state(), 1);
}

struct Split {
    ratio: f32,
    first: egui::Rect,
    second: egui::Rect,
}

fn split<'a>(width: f32) -> Harness<'a, Split> {
    let state = Split {
        ratio: 0.5,
        first: egui::Rect::NOTHING,
        second: egui::Rect::NOTHING,
    };
    common::harness([width, 600.0], state, |ui, state: &mut Split| {
        egui::CentralPanel::default().show(ui, |ui| {
            let (mut first, mut second) = (egui::Rect::NOTHING, egui::Rect::NOTHING);
            egui_workbench::split_panes(
                ui,
                &mut state.ratio,
                |ui| first = ui.max_rect(),
                |ui| second = ui.max_rect(),
            );
            state.first = first;
            state.second = second;
        });
    })
}

#[test]
fn wide_splits_are_columns_and_dragging_moves_the_divider() {
    let mut harness = split(1000.0);
    harness.run();
    let (first, second) = (harness.state().first, harness.state().second);
    assert!(first.right() < second.left(), "{first:?} {second:?}");
    assert_eq!(first.top(), second.top());
    assert!((second.width() / (first.width() + second.width() + 6.0) - 0.5).abs() < 0.02);

    let handle = pos2((first.right() + second.left()) / 2.0, 300.0);
    common::press_at(&harness, handle);
    harness.step();
    for step in 1..=10 {
        harness.event(egui::Event::PointerMoved(pos2(handle.x - step as f32 * 10.0, 300.0)));
        harness.step();
    }
    common::release_at(&harness, pos2(handle.x - 100.0, 300.0));
    harness.run();
    let ratio = harness.state().ratio;
    assert!(
        ratio > 0.55 && ratio <= 0.8,
        "dragging left grows the second pane: {ratio}"
    );
}

#[test]
fn narrow_splits_stack_and_ratios_are_clamped() {
    let mut harness = split(560.0);
    harness.state_mut().ratio = 0.95;
    harness.run();
    let (first, second) = (harness.state().first, harness.state().second);
    assert!(first.bottom() < second.top(), "{first:?} {second:?}");
    assert_eq!(first.left(), second.left());
    assert_eq!(harness.state().ratio, 0.8);
    assert!(first.height() >= 80.0 && second.height() >= 120.0);
}

#[test]
fn editor_scrollbars_apply_inside_and_restore_after() {
    let mut harness = common::harness([300.0, 200.0], Vec::new(), |ui, seen: &mut Vec<f32>| {
        let before = ui.spacing().scroll.dormant_handle_opacity;
        let inside = egui_workbench::with_editor_scrollbars(ui, |ui| ui.spacing().scroll.dormant_handle_opacity);
        let after = ui.spacing().scroll.dormant_handle_opacity;
        *seen = vec![before, inside, after];
    });
    harness.run();
    let seen = harness.state();
    assert_eq!(seen[1], 0.55);
    assert_eq!(seen[0], seen[2], "the parent style is restored");
    let style = egui_workbench::editor_scroll_style(&egui::style::ScrollStyle::default());
    assert!(style.floating && style.bar_width >= 10.0 && style.handle_min_length >= 24.0);
}

#[test]
fn ellipsizing_keeps_the_end_of_a_path() {
    assert_eq!(egui_workbench::ellipsize_start("/a/b", 48), "/a/b");
    let long = "/home/someone/projects/a/very/deep/folder/structure/file.md";
    let short = egui_workbench::ellipsize_start(long, 20);
    assert_eq!(short.chars().count(), 20);
    assert!(short.starts_with('…') && short.ends_with("file.md"));
}

#[test]
fn every_bundled_glyph_loads_and_custom_glyphs_replace_them() {
    let ctx = egui::Context::default();
    let mut icons = Icons::new();
    let names = Icons::bundled();
    for name in [
        "back",
        "forward",
        "close",
        "maximize",
        "minimize",
        "restore",
        "settings",
        "search",
        "command",
        "sidebar",
        "plus",
        "more",
        "history",
        "eye",
        "edit",
        "split",
        "chevron-down",
        "chevron-right",
        "chevron-up",
        "collapse",
        "refresh",
        "file-plus",
        "folder-plus",
        "folder-open",
        "logo",
    ] {
        assert!(names.iter().any(|bundled| bundled == name), "{name} is bundled");
    }
    for name in &names {
        let texture = icons.texture(&ctx, name).unwrap_or_else(|| panic!("{name}"));
        assert_eq!(texture.size(), [48, 48], "{name}");
    }
    assert!(icons.texture(&ctx, "does-not-exist").is_none());

    let original = icons.texture(&ctx, "logo").expect("logo").id();
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><circle cx="12" cy="12" r="10" fill="#fff"/></svg>"##;
    icons.register("logo", svg);
    let replaced = icons.texture(&ctx, "logo").expect("custom logo").id();
    assert_ne!(original, replaced, "a registered glyph replaces the bundled one");
    icons.register("app-mark", svg);
    assert!(icons.texture(&ctx, "app-mark").is_some());
    assert!(icons.texture(&ctx, "broken").is_none());
    icons.register("broken", "not svg");
    assert!(
        icons.texture(&ctx, "broken").is_none(),
        "an invalid SVG yields no texture"
    );
}

#[test]
fn material_icons_resolve_and_load_for_files_and_folders() {
    let ctx = egui::Context::default();
    let mut icons = MaterialIcons::new();
    for (path, light, expected) in [
        ("main.rs", true, "rust"),
        ("README.md", false, "readme"),
        ("Cargo.toml", true, "toml_light"),
        ("Cargo.toml", false, "toml"),
        ("package.json", false, "nodejs"),
        ("unknown.zzz", false, "file"),
    ] {
        let id = icons.file_icon(std::path::Path::new(path), light);
        assert_eq!(id, expected, "{path}");
        let texture = icons.texture(&ctx, &id, light).unwrap_or_else(|| panic!("{id}"));
        assert_eq!(texture.size(), [16, 16], "{id}");
    }
    assert_eq!(icons.folder_icon("src", false, false, false), "folder-src");
    assert_eq!(icons.folder_icon("src", true, false, false), "folder-src-open");
    assert_eq!(icons.folder_icon("misc-stuff", true, false, false), "folder-open");
    assert_eq!(icons.folder_icon("anything", false, true, false), "folder-root");
    let big = icons.texture_sized(&ctx, "rust", false, 32.0).expect("32 pt");
    assert_eq!(big.size(), [32, 32]);
}

#[test]
fn tiny_splits_never_panic_or_overflow() {
    for (width, height) in [(560.0, 180.0), (300.0, 60.0), (900.0, 40.0), (40.0, 30.0)] {
        let mut harness = common::harness(
            [width + 16.0, height + 16.0],
            Vec::new(),
            |ui, rects: &mut Vec<egui::Rect>| {
                let outer = ui.max_rect();
                let mut ratio = 0.5;
                let (mut first, mut second) = (egui::Rect::NOTHING, egui::Rect::NOTHING);
                egui_workbench::split_panes(ui, &mut ratio, |ui| first = ui.max_rect(), |ui| second = ui.max_rect());
                rects.extend([first, second, outer]);
            },
        );
        harness.run();
        let rects = &harness.state()[harness.state().len() - 3..];
        let outer = rects[2];
        for pane in &rects[..2] {
            assert!(
                pane.right() <= outer.right() + 0.5,
                "{width}x{height}: {pane:?} past {outer:?}"
            );
            assert!(
                pane.bottom() <= outer.bottom() + 0.5,
                "{width}x{height}: {pane:?} below {outer:?}"
            );
        }
    }
}
