//! Hit testing, selection, implicit hydrogens, and atom labels.

use egui_workbench_chem_core::{AtomId, BondId, ChemSelection, Element, SelectionMode};

use crate::document::Molecule2D;

/// What is under the pointer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hit {
    /// An atom.
    Atom(AtomId),
    /// A bond.
    Bond(BondId),
}

/// The atom or bond nearest to `point`, within `tolerance` Angstrom.
pub fn hit_test(molecule: &Molecule2D, point: [f64; 2], tolerance: f64) -> Option<Hit> {
    let mut best: Option<(f64, Hit)> = None;
    for atom in &molecule.atoms {
        let position = [atom.position[0], atom.position[1]];
        let distance = distance(position, point);
        if distance <= tolerance && best.is_none_or(|(current, _)| distance < current) {
            best = Some((distance, Hit::Atom(atom.id)));
        }
    }
    if best.is_some() {
        return best.map(|(_, hit)| hit);
    }
    for bond in &molecule.bonds {
        let (Some(a), Some(b)) = (molecule.atom(bond.a), molecule.atom(bond.b)) else {
            continue;
        };
        let a = [a.position[0], a.position[1]];
        let b = [b.position[0], b.position[1]];
        let distance = point_segment_distance(point, a, b);
        if distance <= tolerance && best.is_none_or(|(current, _)| distance < current) {
            best = Some((distance, Hit::Bond(bond.id)));
        }
    }
    best.map(|(_, hit)| hit)
}

/// Select atoms inside `min..max` and bonds whose midpoint is inside.
pub fn select_rect(
    molecule: &Molecule2D,
    selection: &mut ChemSelection,
    min: [f64; 2],
    max: [f64; 2],
    mode: SelectionMode,
) {
    let mode = batch_mode(selection, mode);
    let inside = |point: [f64; 2]| {
        point[0] >= min[0].min(max[0])
            && point[0] <= min[0].max(max[0])
            && point[1] >= min[1].min(max[1])
            && point[1] <= min[1].max(max[1])
    };
    for atom in &molecule.atoms {
        if inside([atom.position[0], atom.position[1]]) {
            selection.select_atom(atom.id, mode);
        }
    }
    for bond in &molecule.bonds {
        let (Some(a), Some(b)) = (molecule.atom(bond.a), molecule.atom(bond.b)) else {
            continue;
        };
        let midpoint = [
            (a.position[0] + b.position[0]) * 0.5,
            (a.position[1] + b.position[1]) * 0.5,
        ];
        if inside(midpoint) {
            selection.select_bond(bond.id, mode);
        }
    }
}

/// Select atoms inside a lasso polygon and bonds whose midpoint is inside.
pub fn select_lasso(molecule: &Molecule2D, selection: &mut ChemSelection, polygon: &[[f64; 2]], mode: SelectionMode) {
    if polygon.len() < 3 {
        return;
    }
    let mode = batch_mode(selection, mode);
    for atom in &molecule.atoms {
        if point_in_polygon([atom.position[0], atom.position[1]], polygon) {
            selection.select_atom(atom.id, mode);
        }
    }
    for bond in &molecule.bonds {
        let (Some(a), Some(b)) = (molecule.atom(bond.a), molecule.atom(bond.b)) else {
            continue;
        };
        let midpoint = [
            (a.position[0] + b.position[0]) * 0.5,
            (a.position[1] + b.position[1]) * 0.5,
        ];
        if point_in_polygon(midpoint, polygon) {
            selection.select_bond(bond.id, mode);
        }
    }
}

/// The implicit hydrogens on `id`.
///
/// The target valence is the element's common valence plus the formal charge (an
/// ammonium nitrogen with charge `+1` targets four bonds; a hydroxide oxygen with
/// charge `-1` targets one). Metals and unknown elements have no implicit hydrogens.
pub fn implicit_hydrogens(molecule: &Molecule2D, id: AtomId) -> u8 {
    // One implementation for the label and the formula, so they never disagree.
    u8::try_from(crate::properties::implicit_hydrogens(molecule, id)).unwrap_or(u8::MAX)
}

