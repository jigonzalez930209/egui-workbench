//! A typed select box.

use egui::{Role, Ui, WidgetInfo};

use crate::components::Component;

/// A 170 pt (adjustable) combo box over typed options.
///
/// The label is not drawn; it names the box for screen readers and salts its id. Pair
/// it with [`crate::components::Label`] or a settings row when a visible label is needed.
///
/// ```
/// # use egui_workbench::components::{Select, prelude::*};
/// # let ctx = egui::Context::default();
/// # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let options = [(0, "Small".to_owned()), (1, "Large".to_owned())];
/// let mut size = 0;
/// if Select::new("Size", &mut size, &options).show(ui) {
///     // size changed
/// }
/// # });
/// # output.textures_delta.clear();
/// ```
pub struct Select<'a, T> {
    label: &'a str,
    value: &'a mut T,
    options: &'a [(T, String)],
    width: f32,
    enabled: bool,
}

impl<'a, T> Select<'a, T> {
    /// A select bound to `value`.
    pub fn new(label: &'a str, value: &'a mut T, options: &'a [(T, String)]) -> Self {
        Self {
            label,
            value,
            options,
            width: 170.0,
            enabled: true,
        }
    }

    /// Width in points.
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Disable the select.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

impl<T: PartialEq + Copy> Component for Select<'_, T> {
    type Output = bool;

    fn show(self, ui: &mut Ui) -> bool {
        ui.add_enabled_ui(self.enabled, |ui| {
            let before = *self.value;
            let selected = self
                .options
                .iter()
                .find(|(option, _)| *option == *self.value)
                .map(|(_, label)| label.clone())
                .unwrap_or_default();
            let response = egui::ComboBox::from_id_salt(("component_select", self.label))
                .width(self.width)
                .selected_text(selected)
                .show_ui(ui, |ui| {
                    for (option, text) in self.options {
                        ui.selectable_value(self.value, *option, text);
                    }
                })
                .response;
            response.widget_info(|| WidgetInfo::labeled(Role::ComboBox, self.enabled, self.label));
            *self.value != before
        })
        .inner
    }
}
