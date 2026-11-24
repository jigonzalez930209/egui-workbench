//! The keybindings editor records a chord and applies it to the registry.

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use egui_workbench::commands::{
    Command, CommandId, CommandRegistry, KeybindingChange, KeybindingsEditorState, KeybindingsLabels,
    show_keybindings_editor,
};
use egui_workbench::egui::{self, Key, Modifiers};
use egui_workbench::theme;

struct State {
    editor: KeybindingsEditorState,
    registry: CommandRegistry,
    changes: Vec<KeybindingChange>,
}

fn harness() -> Harness<'static, State> {
    let mut registry = CommandRegistry::new();
    registry.register(Command::new("file.save", "Save").shortcut("Ctrl+S"));
    let mut editor = KeybindingsEditorState::default();
    editor.open();
    Harness::builder()
        .with_size(egui::vec2(760.0, 520.0))
        .with_max_steps(16)
        .with_context_setup(|ctx| theme::apply_id(ctx, egui_workbench::ThemeId::GithubLight))
        .build_ui_state(
            |ui, state: &mut State| {
                let changes = show_keybindings_editor(
                    ui.ctx(),
                    &mut state.editor,
                    &mut state.registry,
                    KeybindingsLabels::default(),
                );
                state.changes.extend(changes);
            },
            State {
                editor,
                registry,
                changes: Vec::new(),
            },
        )
}

#[test]
fn rebind_records_the_next_key() {
    let mut harness = harness();
    harness.run();
    harness.get_by_label("Rebind").click();
    harness.run();
    assert_eq!(harness.state().editor.recording, Some(CommandId::new("file.save")));
    harness.key_press_modifiers(Modifiers::COMMAND, Key::K);
    harness.run();
    assert!(harness.state().editor.recording.is_none());
    assert_eq!(
        harness
            .state()
            .registry
            .binding(&CommandId::new("file.save"))
            .map(ToString::to_string),
        Some("Ctrl+K".to_owned())
    );
    assert!(matches!(
        harness.state().changes.as_slice(),
        [KeybindingChange::Bound { .. }]
    ));
}