/// The display label for an atom, including implicit hydrogens and formal charge.
///
/// Carbon atoms with no charge and no custom label are unlabelled (skeletal drawing).
/// A heteroatom whose neighbours lie to its left shows `HO`/`H2N`; one whose neighbours
/// lie to its right shows `OH`/`NH2`.
pub fn atom_label(molecule: &Molecule2D, id: AtomId) -> String {
    let Some(atom) = molecule.atom(id) else {
        return String::new();
    };
    let hydrogens = implicit_hydrogens(molecule, id);
    let charge = charge_suffix(atom.charge);
    if atom.element == Element::UNKNOWN && atom.label.is_none() {
        return format!("?{charge}");
    }
    if atom.element.atomic_number() == 6 && atom.charge == 0 && atom.label.is_none() {
        return String::new();
    }
    let hydrogen = match hydrogens {
        0 => String::new(),
        1 => "H".to_owned(),
        count => format!("H{count}"),
    };
    let neighbours = molecule.neighbors(id);
    let prefix = hydrogens > 0
        && matches!(atom.element.symbol(), "N" | "O" | "S" | "P")
        && neighbour_centroid(molecule, &neighbours).is_some_and(|[x, _]| x > atom.position[0]);
    if prefix {
        format!("{hydrogen}{}{charge}", atom.element.symbol())
    } else {
        format!("{}{hydrogen}{charge}", atom.element.symbol())
    }
}

fn charge_suffix(charge: i8) -> String {
    match charge {
        0 => String::new(),
        1 => "+".to_owned(),
        -1 => "-".to_owned(),
        value if value > 1 => format!("{value}+"),
        value => format!("{}-", value.unsigned_abs()),
    }
}

fn neighbour_centroid(molecule: &Molecule2D, ids: &[AtomId]) -> Option<[f64; 2]> {
    if ids.is_empty() {
        return None;
    }
    let mut sum = [0.0; 2];
    let mut count = 0.0;
    for id in ids {
        if let Some(atom) = molecule.atom(*id) {
            sum[0] += atom.position[0];
            sum[1] += atom.position[1];
            count += 1.0;
        }
    }
    (count > 0.0).then(|| [sum[0] / count, sum[1] / count])
}

fn batch_mode(selection: &mut ChemSelection, mode: SelectionMode) -> SelectionMode {
    if mode == SelectionMode::Replace {
        selection.clear();
        SelectionMode::Add
    } else {
        mode
    }
}

