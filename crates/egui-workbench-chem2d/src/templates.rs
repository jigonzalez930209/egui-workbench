//! Ring templates, chain building, and angular snapping.

use egui_workbench_chem_core::{AtomId, BondOrder, Element};

use crate::document::Molecule2D;

/// A standard ring stamp.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum RingTemplate {
    /// Benzene.
    Benzene,
    /// Cyclohexane.
    Cyclohexane,
    /// Cyclopentadiene.
    Cyclopentadiene,
    /// Pyridine.
    Pyridine,
    /// Pyrrole.
    Pyrrole,
    /// Cyclohexane chair.
    Chair,
    /// Cyclohexane boat.
    Boat,
}

impl RingTemplate {
    /// Every template, in menu order.
    pub const ALL: [Self; 7] = [
        Self::Benzene,
        Self::Cyclohexane,
        Self::Cyclopentadiene,
        Self::Pyridine,
        Self::Pyrrole,
        Self::Chair,
        Self::Boat,
    ];

    /// The menu label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Benzene => "Benzene",
            Self::Cyclohexane => "Cyclohexane",
            Self::Cyclopentadiene => "Cyclopentadiene",
            Self::Pyridine => "Pyridine",
            Self::Pyrrole => "Pyrrole",
            Self::Chair => "Chair",
            Self::Boat => "Boat",
        }
    }

    /// The atoms in template-local coordinates.
    pub fn atoms(self) -> Vec<(Element, [f64; 2])> {
        let carbon = Element::from_symbol("C").expect("carbon");
        let nitrogen = Element::from_symbol("N").expect("nitrogen");
        match self {
            Self::Benzene => polygon(6, 1.4, carbon),
            Self::Cyclohexane => polygon(6, 1.4, carbon),
            Self::Cyclopentadiene => polygon(5, 1.2, carbon),
            Self::Pyridine => {
                let mut atoms = polygon(6, 1.4, carbon);
                atoms[0].0 = nitrogen;
                atoms
            }
            Self::Pyrrole => {
                let mut atoms = polygon(5, 1.2, carbon);
                atoms[0].0 = nitrogen;
                atoms
            }
            Self::Chair => chair(1.4),
            Self::Boat => boat(1.4),
        }
    }

    /// The bonds, as `(atom, atom, order)` indices into [`Self::atoms`].
    pub fn bonds(self) -> Vec<(usize, usize, BondOrder)> {
        match self {
            Self::Benzene | Self::Pyridine => (0..6)
                .map(|index| (index, (index + 1) % 6, BondOrder::Aromatic))
                .collect(),
            Self::Cyclopentadiene => vec![
                (0, 1, BondOrder::Double),
                (1, 2, BondOrder::Single),
                (2, 3, BondOrder::Double),
                (3, 4, BondOrder::Single),
                (4, 0, BondOrder::Single),
            ],
            // Kekulé form, so the nitrogen has two single bonds and one implicit hydrogen
            // (N-H). Aromatic bonds would give it valence 3 and hide the hydrogen.
            Self::Pyrrole => vec![
                (0, 1, BondOrder::Single),
                (1, 2, BondOrder::Double),
                (2, 3, BondOrder::Single),
                (3, 4, BondOrder::Double),
                (4, 0, BondOrder::Single),
            ],
            Self::Cyclohexane | Self::Chair | Self::Boat => (0..6)
                .map(|index| (index, (index + 1) % 6, BondOrder::Single))
                .collect(),
        }
    }
}

/// Add a template at `center`, rotated by `rotation` radians.
pub fn add_ring(molecule: &mut Molecule2D, template: RingTemplate, center: [f64; 2], rotation: f64) -> Vec<AtomId> {
    let (sine, cosine) = rotation.sin_cos();
    let ids = template
        .atoms()
        .into_iter()
        .map(|(element, [x, y])| {
            let [x, y] = [x * cosine - y * sine, x * sine + y * cosine];
            molecule.add_atom(element, [center[0] + x, center[1] + y, 0.0])
        })
        .collect::<Vec<_>>();
    for (a, b, order) in template.bonds() {
        if let (Some(a), Some(b)) = (ids.get(a), ids.get(b)) {
            let _ = molecule.add_bond(*a, *b, order);
        }
    }
    ids
}

/// Most atoms one chain command adds.
const MAX_CHAIN: usize = 1_000;

