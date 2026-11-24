---
title: History, journal, and sessions
description: An undo tree, an append-only command journal, macros, workspace persistence, crash recovery, and the first DevTools slice.
---

# History, journal, and sessions

0.5 adds the things that make an application trustworthy: undo that never loses an
abandoned branch, a journal of every command, macros recorded from that journal, a
workspace that survives a restart, and recovery after a crash.

## Undo tree

`History<S>` is a tree of state snapshots, not a stack. Undo moves to the parent; editing
after an undo adds a sibling, so the abandoned path is still reachable.

<!-- compile -->
```rust
use egui_workbench::app::History;

fn main() {
    let mut history = History::new("a".to_owned());
    history.record("type b", "ab".to_owned());
    let third = history.record("type c", "abc".to_owned());
    history.undo();
    let branch = history.record("branch", "a!".to_owned());
    assert_eq!(history.state(), "a!");
    assert!(history.node(third).is_some(), "the abandoned branch remains");
    assert_eq!(history.node(branch).map(|node| node.state.as_str()), Some("a!"));
    history.redo();
    assert_eq!(history.state(), "a!");
}
```

A group coalesces typing or a drag into one node: `begin_group("typing")`, record while
the group is open (the current node updates in place), then `end_group`. The History panel
draws the tree with `show_history_tree`; a click returns a `NodeId` and `History::jump_to`
time-travels.

## Command journal

`CommandJournal` appends each command with its arguments and a timestamp. It is
versioned, serializes to JSON, saves atomically, and can be loaded after a restart.

```rust
use egui_workbench::app::CommandJournal;

let mut journal = CommandJournal::new();
journal.append("file.new");
journal.append_with("file.save", serde_json::json!({"path": "a.rs"}), "Save");
assert_eq!(journal.len(), 2);
let json = journal.to_json().expect("serialize");
let back = CommandJournal::from_json(&json).expect("parse");
assert_eq!(back, journal);
```

## Macros

`MacroLibrary::record_from(&journal, after_sequence, "Align and fit")` turns a journal
range into a named macro with a stable `macro.<name>` command id. `register` adds it to a
`CommandRegistry`; `Macro::replay` feeds each recorded `(command, args)` to your handler
in order. Because replay does not use a window or a clock, a recorded session is also a
deterministic regression test.

```rust
use egui_workbench::app::{CommandJournal, MacroLibrary};

let mut journal = CommandJournal::new();
journal.append("file.new");
journal.append("edit.type");
let mut library = MacroLibrary::new();
let macro_ = library.record_from(&journal, 0, "New and type").clone();
let mut replayed = Vec::new();
macro_.replay(|step| replayed.push(step.command.to_string()));
assert_eq!(replayed, ["file.new", "edit.type"]);
```

## Workspace persistence

`Workspace` stores open documents, the active tab, explorer expansion, panel sizes and
visibility, the theme preferences, and plot views in one versioned file. Writes go
through a temporary sibling and a rename, so a crash leaves either the old file or the new
one.

```rust
use egui_workbench::app::{Workspace, WorkspaceDocument};

let mut workspace = Workspace::new();
workspace.add_document(WorkspaceDocument::new("a", "A"));
workspace.theme = egui_workbench::ThemeId::Dracula;
workspace.sidebar_width = 260.0;
let json = workspace.to_json().expect("serialize");
let back = Workspace::from_json(&json).expect("parse");
assert_eq!(back, workspace);
```

`from_json` accepts version 0 (which had no version field) and fills every field added
since with `serde(default)`. A file from a newer version is rejected with
`SessionError::UnsupportedVersion`.

## Hot exit and crash recovery

`Stash::capture(now, dirty_buffers)` stores unsaved text; `HotExit` is the timer that
writes the workspace, stash, and journal every few seconds and once more on close.
`recover(directory)` reads the three files, keeps what parses, and records a warning for a
corrupt file instead of failing the whole restore. `Recovery::notice()` is the line to
show the user: how many tabs, unsaved buffers, and journal entries came back.

```text
workspace.json   documents, active tab, explorer, panels, theme, plot views
stash.json       unsaved buffers
journal.json     commands, for replay and the journal viewer
```

## DevTools, first slice

`DevToolsState::toggle_on_f12(ctx)` opens a window with two tabs: the journal viewer and
a JSON state inspector. `show_devtools` takes the journal and any
`serde_json::to_value(&app_state)` snapshot. The full DevTools (widget inspector, live
style editor, performance overlay, accessibility audit) arrive in 0.14.

```rust
use egui_workbench::app::{CommandJournal, DevToolsState, show_devtools};

fn devtools(ctx: &egui::Context, state: &mut DevToolsState, journal: &CommandJournal, snapshot: &serde_json::Value) {
    show_devtools(ctx, state, journal, snapshot, Default::default());
}
```
