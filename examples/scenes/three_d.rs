//! The live 3D chart: orbit, zoom, legend animation, surfaces, bars, and a helix.

use egui_workbench::components::plot::*;
use egui_workbench::components::prelude::*;
use egui_workbench::{combo, egui};

use super::{Host, Scene};

const KINDS: [(&str, Kind); 3] = [("Surface", Kind::Surface), ("Bars", Kind::Bars), ("Helix", Kind::Helix)];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Surface,
    Bars,
    Helix,
}

pub struct ThreeD {
    state: Plot3DState,
    kind: usize,
    animation: usize,
    easing: usize,
    projection: usize,
    wireframe: bool,
    direction: bool,
    fixed: bool,
}

impl Default for ThreeD {
    fn default() -> Self {
        Self {
            state: Plot3DState::default(),
            kind: 0,
            animation: 0,
            easing: 0,
            projection: 0,
            wireframe: true,
            direction: true,
            fixed: false,
        }
    }
}

impl ThreeD {
    pub fn surface() -> Self {
        Self {
            kind: 0,
            fixed: true,
            ..Self::default()
        }
    }

    pub fn bars() -> Self {
        Self {
            kind: 1,
            fixed: true,
            ..Self::default()
        }
    }

    pub fn helix() -> Self {
        Self {
            kind: 2,
            fixed: true,
            ..Self::default()
        }
    }

    fn config(&self) -> Plot3DConfig {
        let mut config = Plot3DConfig {
            legend: LegendPosition::BottomRight,
            ..Plot3DConfig::default()
        };
        config.legend_hover.animation = LegendAnimation::ALL[self.animation];
        config.legend_hover.easing = AnimationEasing::ALL[self.easing];
        config.projection = if self.projection == 0 {
            Projection::Perspective
        } else {
            Projection::Orthographic
        };
        config.direction.enabled = self.direction;
        config.surface.wireframe = self.wireframe;
        config
    }

    fn plot(&self) -> Plot3D<'static> {
        let plot = Plot3D::new(("plot-3d", self.kind))
            .config(self.config())
            .export_button(true)
            .height(470.0);
        match KINDS[self.kind].1 {
            Kind::Surface => plot
                .title("Terrain surface")
                .x_label("Longitude")
                .y_label("Latitude")
                .z_label("Height")
                .series(Series3D::surface_fn(
                    "terrain",
                    -2.0..=2.0,
                    -2.0..=2.0,
                    [34, 34],
                    |x, y| (x * x + y * y).sin() * 0.7 + (x * 1.2).cos() * 0.3,
                )),
            Kind::Bars => {
                let cells: Vec<[f64; 3]> = (0..7)
                    .flat_map(|x| {
                        (0..7).map(move |y| {
                            let (x, y) = (f64::from(x), f64::from(y));
                            [x, y, 1.0 + (x * 0.8 + y * 0.5).sin() * 1.4 + (x + y) * 0.05]
                        })
                    })
                    .collect();
                plot.title("Revenue by region")
                    .x_label("Region")
                    .y_label("Segment")
                    .z_label("Revenue")
                    .series(Series3D::bars("revenue", cells).bar_width(0.62))
            }
            Kind::Helix => {
                let line: Vec<[f64; 3]> = (0..140)
                    .map(|index| {
                        let angle = index as f64 * 0.16;
                        [angle.cos() * 2.0, angle.sin() * 2.0, index as f64 * 0.045]
                    })
                    .collect();
                let marks: Vec<[f64; 3]> = line.iter().copied().step_by(12).collect();
                plot.title("Streaming helix")
                    .x_label("x")
                    .y_label("y")
                    .z_label("z")
                    .series(Series3D::line("path", line).width(2.0))
                    .series(Series3D::scatter("samples", marks).radius(4.0))
            }
        }
    }
}

impl Scene for ThreeD {
    fn scroll(&self) -> bool {
        false
    }

    fn ui(&mut self, ui: &mut egui::Ui, host: &mut Host<'_>) {
        ui.horizontal_wrapped(|ui| {
            if !self.fixed {
                let kinds: Vec<(usize, String)> = KINDS
                    .iter()
                    .enumerate()
                    .map(|(index, (label, _))| (index, (*label).to_owned()))
                    .collect();
                combo(ui, "Chart", &mut self.kind, &kinds);
            }
            let projections = vec![(0, "Perspective".to_owned()), (1, "Orthographic".to_owned())];
            combo(ui, "Projection", &mut self.projection, &projections);
            if !self.fixed {
                let animations: Vec<(usize, String)> = LegendAnimation::ALL
                    .iter()
                    .enumerate()
                    .map(|(index, animation)| (index, animation.label().to_owned()))
                    .collect();
                combo(ui, "Legend animation", &mut self.animation, &animations);
                let easings: Vec<(usize, String)> = AnimationEasing::ALL
                    .iter()
                    .enumerate()
                    .map(|(index, easing)| (index, easing.label().to_owned()))
                    .collect();
                combo(ui, "Easing", &mut self.easing, &easings);
            }
            if self.kind == 0 {
                Switch::new(&mut self.wireframe).label("Wireframe").show(ui);
            }
            if self.kind == 2 {
                Switch::new(&mut self.direction).label("Direction arrow").show(ui);
            }
        });
        ui.add_space(6.0);
        let plot = self.plot();
        let mut state = std::mem::take(&mut self.state);
        let response = plot.clone().show(ui, host.icons, &mut state);
        self.state = state;
        if let Some(format) = response.export_requested {
            let options = ExportOptions::default();
            let bytes = match format {
                ExportFormat::Svg => plot.to_svg(ui.ctx(), &self.state, &options).into_bytes(),
                ExportFormat::Png => plot.to_png(ui.ctx(), &self.state, &options).unwrap_or_default(),
                _ => Vec::new(),
            };
            let (name, mime) = match format {
                ExportFormat::Svg => ("chart-3d.svg", "image/svg+xml"),
                ExportFormat::Png => ("chart-3d.png", "image/png"),
                _ => ("chart-3d.png", "image/png"),
            };
            if !bytes.is_empty() {
                let place = (host.save)(name, mime, bytes);
                host.report(place);
            }
        }
    }
}
