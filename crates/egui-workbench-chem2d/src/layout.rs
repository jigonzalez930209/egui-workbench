//! Layout cleanup for parsed and hand-drawn documents.

use crate::document::Molecule2D;

/// Relax bond lengths and overlap for a readable 2D layout.
pub fn relax(molecule: &mut Molecule2D) {
    let count = molecule.atoms.len();
    if count < 2 {
        return;
    }
    // Work on plain arrays. The previous version looked an atom up by id inside the
    // pairwise loop, which made 80 iterations cost O(n^3) and froze the editor on a few
    // hundred atoms.
    let index: std::collections::HashMap<_, _> = molecule
        .atoms
        .iter()
        .enumerate()
        .map(|(position, atom)| (atom.id, position))
        .collect();
    let bonds: Vec<(usize, usize)> = molecule
        .bonds
        .iter()
        .filter_map(|bond| Some((*index.get(&bond.a)?, *index.get(&bond.b)?)))
        .collect();
    let mut points: Vec<[f64; 2]> = molecule
        .atoms
        .iter()
        .map(|atom| [atom.position[0], atom.position[1]])
        .collect();
    // Pairwise repulsion is O(n^2) per pass; past this size only the bond springs run.
    let repel = count <= REPULSION_LIMIT;
    for _ in 0..80 {
        for &(left, right) in &bonds {
            let (direction, distance) = separation(&points, left, right);
            let error = (distance - 1.5) * 0.25;
            points[left][0] += direction[0] * error;
            points[left][1] += direction[1] * error;
            points[right][0] -= direction[0] * error;
            points[right][1] -= direction[1] * error;
        }
        if !repel {
            continue;
        }
        for left in 0..count {
            for right in (left + 1)..count {
                let (direction, distance) = separation(&points, left, right);
                if distance < 1.0 {
                    let push = (1.0 - distance) * 0.25;
                    points[left][0] -= direction[0] * push;
                    points[left][1] -= direction[1] * push;
                    points[right][0] += direction[0] * push;
                    points[right][1] += direction[1] * push;
                }
            }
        }
    }
    for (atom, point) in molecule.atoms.iter_mut().zip(&points) {
        if point[0].is_finite() && point[1].is_finite() {
            atom.position[0] = point[0];
            atom.position[1] = point[1];
        }
    }
}

/// Most atoms for which the pairwise repulsion pass runs.
const REPULSION_LIMIT: usize = 3_000;

/// The unit direction from atom `left` to atom `right` and their distance. Atoms on top of
/// each other get a direction derived from their indices, so they still separate.
fn separation(points: &[[f64; 2]], left: usize, right: usize) -> ([f64; 2], f64) {
    let delta = [points[right][0] - points[left][0], points[right][1] - points[left][1]];
    let distance = delta[0].hypot(delta[1]);
    if distance < 1.0e-6 {
        let angle = (left * 31 + right * 17) as f64;
        return ([angle.cos(), angle.sin()], 1.0e-6);
    }
    ([delta[0] / distance, delta[1] / distance], distance)
}
