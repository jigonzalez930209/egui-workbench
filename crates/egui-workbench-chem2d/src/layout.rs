//! Canonical two-dimensional layout and geometry optimization.

use std::collections::{HashMap, VecDeque};

use crate::document::Molecule2D;

/// Standard bond length in the editor, in Angstrom.
const BOND_LENGTH: f64 = 1.5;
/// Space between disconnected components, in Angstrom.
const COMPONENT_GAP: f64 = 2.0;
/// The angle used for acyclic chains and trigonal centres.
const CHAIN_ANGLE: f64 = 2.0 * std::f64::consts::FRAC_PI_3;

/// Build a clean, deterministic 2D layout for the document.
///
/// The result depends only on the molecular graph, not on the current coordinates. Rings
/// are placed as regular polygons, fused rings share their edge on opposite sides, and
/// acyclic chains and substituents grow at 120°. Disconnected components are packed side
/// by side. The current drawing's centre and first-bond orientation are preserved, so a
/// second call is a no-op and the structure never flips or rotates unexpectedly.
pub fn optimize_layout(molecule: &mut Molecule2D) {
    let count = molecule.atoms.len();
    if count < 2 {
        return;
    }
    let current = molecule
        .atoms
        .iter()
        .map(|atom| {
            if atom.position[0].is_finite() && atom.position[1].is_finite() {
                [atom.position[0], atom.position[1]]
            } else {
                [0.0, 0.0]
            }
        })
        .collect::<Vec<_>>();
    let Some(mut points) = canonical_points(molecule) else {
        return;
    };
    let before_angle = first_bond_angle(molecule, &current);
    let after_angle = first_bond_angle(molecule, &points);
    if let (Some(before), Some(after)) = (before_angle, after_angle) {
        let rotation = wrap_signed(before - after);
        let (sine, cosine) = rotation.sin_cos();
        for point in &mut points {
            let x = point[0];
            let y = point[1];
            point[0] = x * cosine - y * sine;
            point[1] = x * sine + y * cosine;
        }
    }
    let center_before = centroid(&current);
    let center_after = centroid(&points);
    let shift = [center_before[0] - center_after[0], center_before[1] - center_after[1]];
    for (atom, point) in molecule.atoms.iter_mut().zip(&points) {
        if point[0].is_finite() && point[1].is_finite() {
            atom.position[0] = point[0] + shift[0];
            atom.position[1] = point[1] + shift[1];
        }
    }
}

/// The previous name for [`optimize_layout`].
pub fn relax(molecule: &mut Molecule2D) {
    optimize_layout(molecule);
}

fn canonical_points(molecule: &Molecule2D) -> Option<Vec<[f64; 2]>> {
    let count = molecule.atoms.len();
    if count == 0 {
        return None;
    }
    let index = molecule
        .atoms
        .iter()
        .enumerate()
        .map(|(index, atom)| (atom.id, index))
        .collect::<HashMap<_, _>>();
    let mut adjacency = vec![Vec::<usize>::new(); count];
    for bond in &molecule.bonds {
        let (Some(a), Some(b)) = (index.get(&bond.a), index.get(&bond.b)) else {
            continue;
        };
        adjacency[*a].push(*b);
        adjacency[*b].push(*a);
    }

    let rings = molecule
        .rings()
        .into_iter()
        .filter_map(|ring| {
            let atoms = ring
                .atoms
                .iter()
                .filter_map(|atom| index.get(atom).copied())
                .collect::<Vec<_>>();
            (atoms.len() >= 3).then_some(atoms)
        })
        .collect::<Vec<_>>();
    let (system_rings, atom_system) = ring_systems(&rings, count);
    let mut points = vec![None::<[f64; 2]>; count];
    let mut system_placed = vec![false; system_rings.len()];
    let mut queue = VecDeque::<usize>::new();

    while let Some(seed) = (0..count)
        .find(|atom| points[*atom].is_none() && atom_system[*atom].is_some())
        .or_else(|| points.iter().position(Option::is_none))
    {
        // Each disconnected component starts at the origin; the packing pass below moves
        // it to its final position.
        let component = collect_component(seed, &adjacency, &points);
        if component.len() > 1 && component.iter().all(|atom| atom_system[*atom].is_none()) {
            layout_acyclic_component(&component, &adjacency, &mut points);
            continue;
        }
        let mut placed = Vec::new();
        if let Some(system) = atom_system[seed] {
            if system_placed[system] {
                points[seed] = Some([0.0, 0.0]);
                placed.push(seed);
            } else {
                let new = place_ring_system(
                    system,
                    &system_rings,
                    &rings,
                    &mut points,
                    Some(seed),
                    Some([0.0, 0.0]),
                    0.0,
                );
                system_placed[system] = true;
                placed.extend(new);
                if points[seed].is_none() {
                    points[seed] = Some([0.0, 0.0]);
                    placed.push(seed);
                }
            }
        } else {
            points[seed] = Some([0.0, 0.0]);
            placed.push(seed);
        }
        queue.clear();
        queue.extend(placed);
        while let Some(atom) = queue.pop_front() {
            let Some(origin) = points[atom] else {
                continue;
            };
            for neighbor in adjacency[atom].clone() {
                if points[neighbor].is_some() {
                    continue;
                }
                if let Some(system) = atom_system[neighbor] {
                    if system_placed[system] {
                        if points[neighbor].is_none() {
                            points[neighbor] = Some(origin);
                        }
                        continue;
                    }
                    let direction =
                        ring_entry_direction(&points, &adjacency, atom, neighbor, system, &system_rings, &rings);
                    let entry = [
                        origin[0] + BOND_LENGTH * direction.cos(),
                        origin[1] + BOND_LENGTH * direction.sin(),
                    ];
                    let new = place_ring_system(
                        system,
                        &system_rings,
                        &rings,
                        &mut points,
                        Some(neighbor),
                        Some(entry),
                        direction,
                    );
                    system_placed[system] = true;
                    queue.extend(new);
                } else {
                    let direction = next_direction(&points, &adjacency, atom, BOND_LENGTH);
                    points[neighbor] = Some([
                        origin[0] + BOND_LENGTH * direction.cos(),
                        origin[1] + BOND_LENGTH * direction.sin(),
                    ]);
                    queue.push_back(neighbor);
                }
            }
        }
    }

    // A malformed or exotic ring system can leave an atom without a position. Put it
    // next to any placed neighbour rather than returning a half-laid-out molecule.
    while let Some(atom) = points.iter().position(Option::is_none) {
        let neighbor = adjacency[atom].iter().copied().find(|other| points[*other].is_some());
        let direction = neighbor.map_or(0.0, |parent| next_direction(&points, &adjacency, parent, BOND_LENGTH));
        let origin = neighbor.and_then(|parent| points[parent]).unwrap_or([0.0, 0.0]);
        points[atom] = Some([
            origin[0] + BOND_LENGTH * direction.cos(),
            origin[1] + BOND_LENGTH * direction.sin(),
        ]);
    }
    let mut points = points
        .into_iter()
        .map(|point| point.unwrap_or([0.0, 0.0]))
        .collect::<Vec<_>>();
    pack_components(&mut points, &adjacency);
    relocate_leaves(molecule, &adjacency, &mut points);
    let center = centroid(&points);
    for point in &mut points {
        point[0] -= center[0];
        point[1] -= center[1];
    }
    Some(points)
}

