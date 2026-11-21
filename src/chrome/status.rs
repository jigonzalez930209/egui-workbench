//! The status bar, the empty state, and the editor scroll style.

use egui::{Layout, RichText, Ui};

/// One status-bar cell.
#[derive(Clone, Debug)]
pub struct StatusItem {
    /// Small text.
    pub text: String,
    /// Hover text; empty for none.
    pub tooltip: String,
    /// Paint in the warning color ("Large file").
    pub warning: bool,
}

impl StatusItem {
    /// A plain cell.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            tooltip: String::new(),
            warning: false,
        }
    }

    /// The same cell with a tooltip.
    pub fn tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = tooltip.into();
        self
    }

    /// The same cell in the warning color.
    pub fn warning(mut self) -> Self {
        self.warning = true;
        self
    }

    fn show(&self, ui: &mut Ui) {
        let mut text = RichText::new(&self.text).small();
        if self.warning {
            text = text.color(ui.visuals().warn_fg_color);
        }
        let response = ui.label(text);
        if !self.tooltip.is_empty() {
            response.on_hover_text(&self.tooltip);
        }
    }
}

/// Draw the status bar inside a `Panel::bottom` of [`crate::metrics::STATUS_HEIGHT`]:
/// `left` cells in order (path, saved state, cursor), `right` cells from the right edge
/// inward (encoding, line endings, indentation, language), separated by thin rules.
pub fn show_status_bar(ui: &mut Ui, left: &[StatusItem], right: &[StatusItem]) {
    ui.horizontal_centered(|ui| {
        for (index, item) in left.iter().enumerate() {
            if index > 0 {
                ui.separator();
            }
            item.show(ui);
        }
        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
            for (index, item) in right.iter().enumerate() {
                if index > 0 {
                    ui.separator();
                }
                item.show(ui);
            }
        });
    });
}

/// Shorten a path from the left to `max` characters with a leading "…".
pub fn ellipsize_start(text: &str, max: usize) -> String {
    let count = text.chars().count();
    if count <= max {
        return text.to_owned();
    }
    let tail: String = text.chars().skip(count - (max - 1)).collect();
    format!("…{tail}")
}

/// The empty state: a heading 24 % down the panel, a subtitle, one button, and a hint.
/// Returns true when the button is clicked.
pub fn show_empty_state(ui: &mut Ui, heading: &str, subtitle: &str, button: &str, hint: &str) -> bool {
    let mut clicked = false;
    ui.vertical_centered(|ui| {
        ui.add_space((ui.available_height() * 0.24).max(24.0));
        ui.heading(heading);
        ui.label(subtitle);
        ui.add_space(16.0);
        clicked = ui.button(button).clicked();
        ui.add_space(8.0);
        ui.label(hint);
    });
    clicked
}

/// The scroll style of editors and long documents: floating bars whose handle stays
/// visible at rest (55 % opacity), so a long line always shows its horizontal bar,
/// without the solid bars' change to the viewport size.
pub fn editor_scroll_style(base: &egui::style::ScrollStyle) -> egui::style::ScrollStyle {
    egui::style::ScrollStyle {
        floating: true,
        floating_width: base.floating_width.max(6.0),
        bar_width: base.bar_width.max(10.0),
        handle_min_length: base.handle_min_length.max(24.0),
        dormant_background_opacity: 0.0,
        active_background_opacity: 0.3,
        interact_background_opacity: 0.5,
        dormant_handle_opacity: 0.55,
        active_handle_opacity: 0.75,
        interact_handle_opacity: 1.0,
        ..*base
    }
}

/// Run `add` with [`editor_scroll_style`], restoring the parent style afterwards.
pub fn with_editor_scrollbars<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R {
    let parent = ui.spacing().scroll;
    ui.spacing_mut().scroll = editor_scroll_style(&parent);
    let result = add(ui);
    ui.spacing_mut().scroll = parent;
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn long_paths_keep_their_tail() {
        assert_eq!(ellipsize_start("short", 10), "short");
        assert_eq!(ellipsize_start("/home/user/project/file.md", 10), "…t/file.md");
    }

    #[test]
    fn editor_scrollbars_stay_visible_at_rest() {
        let style = editor_scroll_style(&egui::style::ScrollStyle::default());
        assert!(style.floating && style.dormant_handle_opacity > 0.5);
    }
}
