//! A practical SMILES reader/writer for the 2D editor.

use std::collections::{BTreeMap, HashSet};

use egui_workbench_chem_core::{BondOrder, Element, Molecule};

use crate::document::{ChemDocument, Molecule2D};

/// Why a SMILES string could not be parsed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SmilesError {
    /// Human-readable reason.
    pub message: String,
}

impl std::fmt::Display for SmilesError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for SmilesError {}

#[derive(Clone, Debug, PartialEq)]
struct SmilesAtom {
    element: Element,
    charge: i8,
    aromatic: bool,
    hydrogens: usize,
}

/// A parsed SMILES graph.
#[derive(Clone, Debug, PartialEq)]
pub struct SmilesGraph {
    /// Atoms in parse order.
    pub atoms: Vec<Element>,
    /// Formal charges.
    pub charges: Vec<i8>,
    /// Aromatic flags.
    pub aromatic: Vec<bool>,
    /// Bonds with orders.
    pub bonds: Vec<(usize, usize, BondOrder)>,
    hydrogens: Vec<usize>,
}

/// Parse a SMILES string.
///
/// Supports the organic subset, bracket atoms with charge/hydrogen counts, branches,
/// ring closures 1-9, aromatic lowercase atoms, and `-`, `=`, `#`, `:` bond symbols.
///
/// # Errors
/// When the string has an unknown atom, an unmatched branch/ring, or an invalid bond.
pub fn parse_smiles(text: &str) -> Result<SmilesGraph, SmilesError> {
    let chars = text.chars().collect::<Vec<_>>();
    let mut atoms = Vec::<SmilesAtom>::new();
    let mut bonds = Vec::<(usize, usize, BondOrder)>::new();
    let mut current = None::<usize>;
    let mut branches = Vec::<usize>::new();
    let mut rings = BTreeMap::<u8, (usize, Option<BondOrder>)>::new();
    let mut pending = None::<BondOrder>;
    let mut index = 0;
    while index < chars.len() {
        let character = chars[index];
        match character {
            '(' => {
                branches.push(current.ok_or_else(|| error("branch without an atom"))?);
                index += 1;
            }
            ')' => {
                current = Some(branches.pop().ok_or_else(|| error("unmatched `)`"))?);
                index += 1;
            }
            '-' => {
                pending = Some(BondOrder::Single);
                index += 1;
            }
            '=' => {
                pending = Some(BondOrder::Double);
                index += 1;
            }
            '#' => {
                pending = Some(BondOrder::Triple);
                index += 1;
            }
            ':' => {
                pending = Some(BondOrder::Aromatic);
                index += 1;
            }
            '/' | '\\' => {
                pending = Some(BondOrder::Single);
                index += 1;
            }
            '.' => {
                current = None;
                pending = None;
                index += 1;
            }
            '0'..='9' => {
                let number = character as u8 - b'0';
                index += 1;
                let atom = current.ok_or_else(|| error("ring closure without an atom"))?;
                if let Some((other, stored)) = rings.remove(&number) {
                    let order = pending
                        .or(stored)
                        .unwrap_or_else(|| default_order(atoms[atom].aromatic, atoms[other].aromatic));
                    bonds.push((other, atom, order));
                } else {
                    rings.insert(number, (atom, pending));
                }
                pending = None;
            }
            '[' => {
                let start = index + 1;
                let mut end = start;
                while end < chars.len() && chars[end] != ']' {
                    end += 1;
                }
                if end >= chars.len() {
                    return Err(error("unclosed bracket atom"));
                }
                let body = chars[start..end].iter().collect::<String>();
                let atom = parse_bracket(&body)?;
                let atom_index = atoms.len();
                atoms.push(atom);
                if let Some(previous) = current {
                    bonds.push((previous, atom_index, pending.unwrap_or(BondOrder::Single)));
                }
                current = Some(atom_index);
                pending = None;
                index = end + 1;
            }
            'C' | 'N' | 'O' | 'P' | 'S' | 'F' | 'I' | 'B' => {
                let mut symbol = String::from(character);
                if index + 1 < chars.len() && matches!((character, chars[index + 1]), ('C', 'l') | ('B', 'r')) {
                    symbol.push(chars[index + 1]);
                    index += 1;
                }
                let element = Element::from_symbol(&symbol).ok_or_else(|| error("unknown element"))?;
                let atom_index = atoms.len();
                atoms.push(SmilesAtom {
                    element,
                    charge: 0,
                    aromatic: false,
                    hydrogens: 0,
                });
                if let Some(previous) = current {
                    bonds.push((previous, atom_index, pending.unwrap_or(BondOrder::Single)));
                }
                current = Some(atom_index);
                pending = None;
                index += 1;
            }
            'c' | 'n' | 'o' | 's' | 'p' => {
                let element = Element::from_symbol(&character.to_uppercase().to_string())
                    .ok_or_else(|| error("unknown aromatic element"))?;
                let atom_index = atoms.len();
                atoms.push(SmilesAtom {
                    element,
                    charge: 0,
                    aromatic: true,
                    hydrogens: 0,
                });
                if let Some(previous) = current {
                    bonds.push((previous, atom_index, pending.unwrap_or(BondOrder::Aromatic)));
                }
                current = Some(atom_index);
                pending = None;
                index += 1;
            }
            other => return Err(error(&format!("unexpected `{other}`"))),
        }
    }
    if !branches.is_empty() {
        return Err(error("unclosed branch"));
    }
    if !rings.is_empty() {
        return Err(error("unclosed ring"));
    }
    if atoms.is_empty() {
        return Err(error("empty SMILES"));
    }
    let hydrogens = atoms.iter().map(|atom| atom.hydrogens).collect::<Vec<_>>();
    Ok(SmilesGraph {
        atoms: atoms.iter().map(|atom| atom.element).collect(),
        charges: atoms.iter().map(|atom| atom.charge).collect(),
        aromatic: atoms.iter().map(|atom| atom.aromatic).collect(),
        bonds,
        hydrogens,
    })
}

