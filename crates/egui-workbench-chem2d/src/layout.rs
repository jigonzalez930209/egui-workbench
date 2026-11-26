//! Layout cleanup for parsed and hand-drawn documents.

use crate::document::Molecule2D;

/// Relax bond lengths and overlap for a readable 2D layout.
pub fn relax(molecule: &mut Molecule2D) {
    let count = molecule.atoms.len();
    if count < 2 {
        return;
    }
    for _ in 0..80 {
        let bonds = molecule.bonds.iter().map(|bond| (bond.a, bond.b)).collect::<Vec<_>>();
        for (left, right) in bonds {
            let (Some(a), Some(b)) = (molecule.atom(left), molecule.atom(right)) else {
                continue;
            };
            let delta = [b.position[0] - a.position[0], b.position[1] - a.position[1]];
            let distance = (delta[0] * delta[0] + delta[1] * delta[1]).sqrt().max(1.0e-6);
            let error = (distance - 1.5) * 0.25;
            let direction = [delta[0] / distance, delta[1] / distance];
            if let Some(atom) = molecule.atom_mut(left) {
                atom.position[0] += direction[0] * error;
                atom.position[1] += direction[1] * error;
            }
            if let Some(atom) = molecule.atom_mut(right) {
                atom.position[0] -= direction[0] * error;
                atom.position[1] -= direction[1] * error;
            }
        }
        for left in 0..count {
            for right in (left + 1)..count {
                let (Some(a), Some(b)) = (molecule.atoms.get(left), molecule.atoms.get(right)) else {
                    continue;
                };
                let delta = [b.position[0] - a.position[0], b.position[1] - a.position[1]];
                let distance = (delta[0] * delta[0] + delta[1] * delta[1]).sqrt().max(1.0e-6);
                if distance < 1.0 {
                    let push = (1.0 - distance) * 0.25;
                    let direction = [delta[0] / distance, delta[1] / distance];
                    let (left, right) = (molecule.atoms[left].id, molecule.atoms[right].id);
                    if let Some(atom) = molecule.atom_mut(left) {
                        atom.position[0] -= direction[0] * push;
                        atom.position[1] -= direction[1] * push;
                    }
                    if let Some(atom) = molecule.atom_mut(right) {
                        atom.position[0] += direction[0] * push;
                        atom.position[1] += direction[1] * push;
                    }
                }
            }
        }
    }
}
