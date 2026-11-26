//! Interactive 3D geometry measurements.

use egui_workbench_chem_core::AtomId;

use crate::math::Vec3;
use crate::scene::Mol3dScene;

/// A measurement over one or more atoms.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "measurement", rename_all = "snake_case")]
#[non_exhaustive]
pub enum Measurement {
    /// Distance between two atoms.
    Distance {
        /// First atom.
        a: AtomId,
        /// Second atom.
        b: AtomId,
    },
    /// Bond angle at `b` in degrees.
    Angle {
        /// First atom.
        a: AtomId,
        /// Vertex atom.
        b: AtomId,
        /// Third atom.
        c: AtomId,
    },
    /// Dihedral angle `a-b-c-d` in degrees, in `(-180, 180]`.
    Dihedral {
        /// First atom.
        a: AtomId,
        /// Second atom.
        b: AtomId,
        /// Third atom.
        c: AtomId,
        /// Fourth atom.
        d: AtomId,
    },
    /// Centroid of a ring or fragment.
    Centroid {
        /// Atoms whose centroid is measured.
        atoms: Vec<AtomId>,
    },
    /// Least-squares mean plane through a fragment.
    Plane {
        /// Atoms defining the plane.
        atoms: Vec<AtomId>,
    },
}

/// The evaluated value of a measurement.
#[derive(Clone, Debug, PartialEq)]
pub struct MeasurementValue {
    /// Numeric value: Angstrom for distances, degrees for angles.
    pub value: f64,
    /// Unit symbol.
    pub unit: &'static str,
    /// Display label, such as `1.42 Å` or `109.5°`.
    pub label: String,
    /// Points used to draw the measurement on screen.
    pub points: Vec<Vec3>,
    /// Centroid or plane point, when relevant.
    pub anchor: Option<Vec3>,
    /// Mean-plane normal, when relevant.
    pub normal: Option<Vec3>,
}

impl Measurement {
    /// Evaluate the measurement against `scene`.
    ///
    /// Returns `None` when an atom is missing or the geometry is degenerate.
    pub fn evaluate(&self, scene: &Mol3dScene) -> Option<MeasurementValue> {
        match self {
            Self::Distance { a, b } => {
                let (a, b) = (position(scene, *a)?, position(scene, *b)?);
                let value = (b - a).length();
                Some(value_of(value, "Å", format!("{value:.2} Å"), vec![a, b], None, None))
            }
            Self::Angle { a, b, c } => {
                let (a, b, c) = (position(scene, *a)?, position(scene, *b)?, position(scene, *c)?);
                let first = (a - b).normalized();
                let second = (c - b).normalized();
                if first.length() == 0.0 || second.length() == 0.0 {
                    return None;
                }
                let value = first.dot(second).clamp(-1.0, 1.0).acos().to_degrees();
                Some(value_of(
                    value,
                    "°",
                    format!("{value:.1}°"),
                    vec![a, b, c],
                    Some(b),
                    None,
                ))
            }
            Self::Dihedral { a, b, c, d } => {
                let (a, b, c, d) = (
                    position(scene, *a)?,
                    position(scene, *b)?,
                    position(scene, *c)?,
                    position(scene, *d)?,
                );
                let value = dihedral(a, b, c, d)?;
                Some(value_of(
                    value,
                    "°",
                    format!("{value:.1}°"),
                    vec![a, b, c, d],
                    Some((b + c) * 0.5),
                    None,
                ))
            }
            Self::Centroid { atoms } => {
                let points = positions(scene, atoms)?;
                let centroid = centroid(&points)?;
                Some(value_of(
                    0.0,
                    "",
                    format!("centroid ({:.2}, {:.2}, {:.2})", centroid.x, centroid.y, centroid.z),
                    points,
                    Some(centroid),
                    None,
                ))
            }
            Self::Plane { atoms } => {
                let points = positions(scene, atoms)?;
                let centroid = centroid(&points)?;
                let normal = mean_plane_normal(&points)?;
                let rms = (points
                    .iter()
                    .map(|point| (*point - centroid).dot(normal).powi(2))
                    .sum::<f64>()
                    / points.len() as f64)
                    .sqrt();
                Some(value_of(
                    rms,
                    "Å",
                    format!("plane RMS {rms:.3} Å"),
                    points,
                    Some(centroid),
                    Some(normal),
                ))
            }
        }
    }

    /// Atoms referenced by the measurement.
    pub fn atoms(&self) -> Vec<AtomId> {
        match self {
            Self::Distance { a, b } => vec![*a, *b],
            Self::Angle { a, b, c } => vec![*a, *b, *c],
            Self::Dihedral { a, b, c, d } => vec![*a, *b, *c, *d],
            Self::Centroid { atoms } | Self::Plane { atoms } => atoms.clone(),
        }
    }
}

/// The angle between the planes `a-b-c` and `b-c-d`, in `(-180, 180]`.
pub fn dihedral(a: Vec3, b: Vec3, c: Vec3, d: Vec3) -> Option<f64> {
    let first = b - a;
    let axis = c - b;
    let last = d - c;
    if axis.length() <= 1.0e-9 {
        return None;
    }
    let first = first - axis * (first.dot(axis) / axis.dot(axis));
    let last = last - axis * (last.dot(axis) / axis.dot(axis));
    if first.length() <= 1.0e-9 || last.length() <= 1.0e-9 {
        return None;
    }
    let x = first.dot(last);
    let y = first.cross(last).dot(axis.normalized());
    Some(y.atan2(x).to_degrees())
}

