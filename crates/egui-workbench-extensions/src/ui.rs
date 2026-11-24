//! The extension manager view. The application owns the manager; this view returns the
//! action to apply after the frame.

use egui::{RichText, Ui};

use crate::ExtensionInfo;

/// Persistent manager view state.
#[derive(Clone, Debug, Default)]
pub struct ExtensionManagerState {
    /// Filter by name or id.
    pub filter: String,
    /// Selected extension id.
    pub selected: Option<String>,
}

/// What the manager view wants the application to do.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ExtensionManagerAction {
    /// Enable an extension.
    Enable(String),
    /// Disable an extension.
    Disable(String),
    /// Reload an extension after a build.
    Reload(String),
    /// Grant a capability.
    Grant {
        /// Extension id.
        id: String,
        /// Capability spelling.
        capability: String,
    },
}

/// Show the manager. `infos` comes from `ExtensionManager::list()`.
pub fn show_extension_manager(
    ui: &mut Ui,
    state: &mut ExtensionManagerState,
    infos: &[ExtensionInfo],
) -> Option<ExtensionManagerAction> {
    let mut action = None;
    ui.horizontal(|ui| {
        ui.label(RichText::new("Extensions").strong());
        ui.add(egui::TextEdit::singleline(&mut state.filter).hint_text("Filter extensions…"));
    });
    ui.separator();
    egui::ScrollArea::vertical().show(ui, |ui| {
        let filter = state.filter.to_lowercase();
        let visible = infos.iter().filter(|info| {
            filter.is_empty() || info.name.to_lowercase().contains(&filter) || info.id.to_lowercase().contains(&filter)
        });
        for info in visible {
            let selected = state.selected.as_deref() == Some(info.id.as_str());
            if ui
                .selectable_label(selected, format!("{}  {}", info.name, info.version))
                .clicked()
            {
                state.selected = Some(info.id.clone());
            }
            ui.indent(&info.id, |ui| {
                ui.label(RichText::new(&info.id).small().weak());
                ui.horizontal(|ui| {
                    if info.enabled {
                        ui.label(RichText::new("enabled").color(ui.visuals().hyperlink_color));
                        if ui.button("Disable").clicked() {
                            action = Some(ExtensionManagerAction::Disable(info.id.clone()));
                        }
                    } else if ui.button("Enable").clicked() {
                        action = Some(ExtensionManagerAction::Enable(info.id.clone()));
                    }
                    if ui.button("Reload").clicked() {
                        action = Some(ExtensionManagerAction::Reload(info.id.clone()));
                    }
                });
                if !info.requested.is_empty() {
                    ui.label(RichText::new("Requested capabilities").small());
                    for capability in &info.requested {
                        let granted = info.granted.iter().any(|granted| granted.contains(capability));
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(capability).monospace());
                            if granted {
                                ui.label(RichText::new("granted").small().weak());
                            } else if ui.button("Grant").clicked() {
                                action = Some(ExtensionManagerAction::Grant {
                                    id: info.id.clone(),
                                    capability: capability.clone(),
                                });
                            }
                        });
                    }
                }
                if !info.commands.is_empty() || !info.panels.is_empty() || !info.viewers.is_empty() {
                    ui.label(
                        RichText::new(format!(
                            "{} commands, {} panels, {} viewers",
                            info.commands.len(),
                            info.panels.len(),
                            info.viewers.len()
                        ))
                        .small(),
                    );
                }
            });
            ui.separator();
        }
    });
    action
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CommandContribution;

    #[test]
    fn the_manager_lists_and_returns_actions() {
        let ctx = egui::Context::default();
        let mut state = ExtensionManagerState::default();
        let infos = vec![ExtensionInfo {
            id: "com.example.hello".to_owned(),
            name: "Hello".to_owned(),
            version: "0.1.0".to_owned(),
            enabled: false,
            requested: vec!["commands".to_owned()],
            granted: Vec::new(),
            commands: vec![CommandContribution {
                id: "hello.run".to_owned(),
                title: "Hello".to_owned(),
            }],
            panels: Vec::new(),
            viewers: Vec::new(),
        }];
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            let _ = show_extension_manager(ui, &mut state, &infos);
        });
        output.textures_delta.clear();
    }
}
