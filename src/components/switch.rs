//! A labelled toggle switch.

use egui::{RichText, Ui};

use crate::components::Component;
use crate::switch;

/// A 36×19 pt switch with an optional visible label.
///
/// The label is the AccessKit name; it is also drawn to the right of the control when
/// it is not empty. Use `settings_switch` when the label belongs to a settings row.
pub struct Switch<'a> {
    value: &'a mut bool,
    label: &'a str,
    enabled: bool,
}

impl<'a> Switch<'a> {
    /// A switch bound to `value`.
    pub fn new(value: &'a mut bool) -> Self {
        Self {
            value,
            label: "",
            enabled: true,
        }
    }

    /// The accessible name and the optional visible label.
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = label;
        self
    }

    /// Disable interaction.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

impl Component for Switch<'_> {
    type Output = bool;

    fn show(self, ui: &mut Ui) -> bool {
        ui.add_enabled_ui(self.enabled, |ui| {
            if self.label.is_empty() {
                switch(ui, self.value, self.label)
            } else {
                ui.horizontal(|ui| {
                    let changed = switch(ui, self.value, self.label);
                    if ui
                        .add(
                            egui::Label::new(
                                RichText::new(self.label).size(crate::style::get(ui.ctx()).font_size(13.0)),
                            )
                            .sense(egui::Sense::click()),
                        )
                        .clicked()
                    {
                        *self.value = !*self.value;
                        true
                    } else {
                        changed
                    }
                })
                .inner
            }
        })
        .inner
    }
}