/// The centroid of a non-empty point set.
pub fn centroid(points: &[Vec3]) -> Option<Vec3> {
    if points.is_empty() {
        return None;
    }
    let sum = points.iter().fold(Vec3::ZERO, |sum, point| sum + *point);
    Some(sum * (1.0 / points.len() as f64))
}

/// The normal of the least-squares plane through at least three points.
pub fn mean_plane_normal(points: &[Vec3]) -> Option<Vec3> {
    let center = centroid(points)?;
    if points.len() < 3 {
        return None;
    }
    let mut covariance = [[0.0; 3]; 3];
    for point in points {
        let delta = *point - center;
        let values = [delta.x, delta.y, delta.z];
        for row in 0..3 {
            for column in 0..3 {
                covariance[row][column] += values[row] * values[column];
            }
        }
    }
    let (values, vectors) = jacobi_eigen(covariance);
    let smallest = values
        .iter()
        .enumerate()
        .min_by(|(_, left), (_, right)| left.total_cmp(right))?
        .0;
    let normal = Vec3::new(vectors[0][smallest], vectors[1][smallest], vectors[2][smallest]).normalized();
    if normal.length() == 0.0 { None } else { Some(normal) }
}

fn position(scene: &Mol3dScene, id: AtomId) -> Option<Vec3> {
    scene.atom(id).map(|atom| atom.position)
}

fn positions(scene: &Mol3dScene, atoms: &[AtomId]) -> Option<Vec<Vec3>> {
    if atoms.is_empty() {
        return None;
    }
    atoms.iter().map(|id| position(scene, *id)).collect()
}

fn value_of(
    value: f64,
    unit: &'static str,
    label: String,
    points: Vec<Vec3>,
    anchor: Option<Vec3>,
    normal: Option<Vec3>,
) -> MeasurementValue {
    MeasurementValue {
        value,
        unit,
        label,
        points,
        anchor,
        normal,
    }
}

#[allow(clippy::needless_range_loop)]
fn jacobi_eigen(mut matrix: [[f64; 3]; 3]) -> ([f64; 3], [[f64; 3]; 3]) {
    let mut vectors = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    for _ in 0..32 {
        let (p, q) = [(0, 1), (0, 2), (1, 2)]
            .into_iter()
            .max_by(|left, right| matrix[left.0][left.1].abs().total_cmp(&matrix[right.0][right.1].abs()))
            .expect("pairs");
        if matrix[p][q].abs() < 1.0e-12 {
            break;
        }
        let angle = 0.5 * (2.0 * matrix[p][q]).atan2(matrix[q][q] - matrix[p][p]);
        let (sin, cos) = angle.sin_cos();
        for index in 0..3 {
            let left = matrix[index][p];
            let right = matrix[index][q];
            matrix[index][p] = cos * left - sin * right;
            matrix[index][q] = sin * left + cos * right;
        }
        for index in 0..3 {
            let left = matrix[p][index];
            let right = matrix[q][index];
            matrix[p][index] = cos * left - sin * right;
            matrix[q][index] = sin * left + cos * right;
        }
        for row in &mut vectors {
            let left = row[p];
            let right = row[q];
            row[p] = cos * left - sin * right;
            row[q] = sin * left + cos * right;
        }
    }
    ([matrix[0][0], matrix[1][1], matrix[2][2]], vectors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_workbench_chem_core::Element;

    fn scene() -> (Mol3dScene, [AtomId; 4]) {
        let mut scene = Mol3dScene::default();
        let ids = [
            scene.push_atom(Element::from_symbol("C").expect("C"), [0.0, 0.0, 0.0]),
            scene.push_atom(Element::from_symbol("C").expect("C"), [1.0, 0.0, 0.0]),
            scene.push_atom(Element::from_symbol("C").expect("C"), [1.0, 1.0, 0.0]),
            scene.push_atom(Element::from_symbol("C").expect("C"), [1.0, 1.0, 1.0]),
        ];
        (scene, ids)
    }

    #[test]
    fn distances_angles_and_dihedrals_are_exact() {
        let (scene, [a, b, c, d]) = scene();
        let distance = Measurement::Distance { a, b }.evaluate(&scene).expect("distance");
        assert!((distance.value - 1.0).abs() < 1.0e-9);
        let angle = Measurement::Angle { a, b, c }.evaluate(&scene).expect("angle");
        assert!((angle.value - 90.0).abs() < 1.0e-9);
        let dihedral = Measurement::Dihedral { a, b, c, d }.evaluate(&scene).expect("dihedral");
        assert!((dihedral.value.abs() - 90.0).abs() < 1.0e-9);
    }

    #[test]
    fn centroid_and_plane_are_recovered() {
        let (scene, [a, b, c, d]) = scene();
        let plane = Measurement::Plane { atoms: vec![a, b, c] }
            .evaluate(&scene)
            .expect("plane");
        assert!(plane.value < 1.0e-9);
        assert_eq!(plane.normal, Some(Vec3::new(0.0, 0.0, 1.0)));
        let centroid = Measurement::Centroid {
            atoms: vec![a, b, c, d],
        }
        .evaluate(&scene)
        .expect("centroid");
        assert_eq!(centroid.anchor, Some(Vec3::new(0.75, 0.5, 0.25)));
    }

    #[test]
    fn missing_atoms_return_none() {
        let (scene, [a, _, _, _]) = scene();
        let missing = Measurement::Distance {
            a,
            b: AtomId::new(99, 1),
        };
        assert!(missing.evaluate(&scene).is_none());
    }
}