fn parse_bracket(body: &str) -> Result<SmilesAtom, SmilesError> {
    let mut chars = body.chars().peekable();
    let mut symbol = String::new();
    if let Some(first) = chars.next() {
        if !first.is_ascii_alphabetic() {
            return Err(error("bracket atom must start with an element"));
        }
        symbol.push(first);
        if first.is_ascii_lowercase() {
            while chars.peek().is_some_and(|next| next.is_ascii_lowercase()) {
                symbol.push(chars.next().unwrap_or_default());
            }
        } else {
            while chars
                .peek()
                .is_some_and(|next| next.is_ascii_lowercase() && symbol.len() == 1)
            {
                symbol.push(chars.next().unwrap_or_default());
            }
        }
    }
    let aromatic = symbol.chars().next().is_some_and(|first| first.is_ascii_lowercase());
    let element = Element::from_symbol(&symbol.to_uppercase()).ok_or_else(|| error("unknown bracket element"))?;
    let mut hydrogens = 0;
    let mut charge = 0;
    let rest = chars.collect::<String>();
    if let Some(position) = rest.find('H') {
        let suffix = &rest[position + 1..];
        let digits = suffix
            .chars()
            .take_while(|character| character.is_ascii_digit())
            .collect::<String>();
        hydrogens = digits.parse().unwrap_or(1);
        // `to_molecule` adds one atom per hydrogen, so an unchecked count is a memory bomb.
        if hydrogens > MAX_BRACKET_HYDROGENS {
            return Err(error("too many hydrogens on a bracket atom"));
        }
    }
    for sign in ['+', '-'] {
        if let Some(position) = rest.find(sign) {
            let digits = rest[position + 1..]
                .chars()
                .take_while(|character| character.is_ascii_digit())
                .collect::<String>();
            // `[Fe++]` repeats the sign instead of giving a digit.
            let repeated = rest[position..]
                .chars()
                .take_while(|character| *character == sign)
                .count();
            let magnitude = if digits.is_empty() {
                i8::try_from(repeated).unwrap_or(i8::MAX)
            } else {
                digits.parse::<i8>().map_err(|_| error("charge out of range"))?
            };
            charge = if sign == '+' { magnitude } else { -magnitude };
            break;
        }
    }
    Ok(SmilesAtom {
        element,
        charge,
        aromatic,
        hydrogens,
    })
}

