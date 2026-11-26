//! Ring perception: a smallest-first cycle basis and aromaticity.

use std::collections::{BTreeMap, HashSet, VecDeque};

use egui_workbench_chem_core::{AtomId, BondId, BondOrder, Molecule};

/// One ring in the cycle basis.
#[derive(Clone, Debug, PartialEq)]
pub struct Ring {
    /// Atoms in ring order.
    pub atoms: Vec<AtomId>,
    /// Bonds in ring order.
    pub bonds: Vec<BondId>,
    /// Whether the ring is aromatic.
    pub aromatic: bool,
}

impl Ring {
    /// The average position of the ring atoms.
    pub fn centroid(&self, molecule: &Molecule) -> Option<[f64; 2]> {
        let mut sum = [0.0; 2];
        let mut count = 0.0;
        for id in &self.atoms {
            let atom = molecule.atom(*id)?;
            sum[0] += atom.position[0];
            sum[1] += atom.position[1];
            count += 1.0;
        }
        (count > 0.0).then(|| [sum[0] / count, sum[1] / count])
    }
}

/// A smallest-first cycle basis (the SSSR when the graph is well behaved).
pub fn cycle_basis(molecule: &Molecule) -> Vec<Ring> {
    let index = molecule
        .atoms
        .iter()
        .enumerate()
        .map(|(index, atom)| (atom.id, index))
        .collect::<BTreeMap<_, _>>();
    let mut adjacency = vec![Vec::<(usize, BondId)>::new(); molecule.atoms.len()];
    for bond in &molecule.bonds {
        let (Some(a), Some(b)) = (index.get(&bond.a), index.get(&bond.b)) else {
            continue;
        };
        adjacency[*a].push((*b, bond.id));
        adjacency[*b].push((*a, bond.id));
    }
    let mut seen = HashSet::new();
    let mut candidates = Vec::<Ring>::new();
    for bond in &molecule.bonds {
        let (Some(a), Some(b)) = (index.get(&bond.a), index.get(&bond.b)) else {
            continue;
        };
        if let Some((atoms, bonds)) = shortest_cycle(&adjacency, *a, *b, bond.id, &molecule.atoms) {
            let mut key = bonds.clone();
            key.sort_unstable_by_key(|id| id.slot());
            if seen.insert(key) {
                candidates.push(Ring {
                    atoms,
                    bonds,
                    aromatic: false,
                });
            }
        }
    }
    candidates.sort_by_key(|ring| (ring.atoms.len(), ring.bonds.len()));
    let expected = expected_rank(molecule);
    let mut selected = Vec::new();
    let mut basis = Vec::<Vec<u64>>::new();
    let max_bond = molecule
        .bonds
        .iter()
        .map(|bond| bond.id.slot() as usize)
        .max()
        .unwrap_or(0)
        + 1;
    for mut ring in candidates {
        let mut row = vec![0u64; max_bond.div_ceil(64)];
        for bond in &ring.bonds {
            let slot = bond.slot() as usize;
            row[slot / 64] |= 1 << (slot % 64);
        }
        for pivot in &basis {
            let Some((word, bit)) = pivot
                .iter()
                .enumerate()
                .find_map(|(word, value)| (*value != 0).then(|| (word, value.trailing_zeros())))
            else {
                continue;
            };
            if row[word] & (1 << bit) != 0 {
                for (index, value) in row.iter_mut().enumerate() {
                    *value ^= pivot[index];
                }
            }
        }
        if row.iter().any(|word| *word != 0) {
            basis.push(row);
            ring.aromatic = is_aromatic(molecule, &ring);
            selected.push(ring);
            if selected.len() == expected {
                break;
            }
        }
    }
    selected
}

/// The aromatic rings of the molecule.
pub fn aromatic_rings(molecule: &Molecule) -> Vec<Ring> {
    cycle_basis(molecule).into_iter().filter(|ring| ring.aromatic).collect()
}

