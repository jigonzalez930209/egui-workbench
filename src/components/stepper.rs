//! A numeric stepper.

use std::ops::RangeInclusive;

use egui::Ui;

use crate::Icons;
use crate::components::IconComponent;
use crate::numeric_stepper;

/// A numeric field with stacked up/down chevrons.
///
/// ```
/// # use egui_workbench::components::{Stepper, prelude::*};
/// # let ctx = egui::Context::default();
/// # let mut icons = egui_workbench::Icons::new();
/// # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let mut scale = 1.0;
/// if Stepper::new(&mut scale, 0.5..=2.0, 0.1).suffix("×").show(ui, &mut icons) {
///     // scale changed
/// }
/// # });
/// # output.textures_delta.clear();
/// ```
pub struct Stepper<'a> {
    value: &'a mut f32,
    range: RangeInclusive<f32>,
    step: f32,
    width: f32,
    decimals: usize,
    suffix: &'a str,
    enabled: bool,
}

impl<'a> Stepper<'a> {
    /// A stepper bound to `value`, clamped to `range`.
    pub fn new(value: &'a mut f32, range: RangeInclusive<f32>, step: f32) -> Self {
        Self {
            value,
            range,
            step,
            width: 112.0,
            decimals: 0,
            suffix: "",
            enabled: true,
        }
    }

    /// Field width in points.
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Decimal places in the value.
    pub fn decimals(mut self, decimals: usize) -> Self {
        self.decimals = decimals;
        self
    }

    /// Text after the value, such as `%` or `×`.
    pub fn suffix(mut self, suffix: &'a str) -> Self {
        self.suffix = suffix;
        self
    }

    /// Disable the stepper.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

impl IconComponent for Stepper<'_> {
    type Output = bool;

    fn show(self, ui: &mut Ui, icons: &mut Icons) -> bool {
        ui.add_enabled_ui(self.enabled, |ui| {
            let decimals = self.decimals;
            let suffix = self.suffix;
            numeric_stepper(ui, icons, self.value, self.range, self.step, self.width, |value| {
                format!("{value:.decimals$}{suffix}")
            })
        })
        .inner
    }
}
