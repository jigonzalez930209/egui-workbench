//! Typed actions for the egui-workbench agent and remote clients.

use serde::{Deserialize, Serialize};

use egui_workbench_chem_core::AtomId;

use crate::math::Vec3;
use crate::render::RenderOptions;
use crate::view::Mol3dView;
use crate::volume::OrbitalKind;

/// One 3D viewer action that an agent or remote client can request.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "action", content = "args", rename_all = "snake_case")]
#[non_exhaustive]
pub enum Mol3dAction {
    /// Load a CIF from its text.
    LoadCif {
        /// CIF contents.
        text: String,
    },
    /// Expand the current scene into a supercell.
    SetSupercell {
        /// Replicas along a.
        na: usize,
        /// Replicas along b.
        nb: usize,
        /// Replicas along c.
        nc: usize,
    },
    /// Detect coordination polyhedra.
    TogglePolyhedra {
        /// Metal symbols; empty means every atom.
        metals: Vec<String>,
        /// Neighbor cutoff in Angstrom.
        cutoff: f64,
    },
    /// Frame a set of atoms.
    FocusAtoms {
        /// Atoms to frame; empty frames the whole scene.
        atoms: Vec<AtomId>,
    },
    /// Sample and display an orbital isosurface.
    RenderOrbital {
        /// Orbital.
        orbital: OrbitalKind,
        /// Isovalue.
        isovalue: f64,
        /// Grid resolution per axis.
        resolution: usize,
    },
    /// Measure a distance and draw it.
    MeasureDistance {
        /// First atom.
        a: AtomId,
        /// Second atom.
        b: AtomId,
    },
    /// Show or hide a Miller plane.
    ToggleMillerPlane {
        /// h.
        h: i32,
        /// k.
        k: i32,
        /// l.
        l: i32,
    },
    /// Capture a PNG figure.
    CapturePng {
        /// Width in pixels.
        width: u32,
        /// Height in pixels.
        height: u32,
        /// Transparent background.
        transparent: bool,
    },
}

/// What an action produced.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ActionOutcome {
    /// Human-readable result.
    pub message: String,
    /// PNG bytes for `capture_png`.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub png: Option<Vec<u8>>,
}

impl Mol3dView {
    /// Apply an agent action.
    ///
    /// # Errors
    /// When the input file or an atom id is invalid.
    pub fn apply_action(&mut self, action: &Mol3dAction) -> Result<ActionOutcome, String> {
        match action {
            Mol3dAction::LoadCif { text } => {
                let scene = crate::cif::parse_cif(text).map_err(|error| error.to_string())?;
                self.scene = scene;
                self.surfaces.clear();
                self.polyhedra.clear();
                self.measurements.clear();
                self.fit(egui::Rect::from_min_size(
                    egui::pos2(0.0, 0.0),
                    egui::vec2(800.0, 600.0),
                ));
                Ok(outcome(format!("loaded {} atoms", self.scene.atoms.len())))
            }
            Mol3dAction::SetSupercell { na, nb, nc } => {
                self.expand_supercell(*na, *nb, *nc);
                Ok(outcome(format!(
                    "supercell {}x{}x{}: {} atoms",
                    na,
                    nb,
                    nc,
                    self.scene.atoms.len()
                )))
            }
            Mol3dAction::TogglePolyhedra { metals, cutoff } => {
                let symbols = metals.iter().map(String::as_str).collect::<Vec<_>>();
                self.toggle_polyhedra(&symbols, *cutoff);
                Ok(outcome(format!("{} polyhedra", self.polyhedra.len())))
            }
            Mol3dAction::FocusAtoms { atoms } => {
                self.focus_atoms(atoms)?;
                Ok(outcome(format!("focused {} atoms", atoms.len())))
            }
            Mol3dAction::RenderOrbital {
                orbital,
                isovalue,
                resolution,
            } => {
                self.show_orbital(*orbital, self.scene_center(), *isovalue, *resolution);
                Ok(outcome(format!("{} surface layers", self.surfaces.len())))
            }
            Mol3dAction::MeasureDistance { a, b } => {
                let value = self
                    .measure_distance(*a, *b)
                    .ok_or_else(|| "unknown atom in distance".to_owned())?;
                Ok(outcome(format!("{value:.2} Å")))
            }
            Mol3dAction::ToggleMillerPlane { h, k, l } => {
                self.toggle_miller_plane(*h, *k, *l);
                Ok(outcome(format!("{} Miller planes", self.miller_planes.len())))
            }
            Mol3dAction::CapturePng {
                width,
                height,
                transparent,
            } => {
                let png = self.capture_png(*width, *height, *transparent)?;
                Ok(ActionOutcome {
                    message: format!("PNG {}x{} ({} bytes)", width, height, png.len()),
                    png: Some(png),
                })
            }
        }
    }

    /// The centroid of the current scene.
    pub fn scene_center(&self) -> Vec3 {
        crate::render::figure_center(&self.scene).unwrap_or(Vec3::ZERO)
    }

    /// Frame `atoms`, or the whole scene when the list is empty.
    ///
    /// # Errors
    /// When an atom id is unknown.
    pub fn focus_atoms(&mut self, atoms: &[AtomId]) -> Result<(), String> {
        if atoms.is_empty() {
            self.fit(egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(800.0, 600.0),
            ));
            return Ok(());
        }
        let mut min = Vec3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY);
        let mut max = Vec3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
        for id in atoms {
            let atom = self
                .scene
                .atom(*id)
                .ok_or_else(|| format!("unknown atom {}", id.slot()))?;
            min = Vec3::new(
                min.x.min(atom.position.x),
                min.y.min(atom.position.y),
                min.z.min(atom.position.z),
            );
            max = Vec3::new(
                max.x.max(atom.position.x),
                max.y.max(atom.position.y),
                max.z.max(atom.position.z),
            );
        }
        let center = (min + max) * 0.5;
        let extent = (max - min).length().max(1.0);
        self.camera.target = center;
        self.camera.distance = extent * 1.6;
        Ok(())
    }
}

/// A PNG capture through the agent API.
pub fn capture_action(width: u32, height: u32, transparent: bool) -> Mol3dAction {
    Mol3dAction::CapturePng {
        width,
        height,
        transparent,
    }
}

/// The default agent capture options for a publication figure.
pub fn figure_options() -> RenderOptions {
    RenderOptions::figure_4k()
}

fn outcome(message: String) -> ActionOutcome {
    ActionOutcome { message, png: None }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::Mol3dScene;
    use egui_workbench_chem_core::Element;

    #[test]
    fn actions_round_trip_through_json() {
        let action = Mol3dAction::SetSupercell { na: 2, nb: 2, nc: 1 };
        let json = serde_json::to_string(&action).expect("serialize");
        let restored: Mol3dAction = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored, action);
    }

    #[test]
    fn focus_and_capture_work_on_a_small_scene() {
        let mut scene = Mol3dScene::default();
        let atom = scene.push_atom(Element::from_symbol("C").expect("C"), [1.0, 2.0, 3.0]);
        let mut view = Mol3dView::new(scene);
        view.focus_atoms(&[atom]).expect("focus");
        assert_eq!(view.camera.target, Vec3::new(1.0, 2.0, 3.0));
        let outcome = view
            .apply_action(&Mol3dAction::CapturePng {
                width: 32,
                height: 32,
                transparent: true,
            })
            .expect("capture");
        assert!(outcome.png.expect("png").starts_with(b"\x89PNG"));
    }
}
