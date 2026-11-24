//! Three editor groups, a floating panel, and a detached viewport.
//!
//! Run with `cargo run --example workspace`. The layout is a serializable
//! [`Layout`]; the split/save/switch buttons only change that model.

#![allow(clippy::expect_used)]

use egui_workbench::ThemeId;
use egui_workbench::app::{EditorGroupId, Layout, LayoutAction, LayoutTab, SplitDirection, Workspace};
use egui_workbench::eframe;
use egui_workbench::egui::{self, RichText};

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    eframe::run_native(
        "Workspace",
        egui_workbench::native_options("Workspace"),
        Box::new(|cc| {
            egui_workbench::theme::apply_id(&cc.egui_ctx, ThemeId::GithubDark);
            Ok(Box::new(WorkspaceExample::new()))
        }),
    )
}

struct WorkspaceExample {
    layout: Layout,
    workspace: Workspace,
    floating: bool,
    detached: bool,
    status: String,
}

impl WorkspaceExample {
    fn new() -> Self {
        let mut layout = Layout::with_tabs(vec![
            LayoutTab::new("a", "analysis.rs"),
            LayoutTab::new("b", "notes.md"),
        ]);
        let second = layout
            .split(EditorGroupId(0), SplitDirection::Horizontal)
            .expect("split");
        let third = layout.split(EditorGroupId(0), SplitDirection::Vertical).expect("split");
        layout.focus(EditorGroupId(0));
        for id in [second, third] {
            if let Some(group) = layout.group_mut(id) {
                group
                    .tabs
                    .push(LayoutTab::new(format!("group-{}", id.0), format!("Group {}", id.0)));
            }
        }
        let mut workspace = Workspace::new();
        workspace.layout = layout.clone();
        workspace.save_layout("Analysis");
        Self {
            layout,
            workspace,
            floating: true,
            detached: false,
            status: "Keyboard: split, focus, move tab, maximize, Zen".to_owned(),
        }
    }

    fn group(&mut self, id: EditorGroupId, ui: &mut egui::Ui) {
        let Some(group) = self.layout.group(id) else {
            return;
        };
        let (tabs, active) = (group.tabs.clone(), group.active);
        let mut clicked = None;
        ui.horizontal(|ui| {
            for (index, tab) in tabs.iter().enumerate() {
                if ui.selectable_label(index == active, &tab.title).clicked() {
                    clicked = Some(index);
                }
            }
        });
        if let Some(index) = clicked {
            self.layout.focus(id);
            if let Some(group) = self.layout.group_mut(id) {
                group.active = index;
            }
        }
        ui.separator();
        if let Some(tab) = self.layout.group(id).and_then(|group| group.active_tab()) {
            ui.label(RichText::new(format!("{} — group {}", tab.title, id.0)).strong());
            ui.label("The editor content for this group lives here.");
        } else {
            ui.label(RichText::new("Empty group · drop a tab here").weak());
        }
    }
}

impl eframe::App for WorkspaceExample {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let groups = self.layout.groups();
        let left = groups.get(2).copied().unwrap_or(groups[0]);
        let right = groups.get(1).copied().unwrap_or(groups[0]);
        let center = groups[0];
        egui::Panel::left("workspace-left").default_size(240.0).show(ui, |ui| {
            self.group(left, ui);
        });
        egui::Panel::right("workspace-right")
            .default_size(240.0)
            .show(ui, |ui| {
                self.group(right, ui);
            });
        egui::Panel::bottom("workspace-status").exact_size(28.0).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.status);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("{} groups · {} tabs", groups.len(), self.layout.groups().len()));
                });
            });
        });
        egui::CentralPanel::default().show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                if ui.button("Split right").clicked() {
                    self.layout.apply(LayoutAction::SplitRight);
                }
                if ui.button("Split down").clicked() {
                    self.layout.apply(LayoutAction::SplitDown);
                }
                if ui.button("Focus next").clicked() {
                    self.layout.apply(LayoutAction::FocusNext);
                }
                if ui.button("Move tab").clicked() {
                    self.layout.apply(LayoutAction::MoveTabToNextGroup);
                }
                if ui.button("Maximize").clicked() {
                    self.layout.apply(LayoutAction::ToggleMaximize);
                }
                if ui.button("Zen").clicked() {
                    self.layout.apply(LayoutAction::ToggleZen);
                }
                if ui.button("Save layout").clicked() {
                    self.workspace.layout = self.layout.clone();
                    self.status = format!("Saved layout “{}”", self.workspace.save_layout("Analysis"));
                }
                if ui.button("Floating panel").clicked() {
                    self.floating = !self.floating;
                }
                #[cfg(not(target_arch = "wasm32"))]
                if ui.button("Detached window").clicked() {
                    self.detached = true;
                }
            });
            ui.separator();
            self.group(center, ui);
            if self.layout.zen {
                ui.label(RichText::new("Zen mode: the side bars and status bar are hidden.").weak());
            }
        });
        if self.floating {
            egui::Window::new("Floating inspector")
                .default_pos([420.0, 160.0])
                .default_size([260.0, 180.0])
                .show(ui.ctx(), |ui| {
                    ui.label("A floating panel inside the canvas.");
                    ui.label(format!("Active group: {}", self.layout.active.0));
                });
        }
        #[cfg(not(target_arch = "wasm32"))]
        if self.detached {
            let viewport = egui::ViewportId::from_hash_of("workspace-detached");
            let builder = egui::ViewportBuilder::default()
                .with_title("Detached — Workspace")
                .with_inner_size(egui::vec2(420.0, 320.0));
            ui.ctx().show_viewport_immediate(viewport, builder, |ui, _class| {
                egui::CentralPanel::default().show(ui, |ui| {
                    ui.label(RichText::new("A native detached viewport").strong());
                    ui.label("Dock it back with the button in the main window.");
                });
            });
        }
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui_workbench::theme::Theme::dark()
            .background
            .to_normalized_gamma_f32()
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {}
