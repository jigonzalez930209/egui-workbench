# Migrating from 0.4 to 0.5

0.5 is additive. The 0.4 command registry, palette, serde model, and features keep their
signatures.

## New modules

- `app::History<S>`: undo tree, groups, and `jump_to`.
- `app::CommandJournal`, `app::Macro`, `app::MacroLibrary`: journal and macros.
- `app::Workspace`, `app::WorkspaceDocument`, `app::WorkspacePanels`,
  `app::WorkspacePlotView`: versioned session file.
- `app::Stash`, `app::HotExit`, `app::Recovery`, `app::recover`: hot exit and recovery.
- `app::DevToolsState`, `app::show_devtools`: F12 journal and state inspector.

## Session directory

The examples use three files in one directory:

```text
workspace.json
stash.json
journal.json
```

`Workspace::save`, `Stash::save`, and `CommandJournal::save` write through a temporary
sibling and rename. `recover(directory)` reads all three; a corrupt file becomes a
warning instead of blocking the rest.

## Feature gates

The session, journal, recovery, and DevTools modules are behind the `serde` feature (on
by default). A `default-features = false` build that wants them enables `serde`.

## No breaking changes

There are no removed or renamed public items in 0.5. The only behavior change is that the
starter application records every handled command in its journal and shows DevTools on
F12.