/// The ring that contains `bond`, if any.
pub fn ring_for_bond(molecule: &Molecule, bond: BondId) -> Option<Ring> {
    cycle_basis(molecule)
        .into_iter()
        .find(|ring| ring.bonds.contains(&bond))
}

fn shortest_cycle(
    adjacency: &[Vec<(usize, BondId)>],
    start: usize,
    end: usize,
    avoided: BondId,
    atoms: &[egui_workbench_chem_core::Atom],
) -> Option<(Vec<AtomId>, Vec<BondId>)> {
    let mut previous = vec![None::<(usize, BondId)>; adjacency.len()];
    let mut queue = VecDeque::from([start]);
    let mut visited = vec![false; adjacency.len()];
    visited[start] = true;
    while let Some(node) = queue.pop_front() {
        if node == end {
            break;
        }
        for (next, bond) in &adjacency[node] {
            if *bond == avoided || visited[*next] {
                continue;
            }
            visited[*next] = true;
            previous[*next] = Some((node, *bond));
            queue.push_back(*next);
        }
    }
    if !visited[end] {
        return None;
    }
    let mut atom_path = vec![end];
    let mut bond_path = Vec::new();
    let mut node = end;
    while node != start {
        let (previous_node, bond) = previous[node]?;
        bond_path.push(bond);
        atom_path.push(previous_node);
        node = previous_node;
    }
    atom_path.reverse();
    bond_path.reverse();
    bond_path.push(avoided);
    let atoms = atom_path
        .into_iter()
        .filter_map(|index| atoms.get(index).map(|atom| atom.id))
        .collect();
    Some((atoms, bond_path))
}

fn expected_rank(molecule: &Molecule) -> usize {
    let components = component_count(molecule);
    molecule.bonds.len().saturating_sub(molecule.atoms.len()) + components
}

fn component_count(molecule: &Molecule) -> usize {
    let mut parent = (0..molecule.atoms.len()).collect::<Vec<_>>();
    fn find(parent: &mut [usize], index: usize) -> usize {
        if parent[index] != index {
            parent[index] = find(parent, parent[index]);
        }
        parent[index]
    }
    let index = molecule
        .atoms
        .iter()
        .enumerate()
        .map(|(index, atom)| (atom.id, index))
        .collect::<BTreeMap<_, _>>();
    for bond in &molecule.bonds {
        if let (Some(a), Some(b)) = (index.get(&bond.a), index.get(&bond.b)) {
            let (a, b) = (find(&mut parent, *a), find(&mut parent, *b));
            parent[a] = b;
        }
    }
    (0..molecule.atoms.len())
        .map(|index| find(&mut parent, index))
        .collect::<HashSet<_>>()
        .len()
}

/// Furan, pyrrole, and thiophene in Kekulé form: two non-adjacent double bonds, and the
/// ring atom they leave out is N, O, S, or Se, whose lone pair completes the six pi
/// electrons. Cyclopentadiene leaves out a carbon and is not aromatic.
fn is_kekule_heteroaromatic(molecule: &Molecule, ring: &Ring, orders: &[BondOrder]) -> bool {
    let doubles = orders.iter().filter(|order| **order == BondOrder::Double).count();
    if doubles != 2
        || orders
            .iter()
            .any(|order| !matches!(order, BondOrder::Double | BondOrder::Single))
    {
        return false;
    }
    let touched = |atom: AtomId| {
        ring.bonds.iter().any(|id| {
            molecule
                .bond(*id)
                .is_some_and(|bond| bond.order == BondOrder::Double && (bond.a == atom || bond.b == atom))
        })
    };
    let mut left_out = ring.atoms.iter().filter(|atom| !touched(**atom));
    match (left_out.next(), left_out.next()) {
        (Some(atom), None) => molecule
            .atom(*atom)
            .is_some_and(|atom| matches!(atom.element.atomic_number(), 7 | 8 | 16 | 34)),
        _ => false,
    }
}

