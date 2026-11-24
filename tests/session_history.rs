//! End-to-end checks for the 0.5 history, journal, session, and recovery API.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use egui_workbench::app::{
    CommandJournal, DevToolsState, History, HotExit, SessionPaths, Stash, StashDocument, Workspace, WorkspaceDocument,
    recover, show_devtools,
};
use egui_workbench::commands::CommandId;
use egui_workbench::egui::{self, Event, Key, Modifiers, RawInput};

fn directory(name: &str) -> std::path::PathBuf {
    let thread: String = std::thread::current()
        .name()
        .unwrap_or("test")
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '_'
            }
        })
        .collect();
    std::env::temp_dir().join(format!("egui-workbench-{name}-{}-{thread}", std::process::id()))
}

#[test]
fn an_undo_branch_and_a_macro_replay() {
    let mut history = History::new("start".to_owned());
    history.record("one", "one".to_owned());
    let two = history.record("two", "two".to_owned());
    history.undo();
    let branch = history.record("branch", "branch".to_owned());
    assert_eq!(history.node(two).map(|node| node.label.as_str()), Some("two"));
    assert_eq!(history.node(branch).map(|node| node.state.as_str()), Some("branch"));

    let mut journal = CommandJournal::new();
    journal.append("file.new");
    journal.append("edit.type");
    let mut macros = egui_workbench::app::MacroLibrary::new();
    let recorded = macros.record_from(&journal, 0, "New and type").clone();
    let mut replayed = Vec::new();
    recorded.replay(|step| replayed.push(step.command.clone()));
    assert_eq!(replayed, [CommandId::new("file.new"), CommandId::new("edit.type")]);
}

#[test]
fn a_workspace_stash_and_journal_survive_a_crash_and_recover() {
    let directory = directory("recovery");
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("create");
    let paths = SessionPaths::new(&directory);

    let mut workspace = Workspace::new();
    workspace.add_document(WorkspaceDocument::new("a", "A"));
    workspace.theme = egui_workbench::ThemeId::Dracula;
    let stash = Stash::capture(1.0, [StashDocument::new("a", "A", "unsaved text")]);
    let mut journal = CommandJournal::new();
    journal.append("file.new");
    let mut timer = HotExit::with_interval(5.0);
    timer.save(5.0, &paths, &workspace, &stash, &journal).expect("hot exit");

    let recovery = recover(&directory);
    assert_eq!(
        recovery.workspace.as_ref().map(|ws| ws.theme),
        Some(egui_workbench::ThemeId::Dracula)
    );
    assert_eq!(
        recovery
            .stash
            .as_ref()
            .and_then(|stash| stash.documents.first())
            .map(|doc| doc.text.as_str()),
        Some("unsaved text")
    );
    assert_eq!(recovery.journal.as_ref().map(CommandJournal::len), Some(1));
    assert!(recovery.notice().is_some());
    assert!(recovery.warnings.is_empty());
    let _ = std::fs::remove_dir_all(directory);
}

#[test]
fn f12_toggles_devtools_and_a_snapshot_renders() {
    let ctx = egui::Context::default();
    let mut state = DevToolsState::default();
    let mut output = ctx.run_ui(
        RawInput {
            events: vec![Event::Key {
                key: Key::F12,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
            ..RawInput::default()
        },
        |_ui| {
            assert!(state.toggle_on_f12(&ctx));
        },
    );
    output.textures_delta.clear();
    let journal = CommandJournal::new();
    let snapshot = serde_json::json!({"tabs": 1, "dirty": false});
    let mut output = ctx.run_ui(RawInput::default(), |ui| {
        show_devtools(ui.ctx(), &mut state, &journal, &snapshot, Default::default());
    });
    output.textures_delta.clear();
    assert!(state.open);
}