/// Most explicit hydrogens accepted inside one bracket atom.
const MAX_BRACKET_HYDROGENS: usize = 8;

fn default_order(left: bool, right: bool) -> BondOrder {
    if left && right {
        BondOrder::Aromatic
    } else {
        BondOrder::Single
    }
}

fn error(message: &str) -> SmilesError {
    SmilesError {
        message: message.to_owned(),
    }
}

impl SmilesGraph {
    /// Build a 2D molecule with a deterministic force-directed layout.
    pub fn to_molecule(&self) -> Molecule2D {
        let mut molecule = Molecule2D::new();
        let count = self.atoms.len();
        let mut ids = Vec::with_capacity(count);
        for index in 0..count {
            let angle = std::f64::consts::TAU * index as f64 / count.max(1) as f64;
            let radius = 1.4 * (count as f64).sqrt();
            let id = molecule.add_atom(self.atoms[index], [radius * angle.cos(), radius * angle.sin(), 0.0]);
            if let Some(atom) = molecule.atom_mut(id) {
                atom.charge = self.charges[index];
            }
            ids.push(id);
        }
        for (a, b, order) in &self.bonds {
            if let (Some(a), Some(b)) = (ids.get(*a), ids.get(*b)) {
                let _ = molecule.add_bond(*a, *b, *order);
            }
        }
        for (index, count) in self.hydrogens.iter().enumerate() {
            if let Some(parent) = ids.get(index) {
                for _ in 0..*count {
                    let position = molecule.atom(*parent).map_or([0.0, 0.0, 0.0], |atom| atom.position);
                    let hydrogen = molecule.add_atom(
                        Element::from_symbol("H").unwrap_or(Element::UNKNOWN),
                        [position[0] + 1.0, position[1] + 0.5, position[2]],
                    );
                    let _ = molecule.add_bond(*parent, hydrogen, BondOrder::Single);
                }
            }
        }
        crate::layout::relax(&mut molecule);
        molecule
    }

    /// Add the parsed graph at `center` and return its atom ids.
    pub fn insert(&self, document: &mut ChemDocument, center: [f64; 2]) -> Vec<egui_workbench_chem_core::AtomId> {
        let molecule = self.to_molecule();
        insert_molecule(document, &molecule, center)
    }
}

/// Copy a laid-out molecule into a document, offset by `offset`.
pub(crate) fn insert_molecule(
    document: &mut ChemDocument,
    molecule: &Molecule2D,
    offset: [f64; 2],
) -> Vec<egui_workbench_chem_core::AtomId> {
    let mut ids = Vec::new();
    for atom in &molecule.atoms {
        let id = document.molecule.add_atom(
            atom.element,
            [atom.position[0] + offset[0], atom.position[1] + offset[1], 0.0],
        );
        if let Some(target) = document.molecule.atom_mut(id) {
            target.charge = atom.charge;
            target.label.clone_from(&atom.label);
        }
        ids.push(id);
    }
    for bond in &molecule.bonds {
        let a = molecule.atoms.iter().position(|atom| atom.id == bond.a);
        let b = molecule.atoms.iter().position(|atom| atom.id == bond.b);
        if let (Some(a), Some(b)) = (a, b)
            && let (Some(a), Some(b)) = (ids.get(a), ids.get(b))
        {
            let _ = document.molecule.add_bond(*a, *b, bond.order);
        }
    }
    ids
}

