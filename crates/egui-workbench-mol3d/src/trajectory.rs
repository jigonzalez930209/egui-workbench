//! Molecular dynamics trajectories, optimization paths, and vibrational modes.

use egui_workbench_chem_core::AtomId;

use crate::formats::{FormatError, parse_xyz};
use crate::math::Vec3;
use crate::scene::Mol3dScene;

/// A sequence of molecular frames, such as an MD run, IRC, or NEB path.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Trajectory {
    /// Frames in playback order.
    pub frames: Vec<Mol3dScene>,
    /// Optional per-frame labels.
    pub labels: Vec<String>,
}

impl Trajectory {
    /// Read every frame of a multi-frame XYZ file.
    ///
    /// # Errors
    /// When a frame is malformed.
    pub fn from_xyz(text: &str) -> Result<Self, FormatError> {
        let frames = parse_xyz(text)?;
        let labels = (0..frames.len()).map(|index| format!("Frame {}", index + 1)).collect();
        Ok(Self { frames, labels })
    }

    /// Number of frames.
    pub fn len(&self) -> usize {
        self.frames.len()
    }

    /// Whether there are no frames.
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    /// The frame nearest to normalized time `t` in `[0, 1]`.
    pub fn frame(&self, t: f64) -> Option<&Mol3dScene> {
        if self.frames.is_empty() {
            return None;
        }
        let index = (t.clamp(0.0, 1.0) * (self.frames.len() - 1) as f64).round() as usize;
        self.frames.get(index)
    }

    /// Linearly interpolate between adjacent frames.
    pub fn interpolate(&self, t: f64) -> Option<Mol3dScene> {
        if self.frames.is_empty() {
            return None;
        }
        let clamped = t.clamp(0.0, 1.0) * (self.frames.len() - 1) as f64;
        let index = clamped.floor() as usize;
        let fraction = clamped - index as f64;
        let first = &self.frames[index];
        let Some(second) = self.frames.get(index + 1) else {
            return Some(first.clone());
        };
        let mut scene = first.clone();
        for atom in &mut scene.atoms {
            if let Some(other) = second.atom(atom.id) {
                atom.position = atom.position.lerp(other.position, fraction);
            }
        }
        Some(scene)
    }

    /// Build a trajectory by interpolating `steps` frames between two scenes.
    pub fn between(first: &Mol3dScene, second: &Mol3dScene, steps: usize) -> Self {
        let steps = steps.max(2);
        let frames = (0..steps)
            .map(|index| {
                let t = index as f64 / (steps - 1) as f64;
                let mut scene = first.clone();
                for atom in &mut scene.atoms {
                    if let Some(other) = second.atom(atom.id) {
                        atom.position = atom.position.lerp(other.position, t);
                    }
                }
                scene
            })
            .collect();
        let labels = (0..steps)
            .map(|index| format!("Step {}/{}", index + 1, steps))
            .collect();
        Self { frames, labels }
    }
}

/// One normal mode: a displacement per atom.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VibrationalMode {
    /// Displacements in Angstrom for the equilibrium geometry.
    pub displacements: Vec<(AtomId, Vec3)>,
    /// Frequency in reciprocal centimetres, when known.
    pub frequency: f64,
}

impl VibrationalMode {
    /// A mode from per-atom displacements.
    pub fn new(displacements: Vec<(AtomId, Vec3)>, frequency: f64) -> Self {
        Self {
            displacements,
            frequency,
        }
    }

    /// The largest displacement magnitude.
    pub fn amplitude(&self) -> f64 {
        self.displacements
            .iter()
            .map(|(_, displacement)| displacement.length())
            .fold(0.0, f64::max)
    }
}

/// Apply `mode` at `phase` radians with the given scale, returning a displaced copy.
#[must_use]
pub fn animate_mode(scene: &Mol3dScene, mode: &VibrationalMode, phase: f64, scale: f64) -> Mol3dScene {
    let mut animated = scene.clone();
    for (id, displacement) in &mode.displacements {
        if let Some(atom) = animated.atoms.iter_mut().find(|atom| atom.id == *id) {
            atom.position = atom.position + *displacement * (phase.sin() * scale);
        }
    }
    animated
}

