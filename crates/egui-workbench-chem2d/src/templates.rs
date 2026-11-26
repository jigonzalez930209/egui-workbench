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

/// Attach a ring to `anchor` with a new single bond.
///
/// The clicked atom is kept; the ring is placed outward, 120 degrees from the existing
/// bonds when possible, with its first atom one bond length away. This is the ChemDraw
/// and MolView ring-tool behaviour when the pointer is over an atom.
pub fn attach_ring(molecule: &mut Molecule2D, template: RingTemplate, anchor: AtomId) -> Vec<AtomId> {
    let Some(entry) = molecule.atom(anchor) else {
        return Vec::new();
    };
    let origin = [entry.position[0], entry.position[1]];
    let radius = template
        .atoms()
        .iter()
        .map(|(_, [x, y])| x.hypot(*y))
        .fold(0.0, f64::max);
    let angle = sprout_angle(molecule, anchor, 1.5 + radius).unwrap_or(0.0);
    let Some((_, [x, y])) = template.atoms().into_iter().next() else {
        return Vec::new();
    };
    let local_angle = y.atan2(x);
    let rotation = angle + std::f64::consts::PI - local_angle;
    let center = [
        origin[0] + (1.5 + radius) * angle.cos(),
        origin[1] + (1.5 + radius) * angle.sin(),
    ];
    let ids = add_ring(molecule, template, center, rotation);
    if let Some(first) = ids.first() {
        let _ = molecule.add_bond(anchor, *first, BondOrder::Single);
    }
    ids
}

/// The direction in which a new bond or fragment sprouts from `atom`.
///
/// `reach` is how far the new group extends from the atom (a bond is 1.5 Å; a ring is a
/// bond plus its radius). Candidates keep the 120° bond angle and are then scored by the
/// clear space at that reach, so a new ring or fragment never grows back across the
/// molecule that is already drawn.
pub(crate) fn sprout_angle(molecule: &Molecule2D, atom: AtomId, reach: f64) -> Option<f64> {
    let entry = molecule.atom(atom)?;
    let origin = [entry.position[0], entry.position[1]];
    let directions = molecule
        .neighbors(atom)
        .into_iter()
        .filter_map(|neighbor| molecule.atom(neighbor))
        .map(|neighbor| (neighbor.position[1] - origin[1]).atan2(neighbor.position[0] - origin[0]))
        .collect::<Vec<_>>();
    if directions.is_empty() {
        let others = molecule
            .atoms
            .iter()
            .filter(|other| other.id != atom)
            .map(|other| [other.position[0], other.position[1]])
            .collect::<Vec<_>>();
        if others.is_empty() {
            return Some(0.0);
        }
        let mut sum = [0.0; 2];
        for other in &others {
            sum[0] += other[0];
            sum[1] += other[1];
        }
        let center = [sum[0] / others.len() as f64, sum[1] / others.len() as f64];
        let away = [origin[0] - center[0], origin[1] - center[1]];
        return Some(if away[0].hypot(away[1]) <= f64::EPSILON {
            0.0
        } else {
            away[1].atan2(away[0])
        });
    }

    let mut candidates = Vec::<(f64, f64)>::new();
    for direction in &directions {
        for offset in [-2.0 * std::f64::consts::FRAC_PI_3, 2.0 * std::f64::consts::FRAC_PI_3] {
            let candidate = normalize_angle(direction + offset);
            let separation = directions
                .iter()
                .map(|existing| angle_distance(candidate, *existing))
                .fold(f64::INFINITY, f64::min);
            candidates.push((separation, candidate));
        }
    }
    let best_separation = candidates
        .iter()
        .map(|(separation, _)| *separation)
        .fold(f64::NEG_INFINITY, f64::max);
    let mut best: Option<(f64, f64)> = None;
    for (separation, candidate) in candidates {
        if separation + 1.0e-9 < best_separation {
            continue;
        }
        let probe = [origin[0] + reach * candidate.cos(), origin[1] + reach * candidate.sin()];
        let clearance = molecule
            .atoms
            .iter()
            .filter(|other| other.id != atom)
            .map(|other| (other.position[0] - probe[0]).hypot(other.position[1] - probe[1]))
            .fold(f64::INFINITY, f64::min);
        let better = best.is_none_or(|(best_clearance, best_candidate)| {
            clearance > best_clearance + 1.0e-9
                || ((clearance - best_clearance).abs() <= 1.0e-9 && candidate < best_candidate)
        });
        if better {
            best = Some((clearance, candidate));
        }
    }
    best.map(|(_, candidate)| candidate)
}

fn normalize_angle(angle: f64) -> f64 {
    angle.rem_euclid(std::f64::consts::TAU)
}

fn angle_distance(a: f64, b: f64) -> f64 {
    let distance = (a - b).rem_euclid(std::f64::consts::TAU);
    distance.min(std::f64::consts::TAU - distance)
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
    fn a_ring_attaches_to_an_atom_with_one_bond() {
        let mut molecule = Molecule2D::new();
        let anchor = molecule.add_atom(Element::from_symbol("C").expect("C"), [0.0, 0.0, 0.0]);
        let ids = attach_ring(&mut molecule, RingTemplate::Benzene, anchor);
        assert_eq!(ids.len(), 6);
        assert_eq!(molecule.atom_count(), 7);
        assert_eq!(molecule.bond_count(), 7);
        assert_eq!(molecule.rings().len(), 1);
        let first = molecule.atom(ids[0]).expect("first");
        let length = (first.position[0] - 0.0).hypot(first.position[1] - 0.0);
        assert!((length - 1.5).abs() < 1.0e-9, "{length}");
        assert!(molecule.bond_between(anchor, ids[0]).is_some());
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