/// Add a zig-zag carbon chain starting at `start`.
pub fn add_chain(molecule: &mut Molecule2D, start: [f64; 2], count: usize, angle: f64) -> Vec<AtomId> {
    // A request for a billion carbons would allocate until the process dies.
    let count = count.clamp(1, MAX_CHAIN);
    let carbon = Element::from_symbol("C").expect("carbon");
    let step = 1.5;
    let mut ids = Vec::with_capacity(count);
    // Zigzag: every bond is `step` long and leans 30 degrees to one side of `angle`, then
    // the other, which gives the 120 degree bond angle of an alkane. Placing atom `i` at
    // `i * step` along alternating rays made the bonds between them uneven.
    let mut position = start;
    for index in 0..count {
        ids.push(molecule.add_atom(carbon, [position[0], position[1], 0.0]));
        let lean = if index.is_multiple_of(2) { 1.0 } else { -1.0 } * std::f64::consts::FRAC_PI_6;
        position = [
            position[0] + step * (angle + lean).cos(),
            position[1] + step * (angle + lean).sin(),
        ];
    }
    for pair in ids.windows(2) {
        let _ = molecule.add_bond(pair[0], pair[1], BondOrder::Single);
    }
    ids
}

/// Snap an angle to the nearest multiple of `step_degrees`.
pub fn snap_angle(angle: f64, step_degrees: f64) -> f64 {
    if !step_degrees.is_finite() || step_degrees.abs() <= f64::EPSILON {
        return angle;
    }
    let step = step_degrees.to_radians();
    (angle / step).round() * step
}

fn polygon(count: usize, radius: f64, element: Element) -> Vec<(Element, [f64; 2])> {
    (0..count)
        .map(|index| {
            let angle = std::f64::consts::TAU * index as f64 / count as f64;
            (element, [radius * angle.cos(), radius * angle.sin()])
        })
        .collect()
}

fn chair(radius: f64) -> Vec<(Element, [f64; 2])> {
    let carbon = Element::from_symbol("C").expect("carbon");
    let [x, y] = [radius, radius * 0.55];
    vec![
        (carbon, [-x, 0.0]),
        (carbon, [-x * 0.55, y]),
        (carbon, [x * 0.55, y]),
        (carbon, [x, 0.0]),
        (carbon, [x * 0.55, -y]),
        (carbon, [-x * 0.55, -y]),
    ]
}

fn boat(radius: f64) -> Vec<(Element, [f64; 2])> {
    let carbon = Element::from_symbol("C").expect("carbon");
    let [x, y] = [radius, radius * 0.45];
    vec![
        (carbon, [-x, -y * 0.4]),
        (carbon, [-x * 0.55, y]),
        (carbon, [x * 0.55, y]),
        (carbon, [x, -y * 0.4]),
        (carbon, [x * 0.55, -y * 1.2]),
        (carbon, [-x * 0.55, -y * 1.2]),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_template_builds_a_closed_ring() {
        for template in RingTemplate::ALL {
            let mut molecule = Molecule2D::new();
            let ids = add_ring(&mut molecule, template, [0.0, 0.0], 0.0);
            assert_eq!(ids.len(), template.atoms().len(), "{template:?}");
            assert_eq!(molecule.rings().len(), 1, "{template:?}");
        }
    }

    #[test]
    fn benzene_is_aromatic_and_cyclohexane_is_not() {
        let mut benzene = Molecule2D::new();
        add_ring(&mut benzene, RingTemplate::Benzene, [0.0, 0.0], 0.0);
        assert_eq!(benzene.aromatic_rings().len(), 1);

        let mut cyclohexane = Molecule2D::new();
        add_ring(&mut cyclohexane, RingTemplate::Cyclohexane, [0.0, 0.0], 0.0);
        assert!(cyclohexane.aromatic_rings().is_empty());
    }

    #[test]
    fn a_chain_has_the_requested_length_and_bonds() {
        let mut molecule = Molecule2D::new();
        let ids = add_chain(&mut molecule, [0.0, 0.0], 5, 0.0);
        assert_eq!(ids.len(), 5);
        assert_eq!(molecule.bond_count(), 4);
    }

    #[test]
    fn snapping_rounds_to_thirty_degrees() {
        let snapped = snap_angle(31.0_f64.to_radians(), 30.0).to_degrees().round();
        assert_eq!(snapped, 30.0);
    }
}
