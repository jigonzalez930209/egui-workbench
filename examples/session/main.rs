#![allow(clippy::expect_used)]

//! History, journal, workspace persistence, and DevTools in one small application.
//!
//! Run with `cargo run --example session`. Edit the buffer, undo and redo, press F12 for
//! the journal and state inspector, then close and reopen: the workspace, journal, and
//! any unsaved buffer come back from the session directory.

use egui_workbench::ThemeId;
use egui_workbench::app::{
    CommandJournal, DevToolsState, History, HotExit, SessionPaths, Stash, StashDocument, Workspace, WorkspaceDocument,
    recover, show_devtools, show_history_tree,
};
use egui_workbench::commands::CommandId;
use egui_workbench::eframe;
use egui_workbench::egui;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    eframe::run_native(
        "Session",
        egui_workbench::native_options("Session"),
        Box::new(|cc| {
            egui_workbench::theme::apply_id(&cc.egui_ctx, ThemeId::GithubDark);
            let _ = cc;
            Ok(Box::new(Session::new()))
        }),
    )
}

struct Session {
    text: String,
    history: History<String>,
    journal: CommandJournal,
    workspace: Workspace,
    devtools: DevToolsState,
    paths: SessionPaths,
    hot_exit: HotExit,
    last_edit: f64,
    notice: Option<String>,
}

impl Session {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(".egui-workbench-session-example");
        let paths = SessionPaths::new(directory);
        let recovery = recover(&paths.directory);
        let workspace = recovery.workspace.clone().unwrap_or_default();
        let text = recovery
            .stash
            .as_ref()
            .and_then(|stash| stash.documents.first())
            .map(|document| document.text.clone())
            .unwrap_or_default();
        let mut session = Self {
            text: text.clone(),
            history: History::new(text.clone()),
            journal: recovery.journal.clone().unwrap_or_default(),
            workspace,
            devtools: DevToolsState::default(),
            paths,
            hot_exit: HotExit::with_interval(5.0),
            last_edit: 0.0,
            notice: recovery.notice(),
        };
        session.journal.append("session.open");
        session
            .workspace
            .add_document(WorkspaceDocument::new("buffer", "Scratch"));
        session.workspace.documents[0].dirty = !text.is_empty();
        session
    }

    fn record(&mut self, ctx: &egui::Context, label: &str) {
        self.history.record(label.to_owned(), self.text.clone());
        self.journal.append_at(
            ctx.input(|input| input.time),
            CommandId::new("buffer.edit"),
            None,
            Some(label.to_owned()),
        );
        self.last_edit = ctx.input(|input| input.time);
    }

    fn tick(&mut self, ctx: &egui::Context) {
        let now = ctx.input(|input| input.time);
        if self.hot_exit.due(now) {
            let stash = if self.workspace.documents.first().is_some_and(|document| document.dirty) {
                Stash::capture(now, [StashDocument::new("buffer", "Scratch", &self.text)])
            } else {
                Stash::new()
            };
            let _ = self
                .hot_exit
                .save(now, &self.paths, &self.workspace, &stash, &self.journal);
        }
    }
}

impl eframe::App for Session {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.devtools.toggle_on_f12(ctx);
        if self.history.can_undo() && ctx.input_mut(|input| input.consume_key(egui::Modifiers::COMMAND, egui::Key::Z)) {
            self.history.undo();
            self.text = self.history.state().clone();
            self.journal.append("buffer.undo");
        }
        if self.history.can_redo()
            && ctx.input_mut(|input| input.consume_key(egui::Modifiers::COMMAND | egui::Modifiers::SHIFT, egui::Key::Z))
        {
            self.history.redo();
            self.text = self.history.state().clone();
            self.journal.append("buffer.redo");
        }
        self.tick(ctx);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::left("history").default_size(200.0).show(ui, |ui| {
            if let Some(node) = show_history_tree(ui, &self.history, Default::default()) {
                self.history.jump_to(node).expect("the panel returns a real node");
                self.text = self.history.state().clone();
            }
        });
        egui::CentralPanel::default().show(ui, |ui| {
            if let Some(notice) = &self.notice {
                ui.label(egui::RichText::new(notice).color(ui.visuals().warn_fg_color));
            }
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(self.history.can_undo(), egui::Button::new("Undo"))
                    .clicked()
                {
                    self.history.undo();
                    self.text = self.history.state().clone();
                }
                if ui
                    .add_enabled(self.history.can_redo(), egui::Button::new("Redo"))
                    .clicked()
                {
                    self.history.redo();
                    self.text = self.history.state().clone();
                }
                if ui.button("Edit").clicked() {
                    let now = ui.ctx().input(|input| input.time);
                    self.text.push('!');
                    if now - self.last_edit > 0.8 {
                        self.history.begin_group("typing").ok();
                        self.record(ui.ctx(), "edit");
                        self.history.end_group().ok();
                    } else {
                        self.record(ui.ctx(), "edit");
                    }
                }
            });
            let response = ui.add(
                egui::TextEdit::multiline(&mut self.text)
                    .desired_rows(24)
                    .desired_width(f32::INFINITY),
            );
            if response.changed() {
                self.record(ui.ctx(), "typing");
                if let Some(document) = self.workspace.documents.first_mut() {
                    document.dirty = true;
                }
            }
        });
        let snapshot = serde_json::json!({
            "history_nodes": self.history.len(),
            "journal_entries": self.journal.len(),
            "dirty": self.workspace.documents.first().is_some_and(|document| document.dirty),
        });
        show_devtools(
            ui.ctx(),
            &mut self.devtools,
            &self.journal,
            &snapshot,
            Default::default(),
        );
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        let stash = if self.workspace.documents.first().is_some_and(|document| document.dirty) {
            Stash::capture(0.0, [StashDocument::new("buffer", "Scratch", &self.text)])
        } else {
            Stash::new()
        };
        let _ = self.workspace.save(self.paths.workspace());
        let _ = stash.save(self.paths.stash());
        let _ = self.journal.save(self.paths.journal());
    }
}

// A wasm build has no session directory; the example is native-only.
#[cfg(target_arch = "wasm32")]
fn main() {}