fn distance(a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

fn point_segment_distance(point: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let ab = [b[0] - a[0], b[1] - a[1]];
    let length_squared = ab[0] * ab[0] + ab[1] * ab[1];
    if length_squared <= f64::EPSILON {
        return distance(point, a);
    }
    let t = (((point[0] - a[0]) * ab[0] + (point[1] - a[1]) * ab[1]) / length_squared).clamp(0.0, 1.0);
    distance(point, [a[0] + t * ab[0], a[1] + t * ab[1]])
}

fn point_in_polygon(point: [f64; 2], polygon: &[[f64; 2]]) -> bool {
    let mut inside = false;
    let mut j = polygon.len() - 1;
    for i in 0..polygon.len() {
        let a = polygon[i];
        let b = polygon[j];
        if (a[1] > point[1]) != (b[1] > point[1]) && point[0] < (b[0] - a[0]) * (point[1] - a[1]) / (b[1] - a[1]) + a[0]
        {
            inside = !inside;
        }
        j = i;
    }
    inside
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::ChemDocument;
    use egui_workbench_chem_core::{BondOrder, DocumentId, Element};

    fn water() -> (ChemDocument, AtomId, AtomId, AtomId) {
        let mut document = ChemDocument::new(DocumentId::new(1), "water");
        let oxygen = document
            .molecule
            .add_atom(Element::from_symbol("O").expect("O"), [0.0, 0.0, 0.0]);
        let h1 = document
            .molecule
            .add_atom(Element::from_symbol("H").expect("H"), [0.96, 0.0, 0.0]);
        let h2 = document
            .molecule
            .add_atom(Element::from_symbol("H").expect("H"), [-0.24, 0.93, 0.0]);
        document.molecule.add_bond(oxygen, h1, BondOrder::Single).expect("bond");
        document.molecule.add_bond(oxygen, h2, BondOrder::Single).expect("bond");
        (document, oxygen, h1, h2)
    }

    #[test]
    fn hit_testing_prefers_atoms_then_bonds() {
        let (document, oxygen, hydrogen, _) = water();
        assert_eq!(
            hit_test(&document.molecule, [0.02, -0.01], 0.2),
            Some(Hit::Atom(oxygen))
        );
        assert_eq!(
            hit_test(&document.molecule, [0.48, 0.0], 0.1),
            Some(Hit::Bond(document.molecule.bonds[0].id))
        );
        assert_eq!(
            hit_test(&document.molecule, [0.96, 0.0], 0.2),
            Some(Hit::Atom(hydrogen))
        );
    }

    #[test]
    fn water_has_no_implicit_hydrogens_and_an_alcohol_has_one() {
        let (document, oxygen, _, _) = water();
        assert_eq!(implicit_hydrogens(&document.molecule, oxygen), 0);

        let mut alcohol = ChemDocument::new(DocumentId::new(2), "methanol");
        let carbon = alcohol
            .molecule
            .add_atom(Element::from_symbol("C").expect("C"), [-1.2, 0.0, 0.0]);
        let oxygen = alcohol
            .molecule
            .add_atom(Element::from_symbol("O").expect("O"), [0.0, 0.0, 0.0]);
        alcohol
            .molecule
            .add_bond(carbon, oxygen, BondOrder::Single)
            .expect("bond");
        assert_eq!(implicit_hydrogens(&alcohol.molecule, oxygen), 1);
        assert_eq!(atom_label(&alcohol.molecule, oxygen), "OH");
    }

    #[test]
    fn labels_flip_when_the_neighbours_are_on_the_right() {
        let mut document = ChemDocument::new(DocumentId::new(3), "methanol");
        let oxygen = document
            .molecule
            .add_atom(Element::from_symbol("O").expect("O"), [0.0, 0.0, 0.0]);
        let carbon = document
            .molecule
            .add_atom(Element::from_symbol("C").expect("C"), [1.2, 0.0, 0.0]);
        document
            .molecule
            .add_bond(oxygen, carbon, BondOrder::Single)
            .expect("bond");
        assert_eq!(atom_label(&document.molecule, oxygen), "HO");
    }

    #[test]
    fn charged_nitrogen_uses_the_adjusted_valence() {
        let mut document = ChemDocument::new(DocumentId::new(4), "ammonium");
        let nitrogen = document
            .molecule
            .add_atom(Element::from_symbol("N").expect("N"), [0.0, 0.0, 0.0]);
        if let Some(atom) = document.molecule.atom_mut(nitrogen) {
            atom.charge = 1;
        }
        for index in 0..4 {
            let angle = std::f64::consts::TAU * f64::from(index) / 4.0;
            let hydrogen = document
                .molecule
                .add_atom(Element::from_symbol("H").expect("H"), [angle.cos(), angle.sin(), 0.0]);
            document
                .molecule
                .add_bond(nitrogen, hydrogen, BondOrder::Single)
                .expect("bond");
        }
        assert_eq!(implicit_hydrogens(&document.molecule, nitrogen), 0);
        assert_eq!(atom_label(&document.molecule, nitrogen), "N+");
    }

    #[test]
    fn rectangle_and_lasso_selection_pick_atoms() {
        let (document, oxygen, hydrogen, _) = water();
        let mut selection = ChemSelection::new();
        select_rect(
            &document.molecule,
            &mut selection,
            [-0.1, -0.1],
            [0.1, 0.1],
            SelectionMode::Replace,
        );
        assert!(selection.atoms().contains(&oxygen));
        assert!(!selection.atoms().contains(&hydrogen));

        let mut lasso = ChemSelection::new();
        select_lasso(
            &document.molecule,
            &mut lasso,
            &[[-0.2, -0.2], [1.1, -0.2], [1.1, 0.2], [-0.2, 0.2]],
            SelectionMode::Replace,
        );
        assert!(lasso.atoms().contains(&oxygen));
        assert!(lasso.atoms().contains(&hydrogen));
    }
}