/// Playback state for a trajectory.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrajectoryPlayer {
    /// Normalized time in `[0, 1]`.
    pub time: f64,
    /// Playback speed in normalized time per second.
    pub speed: f64,
    /// Whether time advances.
    pub playing: bool,
}

impl Default for TrajectoryPlayer {
    fn default() -> Self {
        Self {
            time: 0.0,
            speed: 0.2,
            playing: false,
        }
    }
}

impl TrajectoryPlayer {
    /// Advance by `dt` seconds, looping at the end.
    pub fn update(&mut self, dt: f64) {
        if self.playing {
            self.time = (self.time + dt * self.speed).rem_euclid(1.0);
        }
    }

    /// The interpolated scene at the current time.
    pub fn scene(&self, trajectory: &Trajectory) -> Option<Mol3dScene> {
        trajectory.interpolate(self.time)
    }
}

/// Read an extended XYZ with a `Lattice="..."` comment into a periodic scene.
///
/// # Errors
/// When the frames are malformed.
pub fn parse_extended_xyz(text: &str) -> Result<Vec<Mol3dScene>, FormatError> {
    let mut frames = parse_xyz(text)?;
    let mut comments = text
        .lines()
        .enumerate()
        .filter(|(index, line)| index > &0 && line.to_ascii_lowercase().contains("lattice="))
        .map(|(_, line)| line.to_owned());
    for frame in &mut frames {
        let Some(comment) = comments.next() else {
            break;
        };
        let Some(start) = comment.find("Lattice=\"") else {
            continue;
        };
        let rest = &comment[start + 9..];
        let Some(end) = rest.find('"') else {
            continue;
        };
        let numbers = rest[..end]
            .split_whitespace()
            .filter_map(|value| value.parse::<f64>().ok())
            .collect::<Vec<_>>();
        if numbers.len() != 9 {
            continue;
        }
        let vectors = [
            [numbers[0], numbers[1], numbers[2]],
            [numbers[3], numbers[4], numbers[5]],
            [numbers[6], numbers[7], numbers[8]],
        ];
        if let Some(cell) = crate::surface::cell_from_vectors(vectors) {
            frame.cell = Some(cell);
        }
    }
    Ok(frames)
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_workbench_chem_core::Element;

    fn two_frames() -> Trajectory {
        let mut first = Mol3dScene::default();
        first.push_atom(Element::from_symbol("C").expect("C"), [0.0, 0.0, 0.0]);
        let mut second = Mol3dScene::default();
        second.push_atom(Element::from_symbol("C").expect("C"), [2.0, 0.0, 0.0]);
        Trajectory {
            frames: vec![first, second],
            labels: vec!["start".to_owned(), "end".to_owned()],
        }
    }

    #[test]
    fn interpolation_moves_atoms_between_frames() {
        let trajectory = two_frames();
        let middle = trajectory.interpolate(0.5).expect("middle");
        assert!((middle.atoms[0].position.x - 1.0).abs() < 1.0e-9);
        assert_eq!(trajectory.frame(1.0).expect("last").atoms[0].position.x, 2.0);
    }

    #[test]
    fn a_mode_oscillates_both_ways() {
        let trajectory = two_frames();
        let scene = &trajectory.frames[0];
        let mode = VibrationalMode::new(vec![(scene.atoms[0].id, Vec3::new(0.0, 0.3, 0.0))], 1000.0);
        let up = animate_mode(scene, &mode, std::f64::consts::FRAC_PI_2, 1.0);
        let down = animate_mode(scene, &mode, -std::f64::consts::FRAC_PI_2, 1.0);
        assert!(up.atoms[0].position.y > 0.2);
        assert!(down.atoms[0].position.y < -0.2);
    }

    #[test]
    fn playback_advances_and_loops() {
        let mut player = TrajectoryPlayer {
            speed: 0.5,
            playing: true,
            ..TrajectoryPlayer::default()
        };
        player.update(1.0);
        assert!((player.time - 0.5).abs() < 1.0e-9);
        player.update(1.5);
        assert!(player.time < 0.5);
    }
}
