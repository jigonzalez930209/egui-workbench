//! The optional application runtime: fixed panel order and command collection.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use egui_workbench::app::{WorkbenchApp, show_workbench};
use egui_workbench::egui::{self, Rect};

#[derive(Default)]
struct Recorder {
    title: Option<Rect>,
    sidebar: Option<Rect>,
    tabs: Option<Rect>,
    status: Option<Rect>,
    central: Option<Rect>,
    commands: Vec<u8>,
}

impl WorkbenchApp for Recorder {
    type Command = u8;

    fn sidebar_visible(&self) -> bool {
        true
    }

    fn tab_strip_visible(&self) -> bool {
        true
    }

    fn status_bar_visible(&self) -> bool {
        true
    }

    fn title_bar(&mut self, ui: &mut egui::Ui) -> Vec<u8> {
        self.title = Some(ui.max_rect());
        ui.allocate_space(ui.available_size());
        vec![1]
    }

    fn sidebar(&mut self, ui: &mut egui::Ui) -> Vec<u8> {
        self.sidebar = Some(ui.max_rect());
        ui.allocate_space(ui.available_size());
        vec![2]
    }

    fn tab_strip(&mut self, ui: &mut egui::Ui) -> Vec<u8> {
        self.tabs = Some(ui.max_rect());
        ui.allocate_space(ui.available_size());
        vec![3]
    }

    fn status_bar(&mut self, ui: &mut egui::Ui) -> Vec<u8> {
        self.status = Some(ui.max_rect());
        ui.allocate_space(ui.available_size());
        vec![4]
    }

    fn central(&mut self, ui: &mut egui::Ui) -> Vec<u8> {
        self.central = Some(ui.max_rect());
        ui.allocate_space(ui.available_size());
        vec![5]
    }

    fn overlays(&mut self, _ui: &mut egui::Ui) -> Vec<u8> {
        vec![6]
    }
}

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() <= 1.0
}

#[test]
fn panels_follow_the_fixed_workbench_order() {
    let mut harness = common::harness([800.0, 600.0], Recorder::default(), |ui, recorder| {
        recorder.commands = show_workbench(ui, recorder);
    });
    harness.run();

    let recorder = harness.state();
    let title = recorder.title.expect("title bar");
    let sidebar = recorder.sidebar.expect("sidebar");
    let tabs = recorder.tabs.expect("tab strip");
    let status = recorder.status.expect("status bar");
    let central = recorder.central.expect("central content");

    assert!(close(title.width(), 768.0), "title spans the window: {title:?}");
    assert!(close(sidebar.width(), 259.0), "sidebar keeps its width: {sidebar:?}");
    assert!(close(tabs.left(), central.left()), "content aligns under the tabs");
    assert!(close(status.left(), central.left()), "status aligns with the content");
    assert!(close(central.right(), title.right()), "content spans the editor column");
    assert!(close(status.bottom(), 590.0), "status sits on the bottom edge");
    assert!(tabs.top() > title.top(), "tabs sit below the title bar");
    assert!(central.top() > tabs.bottom(), "content sits below the tabs");
    assert!(status.top() > central.bottom(), "status sits below the content");
    assert_eq!(recorder.commands, vec![1, 2, 3, 4, 5, 6]);
}

#[test]
fn hidden_panels_leave_no_empty_chrome() {
    #[derive(Default)]
    struct Bare;
    impl WorkbenchApp for Bare {
        type Command = ();
        fn title_bar_visible(&self) -> bool {
            false
        }
        fn central(&mut self, ui: &mut egui::Ui) -> Vec<()> {
            ui.label("content");
            Vec::new()
        }
    }

    let mut harness = common::harness([400.0, 300.0], Bare, |ui, app| {
        let _ = show_workbench(ui, app);
    });
    harness.run();
}

#[derive(Default)]
struct FullLayout {
    sidebar: Option<Rect>,
    right: Option<Rect>,
    tabs: Option<Rect>,
    central: Option<Rect>,
    bottom: Option<Rect>,
    status: Option<Rect>,
}

impl WorkbenchApp for FullLayout {
    type Command = ();

    fn sidebar_visible(&self) -> bool {
        true
    }
    fn tab_strip_visible(&self) -> bool {
        true
    }
    fn status_bar_visible(&self) -> bool {
        true
    }
    fn right_panel_visible(&self) -> bool {
        true
    }
    fn bottom_panel_visible(&self) -> bool {
        true
    }
    fn sidebar(&mut self, ui: &mut egui::Ui) -> Vec<()> {
        self.sidebar = Some(ui.max_rect());
        Vec::new()
    }
    fn right_panel(&mut self, ui: &mut egui::Ui) -> Vec<()> {
        self.right = Some(ui.max_rect());
        Vec::new()
    }
    fn tab_strip(&mut self, ui: &mut egui::Ui) -> Vec<()> {
        self.tabs = Some(ui.max_rect());
        Vec::new()
    }
    fn central(&mut self, ui: &mut egui::Ui) -> Vec<()> {
        self.central = Some(ui.max_rect());
        Vec::new()
    }
    fn bottom_panel(&mut self, ui: &mut egui::Ui) -> Vec<()> {
        self.bottom = Some(ui.max_rect());
        Vec::new()
    }
    fn status_bar(&mut self, ui: &mut egui::Ui) -> Vec<()> {
        self.status = Some(ui.max_rect());
        Vec::new()
    }
}

#[test]
fn right_and_bottom_panels_frame_the_content() {
    for width in [1400.0, 900.0] {
        let mut harness = common::harness([width, 700.0], FullLayout::default(), |ui, layout| {
            show_workbench(ui, layout);
        });
        harness.run();
        let layout = harness.state();
        let (sidebar, right) = (layout.sidebar.unwrap(), layout.right.unwrap());
        let (central, bottom, status) = (layout.central.unwrap(), layout.bottom.unwrap(), layout.status.unwrap());
        let tabs = layout.tabs.unwrap();
        assert!(
            sidebar.right() <= central.left() + 1.0,
            "{width}: explorer, then content"
        );
        assert!(
            central.right() <= right.left() + 1.0,
            "{width}: content, then right panel"
        );
        assert!(close(tabs.left(), central.left()) && close(tabs.right(), central.right()));
        assert!(
            central.bottom() <= bottom.top() + 1.0,
            "{width}: the bottom panel sits under the content"
        );
        assert!(
            bottom.bottom() <= status.top() + 1.0,
            "{width}: and above the status bar"
        );
        assert!(
            close(bottom.left(), central.left()) && close(bottom.right(), central.right()),
            "{width}: {bottom:?} vs {central:?}"
        );
        assert!(
            right.height() > central.height(),
            "{width}: the right panel spans the full height"
        );
        if width < 1000.0 {
            assert!(
                right.width() < 280.0,
                "{width}: the right panel yields on narrow windows: {right:?}"
            );
        }
    }
}