fn ring_systems(rings: &[Vec<usize>], count: usize) -> (Vec<Vec<usize>>, Vec<Option<usize>>) {
    let mut parent = (0..rings.len()).collect::<Vec<_>>();
    fn find(parent: &mut [usize], index: usize) -> usize {
        if parent[index] != index {
            parent[index] = find(parent, parent[index]);
        }
        parent[index]
    }
    for left in 0..rings.len() {
        for right in (left + 1)..rings.len() {
            if rings[left].iter().any(|atom| rings[right].contains(atom)) {
                let a = find(&mut parent, left);
                let b = find(&mut parent, right);
                if a != b {
                    parent[a] = b;
                }
            }
        }
    }
    let mut groups = HashMap::<usize, Vec<usize>>::new();
    for ring in 0..rings.len() {
        let root = find(&mut parent, ring);
        groups.entry(root).or_default().push(ring);
    }
    let mut systems = groups.into_values().collect::<Vec<_>>();
    systems.sort_by_key(|rings| rings.iter().copied().min().unwrap_or(usize::MAX));
    let mut atom_system = vec![None; count];
    for (system, rings_in_system) in systems.iter().enumerate() {
        for ring in rings_in_system {
            for atom in &rings[*ring] {
                if *atom < atom_system.len() {
                    atom_system[*atom] = Some(system);
                }
            }
        }
    }
    (systems, atom_system)
}