fn is_aromatic(molecule: &Molecule, ring: &Ring) -> bool {
    let orders = ring
        .bonds
        .iter()
        .filter_map(|id| molecule.bond(*id).map(|bond| bond.order))
        .collect::<Vec<_>>();
    if orders.is_empty() {
        return false;
    }
    if orders.iter().all(|order| *order == BondOrder::Aromatic) {
        return true;
    }
    if orders.len() == 5 && is_kekule_heteroaromatic(molecule, ring, &orders) {
        return true;
    }
    // A Kekulé ring (alternating single and double bonds) is aromatic only when its pi
    // electron count obeys Hückel's rule: 4n + 2, so 6, 10, 14... Four- and eight-membered
    // alternating rings (cyclobutadiene, cyclooctatetraene) are not aromatic.
    if orders.len() % 4 != 2 {
        return false;
    }
    orders.iter().enumerate().all(|(index, order)| {
        if index.is_multiple_of(2) {
            *order == BondOrder::Double
        } else {
            *order == BondOrder::Single
        }
    }) || orders.iter().enumerate().all(|(index, order)| {
        if index.is_multiple_of(2) {
            *order == BondOrder::Single
        } else {
            *order == BondOrder::Double
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::Molecule2D;
    use egui_workbench_chem_core::Element;

    fn carbon() -> Element {
        Element::from_symbol("C").expect("carbon")
    }

    fn ring(size: usize) -> (Molecule2D, Vec<AtomId>) {
        let mut molecule = Molecule2D::new();
        let ids = (0..size)
            .map(|index| {
                let angle = std::f64::consts::TAU * index as f64 / size as f64;
                molecule.add_atom(carbon(), [angle.cos(), angle.sin(), 0.0])
            })
            .collect::<Vec<_>>();
        for index in 0..size {
            molecule
                .add_bond(ids[index], ids[(index + 1) % size], BondOrder::Single)
                .expect("bond");
        }
        (molecule, ids)
    }

    #[test]
    fn a_benzene_ring_is_found_once() {
        let (molecule, _) = ring(6);
        let rings = cycle_basis(&molecule);
        assert_eq!(rings.len(), 1);
        assert_eq!(rings[0].atoms.len(), 6);
        assert_eq!(rings[0].bonds.len(), 6);
    }

    #[test]
    fn a_chain_has_no_rings() {
        let mut molecule = Molecule2D::new();
        let a = molecule.add_atom(carbon(), [0.0, 0.0, 0.0]);
        let b = molecule.add_atom(carbon(), [1.0, 0.0, 0.0]);
        molecule.add_bond(a, b, BondOrder::Single).expect("bond");
        assert!(cycle_basis(&molecule).is_empty());
    }

    #[test]
    fn alternating_bonds_mark_a_ring_aromatic() {
        let (mut molecule, ids) = ring(6);
        for index in 0..6 {
            let Some(bond) = molecule
                .bond_between(ids[index], ids[(index + 1) % 6])
                .map(|bond| bond.id)
            else {
                continue;
            };
            if let Some(entry) = molecule.bonds.iter_mut().find(|entry| entry.id == bond) {
                entry.order = if index.is_multiple_of(2) {
                    BondOrder::Double
                } else {
                    BondOrder::Single
                };
            }
        }
        let rings = aromatic_rings(&molecule);
        assert_eq!(rings.len(), 1);
        assert!(rings[0].aromatic);
    }

    #[test]
    fn only_huckel_sized_kekule_rings_are_aromatic() {
        for (size, expected) in [(4, false), (6, true), (8, false), (10, true)] {
            let (mut molecule, ids) = ring(size);
            for index in 0..size {
                let bond = molecule
                    .bond_between(ids[index], ids[(index + 1) % size])
                    .map(|bond| bond.id)
                    .expect("ring bond");
                if let Some(entry) = molecule.bonds.iter_mut().find(|entry| entry.id == bond) {
                    entry.order = if index % 2 == 0 {
                        BondOrder::Double
                    } else {
                        BondOrder::Single
                    };
                }
            }
            assert_eq!(
                molecule.aromatic_rings().len() == 1,
                expected,
                "a Kekulé ring of {size} atoms"
            );
        }
    }
}
