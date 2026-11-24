//! Unit tests for the command registry.

use egui::{Event, Key, Modifiers, RawInput};

use super::*;
use crate::commands::{ArgSchema, When, WhenContext};

fn registry() -> CommandRegistry {
    let mut registry = CommandRegistry::new();
    registry.register(Command::new("file.save", "Save").shortcut("Ctrl+S"));
    registry.register(Command::new("file.save-as", "Save As…").shortcut("Ctrl+Shift+S"));
    registry.register(
        Command::new("edit.paste", "Paste")
            .shortcut("Ctrl+V")
            .when("editorFocus"),
    );
    registry
}

#[test]
fn commands_keep_metadata_and_state() {
    let mut registry = registry();
    let id = CommandId::new("file.save");
    assert_eq!(registry.len(), 3);
    assert_eq!(registry.get(&id).map(|command| command.title.as_str()), Some("Save"));
    assert!(registry.set_enabled(&id, false));
    assert!(!registry.get(&id).expect("save").is_enabled());
    assert_eq!(registry.categories(), ["General"]);
}

#[test]
fn conflicting_bindings_are_reported_not_applied() {
    let mut registry = registry();
    registry.register(Command::new("file.duplicate", "Duplicate").shortcut("Ctrl+S"));
    assert_eq!(registry.conflicts().len(), 1);
    assert_eq!(registry.conflicts()[0].command, CommandId::new("file.duplicate"));
    registry.register(Command::new("file.other", "Other").shortcut("Ctrl+S"));
    assert_eq!(registry.conflicts().len(), 2);
}

#[test]
fn rebinding_rejects_a_sequence_owned_by_another_command() {
    let mut registry = registry();
    let paste = CommandId::new("edit.paste");
    let error = registry
        .bind(&paste, KeySequence::parse("Ctrl+S").expect("sequence"))
        .expect_err("conflict");
    assert!(matches!(error, BindingError::Conflict { .. }));
    registry
        .bind(&paste, KeySequence::parse("Ctrl+Shift+V").expect("sequence"))
        .expect("bind");
    assert_eq!(
        registry.binding(&paste).map(ToString::to_string),
        Some("Ctrl+Shift+V".to_owned())
    );
    registry.reset_binding(&paste);
    assert_eq!(
        registry.binding(&paste).map(ToString::to_string),
        Some("Ctrl+V".to_owned())
    );
}

fn press(ctx: &Context, modifiers: Modifiers, key: Key) -> Option<CommandId> {
    let input = RawInput {
        events: vec![Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        }],
        ..RawInput::default()
    };
    let mut hit = None;
    let mut registry = registry();
    let context = WhenContext::new().with("editorFocus");
    let mut output = ctx.run_ui(input, |ui| {
        hit = registry.shortcut(ui.ctx(), &context);
    });
    output.textures_delta.clear();
    hit
}

#[test]
fn shortcuts_respect_context_and_enabled_state() {
    let ctx = Context::default();
    assert_eq!(
        press(&ctx, Modifiers::COMMAND, Key::S),
        Some(CommandId::new("file.save"))
    );
    assert_eq!(
        press(&ctx, Modifiers::COMMAND, Key::V),
        Some(CommandId::new("edit.paste"))
    );
}

#[test]
fn chord_sequences_wait_for_the_second_chord() {
    let ctx = Context::default();
    let mut registry = CommandRegistry::new();
    registry.register(Command::new("workbench.keybindings", "Keybindings").shortcut("Ctrl+K Ctrl+S"));
    let context = WhenContext::new();
    let mut hit = None;
    let mut output = ctx.run_ui(
        RawInput {
            events: vec![Event::Key {
                key: Key::K,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::COMMAND,
            }],
            ..RawInput::default()
        },
        |ui| {
            hit = registry.shortcut(ui.ctx(), &context);
            assert_eq!(registry.pending().len(), 1);
        },
    );
    output.textures_delta.clear();
    assert!(hit.is_none());
    let mut output = ctx.run_ui(
        RawInput {
            events: vec![Event::Key {
                key: Key::S,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::COMMAND,
            }],
            ..RawInput::default()
        },
        |ui| hit = registry.shortcut(ui.ctx(), &context),
    );
    output.textures_delta.clear();
    assert_eq!(hit, Some(CommandId::new("workbench.keybindings")));
}

#[test]
fn recent_commands_keep_the_newest_twelve() {
    let mut registry = registry();
    for index in 0..20 {
        registry.note_used(CommandId::new(format!("command.{index}")));
    }
    assert_eq!(registry.recent().count(), 12);
    assert_eq!(
        registry.recent().next().map(ToString::to_string),
        Some("command.19".to_owned())
    );
}

#[test]
fn invalid_identifiers_are_visible() {
    assert!(CommandId::new("file.save").is_valid());
    assert!(CommandId::new("file.save").is_namespaced());
    assert!(!CommandId::new("file save").is_valid());
    assert!(!CommandId::new("save").is_namespaced());
}

#[cfg(feature = "serde")]
#[test]
fn commands_and_ids_round_trip_through_json() {
    let id = CommandId::new("file.save");
    let json = serde_json::to_string(&id).expect("serialize");
    assert_eq!(json, "\"file.save\"");
    let back: CommandId = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, id);
    let when = When::parse("editorFocus").expect("when");
    let command = Command::new("file.save", "Save")
        .when(when.to_string())
        .args(ArgSchema::text("path", "Path"));
    let json = serde_json::to_string(&command).expect("command");
    let back: Command = serde_json::from_str(&json).expect("deserialize command");
    assert_eq!(back.id, command.id);
    assert_eq!(back.args, command.args);
}