fn place_ring_system(
    system: usize,
    system_rings: &[Vec<usize>],
    rings: &[Vec<usize>],
    points: &mut [Option<[f64; 2]>],
    entry_atom: Option<usize>,
    entry_position: Option<[f64; 2]>,
    direction: f64,
) -> Vec<usize> {
    let Some(system_ring_indices) = system_rings.get(system) else {
        return Vec::new();
    };
    let entry_ring = entry_atom
        .and_then(|atom| {
            system_ring_indices
                .iter()
                .copied()
                .find(|ring| rings[*ring].contains(&atom))
        })
        .unwrap_or(system_ring_indices[0]);
    let ring = &rings[entry_ring];
    let count = ring.len();
    let mut placed = Vec::new();
    if let Some(entry) = entry_atom {
        let Some(position) = entry_position else {
            return placed;
        };
        let position_index = ring.iter().position(|atom| *atom == entry).unwrap_or(0);
        let radius = polygon_radius(count);
        let center = [
            position[0] + radius * direction.cos(),
            position[1] + radius * direction.sin(),
        ];
        let start = direction + std::f64::consts::PI;
        for step in 0..count {
            let atom = ring[(position_index + step) % count];
            if points[atom].is_none() {
                let angle = start + std::f64::consts::TAU * step as f64 / count as f64;
                points[atom] = Some([center[0] + radius * angle.cos(), center[1] + radius * angle.sin()]);
                placed.push(atom);
            }
        }
        if points[entry].is_none() {
            points[entry] = Some(position);
            placed.push(entry);
        }
    } else {
        // A seeded ring system starts with a regular polygon around the origin.
        let center = [0.0, 0.0];
        let radius = polygon_radius(count);
        let start = std::f64::consts::FRAC_PI_2;
        for (step, atom) in ring.iter().enumerate() {
            let atom = *atom;
            let angle = start + std::f64::consts::TAU * step as f64 / count as f64;
            points[atom] = Some([center[0] + radius * angle.cos(), center[1] + radius * angle.sin()]);
            placed.push(atom);
        }
    }

    let mut pending = system_ring_indices
        .iter()
        .copied()
        .filter(|ring| *ring != entry_ring)
        .collect::<Vec<_>>();
    while !pending.is_empty() {
        let mut progress = false;
        let mut next = Vec::new();
        for ring_index in pending {
            let ring = &rings[ring_index];
            let placed_count = ring.iter().filter(|atom| points[**atom].is_some()).count();
            if placed_count >= 2 {
                let new = place_fused_ring(ring, points);
                if new.is_empty() {
                    next.push(ring_index);
                } else {
                    placed.extend(new);
                    progress = true;
                }
            } else if placed_count == 1 {
                let shared = *ring.iter().find(|atom| points[**atom].is_some()).expect("one placed");
                let new = place_spiro_ring(ring, shared, points);
                if new.is_empty() {
                    next.push(ring_index);
                } else {
                    placed.extend(new);
                    progress = true;
                }
            } else {
                next.push(ring_index);
            }
        }
        pending = next;
        if !progress {
            break;
        }
    }
    // Last resort for an exotic ring system: attach the remaining atoms around a placed
    // ring atom without overlaps.
    for ring_index in pending {
        for atom in &rings[ring_index] {
            if points[*atom].is_some() {
                continue;
            }
            let parent = ring_index.checked_sub(0).and_then(|_| {
                rings[ring_index]
                    .iter()
                    .copied()
                    .find(|candidate| points[*candidate].is_some())
            });
            if let Some(parent) = parent {
                let direction: f64 = 0.0;
                let origin = points[parent].expect("placed");
                points[*atom] = Some([
                    origin[0] + BOND_LENGTH * direction.cos(),
                    origin[1] + BOND_LENGTH * direction.sin(),
                ]);
                placed.push(*atom);
            }
        }
    }
    placed
}