/// Write a SMILES string for `molecule`.
///
/// The output is canonical only for the atom order in the molecule; it preserves
/// branches, ring closures, aromatic atoms, charges, and explicit hydrogens.
pub fn to_smiles(molecule: &Molecule) -> String {
    let count = molecule.atoms.len();
    if count == 0 {
        return String::new();
    }
    let index = molecule
        .atoms
        .iter()
        .enumerate()
        .map(|(index, atom)| (atom.id, index))
        .collect::<BTreeMap<_, _>>();
    let mut adjacency = vec![Vec::<(usize, BondOrder)>::new(); count];
    for bond in &molecule.bonds {
        if let (Some(a), Some(b)) = (index.get(&bond.a), index.get(&bond.b)) {
            adjacency[*a].push((*b, bond.order));
            adjacency[*b].push((*a, bond.order));
        }
    }
    let mut visited = vec![false; count];
    let mut tree = vec![Vec::<(usize, BondOrder)>::new(); count];
    let mut components = Vec::<usize>::new();
    for start in 0..count {
        if visited[start] {
            continue;
        }
        components.push(start);
        visited[start] = true;
        let mut stack = vec![start];
        while let Some(node) = stack.pop() {
            for (next, order) in &adjacency[node] {
                if !visited[*next] {
                    visited[*next] = true;
                    tree[node].push((*next, *order));
                    tree[*next].push((node, *order));
                    stack.push(*next);
                }
            }
        }
    }
    let mut tree_edges = HashSet::new();
    for (node, neighbours) in tree.iter().enumerate() {
        for (next, _) in neighbours {
            tree_edges.insert((node.min(*next), node.max(*next)));
        }
    }
    let mut closures = vec![Vec::<(usize, u8, BondOrder)>::new(); count];
    let mut closure_id = 1u8;
    for bond in &molecule.bonds {
        let (Some(a), Some(b)) = (index.get(&bond.a), index.get(&bond.b)) else {
            continue;
        };
        if tree_edges.contains(&((*a).min(*b), (*a).max(*b))) {
            continue;
        }
        closures[*a].push((*b, closure_id, bond.order));
        closures[*b].push((*a, closure_id, bond.order));
        closure_id = closure_id % 9 + 1;
    }
    let mut output = String::new();
    for (component, start) in components.iter().enumerate() {
        if component > 0 {
            output.push('.');
        }
        write_component(*start, usize::MAX, molecule, &tree, &closures, &mut output);
    }
    output
}

fn write_component(
    atom: usize,
    parent: usize,
    molecule: &Molecule,
    tree: &[Vec<(usize, BondOrder)>],
    closures: &[Vec<(usize, u8, BondOrder)>],
    output: &mut String,
) {
    let Some(entry) = molecule.atoms.get(atom) else {
        return;
    };
    let aromatic = closures[atom].iter().any(|(_, _, order)| *order == BondOrder::Aromatic)
        || tree[atom].iter().any(|(_, order)| *order == BondOrder::Aromatic);
    let symbol = if aromatic && matches!(entry.element.symbol(), "C" | "N" | "O" | "S" | "P") {
        entry.element.symbol().to_lowercase()
    } else {
        entry.element.symbol().to_owned()
    };
    let bracket = entry.charge != 0
        || !matches!(
            entry.element.symbol(),
            "B" | "C" | "N" | "O" | "P" | "S" | "F" | "Cl" | "Br" | "I"
        );
    if bracket {
        output.push('[');
        output.push_str(&symbol);
        if entry.charge > 0 {
            output.push('+');
            if entry.charge > 1 {
                output.push_str(&entry.charge.to_string());
            }
        } else if entry.charge < 0 {
            output.push('-');
            if entry.charge < -1 {
                output.push_str(&(-entry.charge).to_string());
            }
        }
        output.push(']');
    } else {
        output.push_str(&symbol);
    }
    for (_, number, _) in &closures[atom] {
        output.push_str(&number.to_string());
    }
    let children = tree[atom]
        .iter()
        .filter(|(next, _)| *next != parent)
        .copied()
        .collect::<Vec<_>>();
    for (position, (next, order)) in children.iter().enumerate() {
        let branch = position + 1 > 1;
        let ring = closures[atom].iter().any(|(other, _, _)| other == next);
        if branch {
            output.push('(');
        }
        if !ring {
            output.push_str(bond_symbol(*order));
        }
        write_component(*next, atom, molecule, tree, closures, output);
        if branch {
            output.push(')');
        }
    }
}

fn bond_symbol(order: BondOrder) -> &'static str {
    match order {
        BondOrder::Single | BondOrder::Aromatic => "",
        BondOrder::Double => "=",
        BondOrder::Triple => "#",
        BondOrder::Dative => "-",
        _ => "-",
    }
}

#[cfg(test)]
mod tests;
