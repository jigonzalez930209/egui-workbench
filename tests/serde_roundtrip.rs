//! Every built-in state and event round-trips through JSON.
//!
//! This is the 0.4 exit criterion: a host can save, send, and restore any component
//! state without a closure or a custom encoder.

#![cfg(feature = "serde")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use serde::Serialize;
use serde::de::DeserializeOwned;

use egui_workbench::app::{Document, Documents, ThemeState};
use egui_workbench::commands::{Command, CommandId, KeySequence, KeybindingEntry, Keybindings, When, WhenContext};
use egui_workbench::components::plot::{
    AxisConfig, Bounds, PlotConfig, PlotHover, PlotState, PlotTool, SeriesStyle, YAxis,
};
use egui_workbench::components::plot::{Camera3D, Plot3DState};
use egui_workbench::components::viewer::{Cell, FileFormat, FileViewer, Table, TextFile, ViewerError};
use egui_workbench::components::{PaletteState, ToastEvent, ToastId, ToastKind, ToastPosition};
use egui_workbench::egui::{self, Key, Modifiers};
use egui_workbench::{
    KeyChord, KeyboardMenuState, PromptAction, TabEvent, Theme, ThemeId, ThemeMode, TreeEvent, TreeState, WindowAction,
};

fn round_trip<T: Serialize + DeserializeOwned>(value: &T) {
    let first = serde_json::to_value(value).expect("serialize");
    let back: T = serde_json::from_value(first.clone()).expect("deserialize");
    let second = serde_json::to_value(&back).expect("serialize again");
    assert_eq!(first, second);
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
struct Tab {
    title: String,
    dirty: bool,
}

impl Document for Tab {
    fn title(&self) -> &str {
        &self.title
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }
}

#[test]
fn runtime_states_round_trip() {
    let ctx = egui::Context::default();
    round_trip(&ThemeState::new(&ctx, ThemeId::Dracula, true));
    round_trip(&Documents::with_items(vec![
        Tab {
            title: "one".to_owned(),
            dirty: false,
        },
        Tab {
            title: "two".to_owned(),
            dirty: true,
        },
    ]));
    round_trip(&PaletteState {
        open: true,
        query: "save".to_owned(),
        recent: vec!["Save".to_owned()],
    });
    round_trip(&KeyboardMenuState {
        open: true,
        group: 1,
        item: 2,
    });
    round_trip(&TreeState::<usize>::default());
    round_trip(&Theme::dark());
    round_trip(&ThemeId::OneDarkPro);
    round_trip(&ThemeMode::Dark);
}

#[test]
fn component_events_round_trip() {
    round_trip(&TabEvent::Select(2));
    round_trip(&TabEvent::Close(0));
    round_trip(&TreeEvent::Toggle(3));
    round_trip(&TreeEvent::Open(4));
    round_trip(&TreeEvent::Rename(5));
    round_trip(&PromptAction::Submit);
    round_trip(&PromptAction::Cancel);
    round_trip(&ToastKind::Success);
    round_trip(&ToastPosition::BottomRight);
    round_trip(&ToastEvent::Action(ToastId::default()));
    round_trip(&WindowAction::ToggleMaximize);
    round_trip(&FileFormat::Csv);
    round_trip(&ViewerError::Binary);
}

#[test]
fn plot_states_and_configs_round_trip() {
    round_trip(&PlotState::default());
    round_trip(&PlotConfig {
        title: Some("Signal".to_owned()),
        x: AxisConfig {
            label: Some("t (s)".to_owned()),
            ..AxisConfig::default()
        },
        series: vec![SeriesStyle {
            name: "sin".to_owned(),
            ..SeriesStyle::default()
        }],
        ..PlotConfig::default()
    });
    round_trip(&PlotHover {
        series: "sin".to_owned(),
        index: 3,
        point: [1.0, 2.0],
        axis: YAxis::Left,
    });
    round_trip(&PlotTool::Pan);
    round_trip(&Bounds::from_corners([0.0, 0.0], [1.0, 1.0]));
    let mut state = Plot3DState::default();
    state.set_hidden("surface", true);
    round_trip(&state);
    round_trip(&Camera3D::default());
}

#[test]
fn viewer_documents_round_trip() {
    let table = Table::new([
        [Cell::text("name"), Cell::number("1")],
        [Cell::text("value"), Cell::number("2")],
    ]);
    round_trip(&table);
    round_trip(&TextFile::new("one\ntwo\n"));
    let viewer = FileViewer::from_bytes("sample.csv", "name,value\none,1\n").expect("csv");
    round_trip(&viewer);
}

#[test]
fn command_types_round_trip() {
    round_trip(&CommandId::new("file.save"));
    round_trip(&KeyChord::new(Modifiers::COMMAND, Key::S));
    round_trip(&KeySequence::parse("Ctrl+K Ctrl+S").expect("sequence"));
    round_trip(&When::parse("editorFocus && !dialogOpen").expect("when"));
    round_trip(&WhenContext::new().with("editorFocus"));
    round_trip(&Command::new("file.save", "Save").shortcut("Ctrl+S"));
    round_trip(
        &Keybindings::new()
            .with(KeybindingEntry::new("file.save", "Ctrl+S"))
            .with(KeybindingEntry::new("edit.paste", "Ctrl+V")),
    );
}