fn place_fused_ring(ring: &[usize], points: &mut [Option<[f64; 2]>]) -> Vec<usize> {
    let count = ring.len();
    let Some((first, second)) = (0..count).find_map(|index| {
        let a = ring[index];
        let b = ring[(index + 1) % count];
        (points[a].is_some() && points[b].is_some()).then_some((index, (index + 1) % count))
    }) else {
        return Vec::new();
    };
    let p0 = points[ring[first]].expect("placed");
    let p1 = points[ring[second]].expect("placed");
    let edge = [p1[0] - p0[0], p1[1] - p0[1]];
    let length = edge[0].hypot(edge[1]);
    if length <= f64::EPSILON {
        return Vec::new();
    }
    let midpoint = [(p0[0] + p1[0]) * 0.5, (p0[1] + p1[1]) * 0.5];
    let normal = [-edge[1] / length, edge[0] / length];
    let apothem = length / (2.0 * (std::f64::consts::PI / count as f64).tan());
    let candidates = [
        [midpoint[0] + normal[0] * apothem, midpoint[1] + normal[1] * apothem],
        [midpoint[0] - normal[0] * apothem, midpoint[1] - normal[1] * apothem],
    ];
    let center = candidates
        .into_iter()
        .max_by(|left, right| {
            clearance(left, points, ring)
                .partial_cmp(&clearance(right, points, ring))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap_or(midpoint);
    let angle0 = (p0[1] - center[1]).atan2(p0[0] - center[0]);
    let angle1 = (p1[1] - center[1]).atan2(p1[0] - center[0]);
    let step = (second + count - first) % count;
    let mut sign = if step == 1 {
        wrap_signed(angle1 - angle0)
    } else {
        wrap_signed(angle0 - angle1)
    };
    if sign == 0.0 {
        sign = 1.0;
    }
    let sign = if sign < 0.0 { -1.0 } else { 1.0 };
    let radius = length / (2.0 * (std::f64::consts::PI / count as f64).sin());
    let mut placed = Vec::new();
    for step in 0..count {
        let atom = ring[(first + step) % count];
        if points[atom].is_some() {
            continue;
        }
        let angle = angle0 + sign * std::f64::consts::TAU * step as f64 / count as f64;
        points[atom] = Some([center[0] + radius * angle.cos(), center[1] + radius * angle.sin()]);
        placed.push(atom);
    }
    placed
}

fn place_spiro_ring(ring: &[usize], shared: usize, points: &mut [Option<[f64; 2]>]) -> Vec<usize> {
    let count = ring.len();
    let Some(point) = points[shared] else {
        return Vec::new();
    };
    let radius = polygon_radius(count);
    let mut best = None;
    for step in 0..24 {
        let angle = std::f64::consts::TAU * step as f64 / 24.0;
        let center = [point[0] + radius * angle.cos(), point[1] + radius * angle.sin()];
        let mut score = f64::INFINITY;
        for offset in 0..count {
            let vertex = [
                center[0]
                    + radius
                        * (angle + std::f64::consts::PI + std::f64::consts::TAU * offset as f64 / count as f64).cos(),
                center[1]
                    + radius
                        * (angle + std::f64::consts::PI + std::f64::consts::TAU * offset as f64 / count as f64).sin(),
            ];
            for (index, other) in points.iter().enumerate() {
                if ring.contains(&index) || other.is_none() {
                    continue;
                }
                let other = other.expect("point");
                score = score.min((vertex[0] - other[0]).hypot(vertex[1] - other[1]));
            }
        }
        if best.is_none_or(|(best_score, _)| score > best_score) {
            best = Some((score, angle));
        }
    }
    let angle = best.map_or(0.0, |(_, angle)| angle);
    let center = [point[0] + radius * angle.cos(), point[1] + radius * angle.sin()];
    let start = angle + std::f64::consts::PI;
    let shared_index = ring.iter().position(|atom| *atom == shared).unwrap_or(0);
    let mut placed = Vec::new();
    for step in 0..count {
        let atom = ring[(shared_index + step) % count];
        if points[atom].is_some() {
            continue;
        }
        let vertex_angle = start + std::f64::consts::TAU * step as f64 / count as f64;
        points[atom] = Some([
            center[0] + radius * vertex_angle.cos(),
            center[1] + radius * vertex_angle.sin(),
        ]);
        placed.push(atom);
    }
    placed
}

fn clearance(center: &[f64; 2], points: &[Option<[f64; 2]>], ring: &[usize]) -> f64 {
    points
        .iter()
        .enumerate()
        .filter(|(index, point)| point.is_some() && !ring.contains(index))
        .map(|(_, point)| {
            let point = point.expect("point");
            (point[0] - center[0]).hypot(point[1] - center[1])
        })
        .fold(f64::INFINITY, f64::min)
}

fn collect_component(seed: usize, adjacency: &[Vec<usize>], points: &[Option<[f64; 2]>]) -> Vec<usize> {
    let mut component = Vec::new();
    let mut seen = vec![false; points.len()];
    let mut stack = vec![seed];
    seen[seed] = true;
    while let Some(atom) = stack.pop() {
        component.push(atom);
        for next in &adjacency[atom] {
            if !seen[*next] && points[*next].is_none() {
                seen[*next] = true;
                stack.push(*next);
            }
        }
    }
    component.sort_unstable();
    component
}

fn layout_acyclic_component(component: &[usize], adjacency: &[Vec<usize>], points: &mut [Option<[f64; 2]>]) {
    if component.is_empty() {
        return;
    }
    if component.len() == 1 {
        points[component[0]] = Some([0.0, 0.0]);
        return;
    }
    let allowed = component.iter().copied().collect::<std::collections::HashSet<_>>();
    let path = tree_diameter_path(component[0], &allowed, adjacency);
    if path.len() >= 2 {
        points[path[0]] = Some([0.0, 0.0]);
        for step in 1..path.len() {
            let previous = points[path[step - 1]].expect("placed path");
            let angle = if (step - 1) % 2 == 0 {
                std::f64::consts::FRAC_PI_6
            } else {
                -std::f64::consts::FRAC_PI_6
            };
            points[path[step]] = Some([
                previous[0] + BOND_LENGTH * angle.cos(),
                previous[1] + BOND_LENGTH * angle.sin(),
            ]);
        }
    } else {
        points[path.first().copied().unwrap_or(component[0])] = Some([0.0, 0.0]);
    }
    let mut queue = VecDeque::from(path);
    while let Some(atom) = queue.pop_front() {
        let Some(origin) = points[atom] else {
            continue;
        };
        for neighbor in adjacency[atom].clone() {
            if !allowed.contains(&neighbor) || points[neighbor].is_some() {
                continue;
            }
            let direction = next_direction(points, adjacency, atom, BOND_LENGTH);
            points[neighbor] = Some([
                origin[0] + BOND_LENGTH * direction.cos(),
                origin[1] + BOND_LENGTH * direction.sin(),
            ]);
            queue.push_back(neighbor);
        }
    }
    // A component with no path (should be impossible for a connected acyclic graph) still
    // gets every atom placed.
    for atom in component {
        if points[*atom].is_none() {
            points[*atom] = Some([0.0, 0.0]);
        }
    }
}

fn tree_diameter_path(
    start: usize,
    allowed: &std::collections::HashSet<usize>,
    adjacency: &[Vec<usize>],
) -> Vec<usize> {
    let (farthest, _) = farthest_atom(start, allowed, adjacency);
    let (other, parent) = farthest_atom(farthest, allowed, adjacency);
    let mut path = vec![other];
    let mut current = other;
    while let Some(previous) = parent[current] {
        path.push(previous);
        current = previous;
    }
    path.reverse();
    path
}

fn farthest_atom(
    start: usize,
    allowed: &std::collections::HashSet<usize>,
    adjacency: &[Vec<usize>],
) -> (usize, Vec<Option<usize>>) {
    let mut parent = vec![None; adjacency.len()];
    let mut seen = vec![false; adjacency.len()];
    let mut last = start;
    let mut queue = VecDeque::from([start]);
    seen[start] = true;
    while let Some(atom) = queue.pop_front() {
        last = atom;
        for neighbor in &adjacency[atom] {
            if allowed.contains(neighbor) && !seen[*neighbor] {
                seen[*neighbor] = true;
                parent[*neighbor] = Some(atom);
                queue.push_back(*neighbor);
            }
        }
    }
    (last, parent)
}

fn ring_entry_direction(
    points: &[Option<[f64; 2]>],
    adjacency: &[Vec<usize>],
    atom: usize,
    entry_atom: usize,
    system: usize,
    systems: &[Vec<usize>],
    rings: &[Vec<usize>],
) -> f64 {
    let Some(origin) = points[atom] else {
        return 0.0;
    };
    let mut candidates = Vec::new();
    for neighbor in &adjacency[atom] {
        if let Some(point) = points[*neighbor] {
            let angle = (point[1] - origin[1]).atan2(point[0] - origin[0]);
            candidates.push(normalize_angle(angle + CHAIN_ANGLE));
            candidates.push(normalize_angle(angle - CHAIN_ANGLE));
        }
    }
    let reach = BOND_LENGTH + ring_system_radius(system, systems, rings);
    candidates.push(next_direction(points, adjacency, atom, reach));
    // Crowded ring systems need a direction that clears every new vertex, not only the
    // 120° candidates; include the full circle as a fallback.
    for step in 0..24 {
        candidates.push(std::f64::consts::TAU * step as f64 / 24.0);
    }
    candidates.sort_by(f64::total_cmp);
    candidates.dedup_by(|left, right| (*left - *right).abs() < 1.0e-9);
    let mut best: Option<(f64, f64)> = None;
    for direction in candidates {
        let entry = [
            origin[0] + BOND_LENGTH * direction.cos(),
            origin[1] + BOND_LENGTH * direction.sin(),
        ];
        let mut trial = points.to_vec();
        let new = place_ring_system(
            system,
            systems,
            rings,
            &mut trial,
            Some(entry_atom),
            Some(entry),
            direction,
        );
        if new.is_empty() {
            continue;
        }
        let mut clearance = f64::INFINITY;
        for atom in &new {
            let Some(point) = trial[*atom] else {
                continue;
            };
            for (other, existing) in trial.iter().enumerate() {
                if new.contains(&other) || existing.is_none() {
                    continue;
                }
                let existing = existing.expect("placed");
                clearance = clearance.min((point[0] - existing[0]).hypot(point[1] - existing[1]));
            }
        }
        if best.is_none_or(|(best_clearance, _)| clearance > best_clearance + 1.0e-9) {
            best = Some((clearance, direction));
        }
    }
    best.map_or_else(
        || next_direction(points, adjacency, atom, reach),
        |(_, direction)| direction,
    )
}

fn next_direction(points: &[Option<[f64; 2]>], adjacency: &[Vec<usize>], atom: usize, reach: f64) -> f64 {
    let Some(origin) = points[atom] else {
        return 0.0;
    };
    let directions = adjacency[atom]
        .iter()
        .filter_map(|neighbor| points[*neighbor])
        .map(|neighbor| (neighbor[1] - origin[1]).atan2(neighbor[0] - origin[0]))
        .collect::<Vec<_>>();
    if directions.is_empty() {
        let others = points
            .iter()
            .enumerate()
            .filter(|(index, point)| *index != atom && point.is_some())
            .map(|(_, point)| point.expect("point"))
            .collect::<Vec<_>>();
        if others.is_empty() {
            return 0.0;
        }
        let center = centroid(&others);
        let away = [origin[0] - center[0], origin[1] - center[1]];
        return if away[0].hypot(away[1]) <= f64::EPSILON {
            0.0
        } else {
            away[1].atan2(away[0])
        };
    }
    let crowded = directions.len() >= 3;
    let offsets = if crowded {
        vec![
            -std::f64::consts::PI,
            -5.0 * std::f64::consts::FRAC_PI_6,
            -2.0 * std::f64::consts::FRAC_PI_3,
            -std::f64::consts::FRAC_PI_2,
            -std::f64::consts::FRAC_PI_3,
            std::f64::consts::FRAC_PI_3,
            std::f64::consts::FRAC_PI_2,
            2.0 * std::f64::consts::FRAC_PI_3,
            5.0 * std::f64::consts::FRAC_PI_6,
            std::f64::consts::PI,
        ]
    } else {
        vec![-CHAIN_ANGLE, CHAIN_ANGLE]
    };
    let mut candidates = Vec::<(f64, f64, f64)>::new();
    for direction in &directions {
        for offset in &offsets {
            let candidate = normalize_angle(direction + offset);
            let separation = directions
                .iter()
                .map(|existing| angle_distance(candidate, *existing))
                .fold(f64::INFINITY, f64::min);
            let probe = [origin[0] + reach * candidate.cos(), origin[1] + reach * candidate.sin()];
            let clear = points
                .iter()
                .enumerate()
                .filter(|(index, point)| *index != atom && point.is_some())
                .map(|(_, point)| {
                    let point = point.expect("point");
                    (point[0] - probe[0]).hypot(point[1] - probe[1])
                })
                .fold(f64::INFINITY, f64::min);
            candidates.push((separation, clear, candidate));
        }
    }
    const MIN_SEPARATION: f64 = std::f64::consts::FRAC_PI_3;
    let usable = candidates
        .iter()
        .filter(|(separation, _, _)| *separation >= MIN_SEPARATION - 1.0e-9)
        .copied()
        .collect::<Vec<_>>();
    let pool = if usable.is_empty() { candidates } else { usable };
    let mut best: Option<(f64, f64, f64)> = None;
    for (separation, clear, candidate) in pool {
        let better = if crowded {
            // Crowded centres care most about not drawing two bonds through each other.
            clear > best.map_or(f64::NEG_INFINITY, |(_, clear, _)| clear) + 1.0e-9
                || (best.is_some_and(|(_, best_clear, _)| (clear - best_clear).abs() <= 1.0e-9)
                    && best.is_some_and(|(best_separation, _, best_angle)| {
                        separation > best_separation + 1.0e-9
                            || ((separation - best_separation).abs() <= 1.0e-9 && candidate < best_angle)
                    }))
        } else {
            best.is_none_or(|(best_separation, best_clear, best_angle)| {
                separation > best_separation + 1.0e-9
                    || ((separation - best_separation).abs() <= 1.0e-9
                        && (clear > best_clear + 1.0e-9
                            || ((clear - best_clear).abs() <= 1.0e-9 && candidate < best_angle)))
            })
        };
        if better {
            best = Some((separation, clear, candidate));
        }
    }
    best.map_or(0.0, |(_, _, angle)| angle)
}

fn ring_system_radius(system: usize, systems: &[Vec<usize>], rings: &[Vec<usize>]) -> f64 {
    systems
        .get(system)
        .map(|system| {
            system
                .iter()
                .map(|ring| polygon_radius(rings[*ring].len()))
                .fold(0.0, f64::max)
        })
        .unwrap_or(0.0)
}

fn polygon_radius(count: usize) -> f64 {
    if count < 3 {
        return BOND_LENGTH;
    }
    BOND_LENGTH / (2.0 * (std::f64::consts::PI / count as f64).sin())
}

fn relocate_leaves(molecule: &Molecule2D, adjacency: &[Vec<usize>], points: &mut [[f64; 2]]) {
    // Two passes let two terminal branches that landed on the same point move apart.
    for _ in 0..2 {
        for (index, atom) in molecule.atoms.iter().enumerate() {
            if adjacency[index].len() != 1 || adjacency.len() == 2 {
                continue;
            }
            let parent = adjacency[index][0];
            let origin = points[parent];
            let bond = if atom.element.atomic_number() == 1 {
                1.2
            } else {
                BOND_LENGTH
            };
            if atom.element.atomic_number() == 1 {
                let delta = [points[index][0] - origin[0], points[index][1] - origin[1]];
                let length = delta[0].hypot(delta[1]);
                if length > f64::EPSILON {
                    points[index] = [
                        origin[0] + bond * delta[0] / length,
                        origin[1] + bond * delta[1] / length,
                    ];
                }
            }
            let current_clearance = points
                .iter()
                .enumerate()
                .filter(|(other, _)| *other != index && *other != parent)
                .map(|(_, point)| (point[0] - points[index][0]).hypot(point[1] - points[index][1]))
                .fold(f64::INFINITY, f64::min);
            if current_clearance > 1.0 {
                continue;
            }
            let mut best = (f64::NEG_INFINITY, 0.0_f64);
            for step in 0..36 {
                let angle = std::f64::consts::TAU * step as f64 / 36.0;
                let candidate = [origin[0] + bond * angle.cos(), origin[1] + bond * angle.sin()];
                let clearance = points
                    .iter()
                    .enumerate()
                    .filter(|(other, _)| *other != index && *other != parent)
                    .map(|(_, point)| (point[0] - candidate[0]).hypot(point[1] - candidate[1]))
                    .fold(f64::INFINITY, f64::min);
                if clearance > best.0 + 1.0e-9 {
                    best = (clearance, angle);
                }
            }
            let angle = best.1;
            points[index] = [origin[0] + bond * angle.cos(), origin[1] + bond * angle.sin()];
        }
    }
}

fn pack_components(points: &mut [[f64; 2]], adjacency: &[Vec<usize>]) {
    let count = points.len();
    let mut seen = vec![false; count];
    let mut components = Vec::<Vec<usize>>::new();
    for start in 0..count {
        if seen[start] {
            continue;
        }
        let mut component = Vec::new();
        let mut stack = vec![start];
        seen[start] = true;
        while let Some(atom) = stack.pop() {
            component.push(atom);
            for next in &adjacency[atom] {
                if !seen[*next] {
                    seen[*next] = true;
                    stack.push(*next);
                }
            }
        }
        component.sort_unstable();
        components.push(component);
    }
    components.sort_by_key(|component| component[0]);
    let mut cursor = 0.0;
    for component in components {
        let min_x = component
            .iter()
            .map(|atom| points[*atom][0])
            .fold(f64::INFINITY, f64::min);
        let max_x = component
            .iter()
            .map(|atom| points[*atom][0])
            .fold(f64::NEG_INFINITY, f64::max);
        let shift = cursor - min_x;
        for atom in &component {
            points[*atom][0] += shift;
        }
        cursor += max_x - min_x + COMPONENT_GAP;
    }
    let min_x = points.iter().map(|point| point[0]).fold(f64::INFINITY, f64::min);
    let max_x = points.iter().map(|point| point[0]).fold(f64::NEG_INFINITY, f64::max);
    let shift = -(min_x + max_x) * 0.5;
    for point in points {
        point[0] += shift;
    }
}

fn first_bond_angle(molecule: &Molecule2D, points: &[[f64; 2]]) -> Option<f64> {
    let index = molecule
        .atoms
        .iter()
        .enumerate()
        .map(|(index, atom)| (atom.id, index))
        .collect::<HashMap<_, _>>();
    for bond in &molecule.bonds {
        if let (Some(a), Some(b)) = (index.get(&bond.a), index.get(&bond.b)) {
            let delta = [points[*b][0] - points[*a][0], points[*b][1] - points[*a][1]];
            if delta[0].hypot(delta[1]) > 1.0e-6 {
                return Some(delta[1].atan2(delta[0]));
            }
        }
    }
    None
}

fn centroid(points: &[[f64; 2]]) -> [f64; 2] {
    if points.is_empty() {
        return [0.0; 2];
    }
    let mut sum = [0.0; 2];
    for point in points {
        sum[0] += point[0];
        sum[1] += point[1];
    }
    [sum[0] / points.len() as f64, sum[1] / points.len() as f64]
}

fn normalize_angle(angle: f64) -> f64 {
    angle.rem_euclid(std::f64::consts::TAU)
}

fn wrap_signed(angle: f64) -> f64 {
    let mut angle = angle.rem_euclid(std::f64::consts::TAU);
    if angle > std::f64::consts::PI {
        angle -= std::f64::consts::TAU;
    }
    angle
}

fn angle_distance(a: f64, b: f64) -> f64 {
    let distance = (a - b).rem_euclid(std::f64::consts::TAU);
    distance.min(std::f64::consts::TAU - distance)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::ChemDocument;
    use crate::templates::{RingTemplate, add_ring};
    use egui_workbench_chem_core::{BondOrder, DocumentId, Element};

    fn carbon() -> Element {
        Element::from_symbol("C").expect("carbon")
    }

    #[test]
    fn a_zigzag_chain_has_uniform_bonds_and_120_degree_angles() {
        let mut molecule = Molecule2D::new();
        let ids = (0..5)
            .map(|index| molecule.add_atom(carbon(), [index as f64 * 1.1, (index % 2) as f64 * 0.7, 0.0]))
            .collect::<Vec<_>>();
        for pair in ids.windows(2) {
            molecule.add_bond(pair[0], pair[1], BondOrder::Single).expect("bond");
        }
        optimize_layout(&mut molecule);
        let points = ids
            .iter()
            .map(|id| {
                let atom = molecule.atom(*id).expect("atom");
                [atom.position[0], atom.position[1]]
            })
            .collect::<Vec<_>>();
        for pair in points.windows(2) {
            let length = (pair[1][0] - pair[0][0]).hypot(pair[1][1] - pair[0][1]);
            assert!((length - BOND_LENGTH).abs() < 0.08, "{length}");
        }
        for triple in points.windows(3) {
            let a = [triple[0][0] - triple[1][0], triple[0][1] - triple[1][1]];
            let b = [triple[2][0] - triple[1][0], triple[2][1] - triple[1][1]];
            let angle = ((a[0] * b[0] + a[1] * b[1]) / (a[0].hypot(a[1]) * b[0].hypot(b[1]))).acos();
            assert!((angle.to_degrees() - 120.0).abs() < 8.0, "{angle}");
        }
    }

    #[test]
    fn a_collinear_three_atom_chain_opens_to_120() {
        let mut molecule = Molecule2D::new();
        let a = molecule.add_atom(carbon(), [-1.5, 0.0, 0.0]);
        let b = molecule.add_atom(carbon(), [0.0, 0.0, 0.0]);
        let c = molecule.add_atom(carbon(), [1.5, 0.0, 0.0]);
        molecule.add_bond(a, b, BondOrder::Single).expect("bond");
        molecule.add_bond(b, c, BondOrder::Single).expect("bond");
        optimize_layout(&mut molecule);
        let a = molecule.atom(a).expect("a");
        let b = molecule.atom(b).expect("b");
        let c = molecule.atom(c).expect("c");
        let u = [a.position[0] - b.position[0], a.position[1] - b.position[1]];
        let v = [c.position[0] - b.position[0], c.position[1] - b.position[1]];
        let angle = ((u[0] * v[0] + u[1] * v[1]) / (u[0].hypot(u[1]) * v[0].hypot(v[1]))).acos();
        assert!((angle.to_degrees() - 120.0).abs() < 8.0, "{}", angle.to_degrees());
    }

    #[test]
    fn a_distorted_benzene_becomes_regular() {
        let mut molecule = Molecule2D::new();
        let ids = add_ring(&mut molecule, RingTemplate::Benzene, [0.0, 0.0], 0.0);
        for (index, id) in ids.iter().enumerate() {
            let atom = molecule.atom_mut(*id).expect("atom");
            atom.position[0] *= 0.6 + index as f64 * 0.08;
            atom.position[1] *= 0.5;
        }
        optimize_layout(&mut molecule);
        let points = ids
            .iter()
            .map(|id| {
                let atom = molecule.atom(*id).expect("atom");
                [atom.position[0], atom.position[1]]
            })
            .collect::<Vec<_>>();
        for (left, right) in points.iter().zip(points.iter().cycle().skip(1)).take(6) {
            let length = (right[0] - left[0]).hypot(right[1] - left[1]);
            assert!((length - BOND_LENGTH).abs() < 0.1, "{length}");
        }
    }

    #[test]
    fn a_five_membered_ring_uses_its_own_interior_angle() {
        let mut molecule = Molecule2D::new();
        let ids = add_ring(&mut molecule, RingTemplate::Cyclopentadiene, [0.0, 0.0], 0.0);
        for (index, id) in ids.iter().enumerate() {
            let atom = molecule.atom_mut(*id).expect("atom");
            atom.position[0] *= 0.8 + index as f64 * 0.05;
            atom.position[1] *= 0.7;
        }
        optimize_layout(&mut molecule);
        let points = ids
            .iter()
            .map(|id| {
                let atom = molecule.atom(*id).expect("atom");
                [atom.position[0], atom.position[1]]
            })
            .collect::<Vec<_>>();
        for index in 0..points.len() {
            let previous = points[(index + points.len() - 1) % points.len()];
            let center = points[index];
            let next = points[(index + 1) % points.len()];
            let a = [previous[0] - center[0], previous[1] - center[1]];
            let b = [next[0] - center[0], next[1] - center[1]];
            let angle = ((a[0] * b[0] + a[1] * b[1]) / (a[0].hypot(a[1]) * b[0].hypot(b[1]))).acos();
            assert!((angle.to_degrees() - 108.0).abs() < 8.0, "{}", angle.to_degrees());
        }
    }

    #[test]
    fn the_optimizer_is_idempotent_and_keeps_the_centre() {
        let mut molecule = Molecule2D::new();
        let a = molecule.add_atom(carbon(), [4.0, 3.0, 0.0]);
        let b = molecule.add_atom(carbon(), [4.6, 3.2, 0.0]);
        molecule.add_bond(a, b, BondOrder::Single).expect("bond");
        let before = centroid(
            &molecule
                .atoms
                .iter()
                .map(|atom| [atom.position[0], atom.position[1]])
                .collect::<Vec<_>>(),
        );
        optimize_layout(&mut molecule);
        let first = molecule.atoms.iter().map(|atom| atom.position).collect::<Vec<_>>();
        optimize_layout(&mut molecule);
        let second = molecule.atoms.iter().map(|atom| atom.position).collect::<Vec<_>>();
        let moved = first
            .iter()
            .zip(&second)
            .map(|(a, b)| (a[0] - b[0]).hypot(a[1] - b[1]))
            .fold(0.0_f64, f64::max);
        assert!(moved < 1.0e-9, "{moved}");
        let after = centroid(
            &molecule
                .atoms
                .iter()
                .map(|atom| [atom.position[0], atom.position[1]])
                .collect::<Vec<_>>(),
        );
        assert!((before[0] - after[0]).abs() < 1.0e-9);
        assert!((before[1] - after[1]).abs() < 1.0e-9);
    }

    #[test]
    fn unbound_atoms_on_top_of_each_other_separate() {
        let mut molecule = Molecule2D::new();
        let a = molecule.add_atom(carbon(), [0.0, 0.0, 0.0]);
        let b = molecule.add_atom(carbon(), [0.0, 0.0, 0.0]);
        optimize_layout(&mut molecule);
        let a = molecule.atom(a).expect("a");
        let b = molecule.atom(b).expect("b");
        let distance = (a.position[0] - b.position[0]).hypot(a.position[1] - b.position[1]);
        assert!(distance > 1.2, "{distance}");
    }

    #[test]
    fn overlapping_atoms_separate() {
        let mut molecule = Molecule2D::new();
        let a = molecule.add_atom(carbon(), [0.0, 0.0, 0.0]);
        let b = molecule.add_atom(carbon(), [0.01, 0.0, 0.0]);
        let c = molecule.add_atom(carbon(), [0.0, 0.01, 0.0]);
        molecule.add_bond(a, b, BondOrder::Single).expect("bond");
        molecule.add_bond(a, c, BondOrder::Single).expect("bond");
        optimize_layout(&mut molecule);
        let a = molecule.atom(a).expect("atom");
        let b = molecule.atom(b).expect("atom");
        let c = molecule.atom(c).expect("atom");
        let ab = (a.position[0] - b.position[0]).hypot(a.position[1] - b.position[1]);
        let ac = (a.position[0] - c.position[0]).hypot(a.position[1] - c.position[1]);
        let bc = (b.position[0] - c.position[0]).hypot(b.position[1] - c.position[1]);
        assert!(ab > 1.2 && ac > 1.2 && bc > 1.8, "{ab} {ac} {bc}");
    }

    #[test]
    fn a_ring_document_stays_valid() {
        let mut document = ChemDocument::new(DocumentId::new(1), "layout");
        add_ring(&mut document.molecule, RingTemplate::Pyridine, [0.0, 0.0], 0.0);
        document.molecule.validate().expect("valid");
        optimize_layout(&mut document.molecule);
        document.molecule.validate().expect("valid");
    }
}
