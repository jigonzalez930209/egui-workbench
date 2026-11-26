//! Molfile V2000/V3000, SDF, and RXN readers and writers.

use egui_workbench_chem_core::{BondOrder, BondStereo, Element};

use crate::document::Molecule2D;

/// Why a molfile could not be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MolfileError {
    /// Human-readable reason.
    pub message: String,
}

impl std::fmt::Display for MolfileError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for MolfileError {}

impl From<crate::Chem2dError> for MolfileError {
    fn from(error: crate::Chem2dError) -> Self {
        Self {
            message: error.to_string(),
        }
    }
}

impl From<egui_workbench_chem_core::ChemError> for MolfileError {
    fn from(error: egui_workbench_chem_core::ChemError) -> Self {
        Self {
            message: error.to_string(),
        }
    }
}

/// Write a V2000 molfile.
pub fn to_v2000(molecule: &Molecule2D, name: &str) -> String {
    let mut output = format!(
        "{name}\n  egui-workbench\n\n{:>3}{:>3}  0  0  0  0  0  0  0  0999 V2000\n",
        molecule.atoms.len(),
        molecule.bonds.len()
    );
    for atom in &molecule.atoms {
        let symbol = atom.element.symbol();
        output.push_str(&format!(
            "{:>10.4}{:>10.4}{:>10.4} {:<3} 0  0  0  0  0  0  0  0  0  0  0  0\n",
            atom.position[0], atom.position[1], atom.position[2], symbol
        ));
    }
    for bond in &molecule.bonds {
        let a = molecule.atoms.iter().position(|atom| atom.id == bond.a).unwrap_or(0) + 1;
        let b = molecule.atoms.iter().position(|atom| atom.id == bond.b).unwrap_or(0) + 1;
        let order = match bond.order {
            BondOrder::Double => 2,
            BondOrder::Triple => 3,
            BondOrder::Aromatic => 4,
            _ => 1,
        };
        let stereo = match bond.stereo {
            BondStereo::Wedge => 1,
            BondStereo::Either => 3,
            BondStereo::Hash => 6,
            _ => 0,
        };
        output.push_str(&format!("{:>3}{:>3}{:>3}  {stereo}  0  0  0\n", a, b, order));
    }
    let charges = molecule
        .atoms
        .iter()
        .enumerate()
        .filter(|(_, atom)| atom.charge != 0)
        .collect::<Vec<_>>();
    // The format allows eight entries per `M  CHG` line.
    for chunk in charges.chunks(8) {
        output.push_str(&format!("M  CHG{:>3}", chunk.len()));
        for (index, atom) in chunk {
            output.push_str(&format!("{:>4}{:>4}", index + 1, atom.charge));
        }
        output.push('\n');
    }
    output.push_str("M  END\n");
    output
}

/// A fixed-width column, or `None` when it is out of range or splits a character.
fn field(line: &str, from: usize, to: usize) -> Option<&str> {
    line.get(from..to).map(str::trim)
}

/// Read a V2000 or V3000 molfile.
///
/// # Errors
/// When the counts line, an atom, or a bond is malformed.
pub fn from_molfile(text: &str) -> Result<Molecule2D, MolfileError> {
    let lines = text.lines().collect::<Vec<_>>();
    if lines.iter().any(|line| line.contains("V3000")) {
        return from_v3000(text);
    }
    let counts = lines
        .iter()
        .position(|line| line.contains("V2000"))
        .ok_or_else(|| error("missing V2000 counts line"))?;
    let count_line = lines[counts];
    let atom_count = count_line
        .get(0..3)
        .and_then(|part| part.trim().parse::<usize>().ok())
        .ok_or_else(|| error("invalid atom count"))?;
    let bond_count = count_line
        .get(3..6)
        .and_then(|part| part.trim().parse::<usize>().ok())
        .ok_or_else(|| error("invalid bond count"))?;
    let mut molecule = Molecule2D::new();
    let mut ids = Vec::with_capacity(atom_count);
    // `str::get` instead of `[a..b]`: a multi-byte character inside a fixed column would
    // make the slice panic on untrusted input.
    for line in lines.iter().skip(counts + 1).take(atom_count) {
        let coordinate = |from: usize, to: usize, name: &str| {
            field(line, from, to)
                .and_then(|part| part.parse::<f64>().ok())
                .filter(|value| value.is_finite())
                .ok_or_else(|| error(name))
        };
        let x = coordinate(0, 10, "bad x")?;
        let y = coordinate(10, 20, "bad y")?;
        let z = coordinate(20, 30, "bad z")?;
        let symbol = field(line, 31, 34).ok_or_else(|| error("short atom line"))?;
        let element = Element::from_symbol(symbol).ok_or_else(|| error("unknown element"))?;
        ids.push(molecule.add_atom(element, [x, y, z]));
    }
    if ids.len() < atom_count {
        return Err(error("truncated atom block"));
    }
    let bond_start = counts + 1 + atom_count;
    let mut bond_lines = 0;
    for line in lines.iter().skip(bond_start).take(bond_count) {
        bond_lines += 1;
        let a = field(line, 0, 3)
            .and_then(|part| part.parse::<usize>().ok())
            .ok_or_else(|| error("bad bond atom"))?;
        let b = field(line, 3, 6)
            .and_then(|part| part.parse::<usize>().ok())
            .ok_or_else(|| error("bad bond atom"))?;
        let order = match field(line, 6, 9).and_then(|part| part.parse::<u8>().ok()).unwrap_or(1) {
            2 => BondOrder::Double,
            3 => BondOrder::Triple,
            4 => BondOrder::Aromatic,
            _ => BondOrder::Single,
        };
        let stereo = match field(line, 9, 12).and_then(|part| part.parse::<u8>().ok()) {
            Some(1) => BondStereo::Wedge,
            Some(6) => BondStereo::Hash,
            Some(3) => BondStereo::Either,
            _ => BondStereo::None,
        };
        // Atom numbers are 1-based; `0` used to saturate to the first atom.
        if let (Some(a), Some(b)) = (
            a.checked_sub(1).and_then(|index| ids.get(index)),
            b.checked_sub(1).and_then(|index| ids.get(index)),
        ) {
            let id = molecule.add_bond(*a, *b, order)?;
            if let Some(entry) = molecule.bonds.iter_mut().find(|entry| entry.id == id) {
                entry.stereo = stereo;
            }
        }
    }
    if bond_lines < bond_count {
        return Err(error("truncated bond block"));
    }
    for line in lines.iter().filter(|line| line.starts_with("M  CHG")) {
        let numbers = line
            .get(6..)
            .unwrap_or_default()
            .split_whitespace()
            .filter_map(|part| part.parse::<i32>().ok())
            .collect::<Vec<_>>();
        // The first number is the entry count; a bare `M  CHG` line has none.
        for pair in numbers.get(1..).unwrap_or_default().chunks(2) {
            if let [atom, charge] = pair
                && let Some(index) = usize::try_from(*atom).ok().and_then(|atom| atom.checked_sub(1))
                && let Some(id) = ids.get(index)
                && let Some(entry) = molecule.atom_mut(*id)
                && let Ok(charge) = i8::try_from(*charge)
            {
                entry.charge = charge;
            }
        }
    }
    Ok(molecule)
}

/// Write a V3000 molfile.
pub fn to_v3000(molecule: &Molecule2D, name: &str) -> String {
    let mut output = format!(
        "{name}\n  egui-workbench\n\n  0  0  0  0  0  0  0  0  0  0999 V3000\nM  V30 BEGIN CTAB\nM  V30 COUNTS {} {} 0 0 0\nM  V30 BEGIN ATOM\n",
        molecule.atoms.len(),
        molecule.bonds.len()
    );
    for (index, atom) in molecule.atoms.iter().enumerate() {
        // Field six is the atom-atom mapping number; the charge is the `CHG=` property.
        let charge = if atom.charge == 0 {
            String::new()
        } else {
            format!(" CHG={}", atom.charge)
        };
        output.push_str(&format!(
            "M  V30 {} {} {:.4} {:.4} {:.4} 0{charge}\n",
            index + 1,
            atom.element.symbol(),
            atom.position[0],
            atom.position[1],
            atom.position[2],
        ));
    }
    output.push_str("M  V30 END ATOM\nM  V30 BEGIN BOND\n");
    for (index, bond) in molecule.bonds.iter().enumerate() {
        let a = molecule.atoms.iter().position(|atom| atom.id == bond.a).unwrap_or(0) + 1;
        let b = molecule.atoms.iter().position(|atom| atom.id == bond.b).unwrap_or(0) + 1;
        let order = match bond.order {
            BondOrder::Double => 2,
            BondOrder::Triple => 3,
            BondOrder::Aromatic => 4,
            _ => 1,
        };
        output.push_str(&format!("M  V30 {} {} {} {}\n", index + 1, order, a, b));
    }
    output.push_str("M  V30 END BOND\nM  V30 END CTAB\nM  END\n");
    output
}

/// Read a V3000 molfile.
///
/// # Errors
/// When a V30 atom or bond line is malformed.
pub fn from_v3000(text: &str) -> Result<Molecule2D, MolfileError> {
    let mut molecule = Molecule2D::new();
    let mut ids = Vec::new();
    let mut section = "";
    for line in text.lines() {
        let Some(rest) = line.strip_prefix("M  V30 ") else {
            continue;
        };
        match rest.trim() {
            "BEGIN ATOM" => {
                section = "atom";
                continue;
            }
            "END ATOM" => {
                section = "";
                continue;
            }
            "BEGIN BOND" => {
                section = "bond";
                continue;
            }
            "END BOND" => {
                section = "";
                continue;
            }
            _ => {}
        }
        let fields = rest.split_whitespace().collect::<Vec<_>>();
        match section {
            "atom" if fields.len() >= 5 => {
                let element = Element::from_symbol(fields[1]).ok_or_else(|| error("unknown V30 element"))?;
                let x = fields[2].parse::<f64>().map_err(|_| error("bad V30 x"))?;
                let y = fields[3].parse::<f64>().map_err(|_| error("bad V30 y"))?;
                let z = fields[4].parse::<f64>().map_err(|_| error("bad V30 z"))?;
                let id = molecule.add_atom(element, [x, y, z]);
                if let Some(charge) = fields
                    .iter()
                    .skip(5)
                    .find_map(|field| field.strip_prefix("CHG="))
                    .and_then(|value| value.parse::<i8>().ok())
                    && let Some(atom) = molecule.atom_mut(id)
                {
                    atom.charge = charge;
                }
                ids.push(id);
            }
            "bond" if fields.len() >= 4 => {
                let order = match fields[1].parse::<u8>().unwrap_or(1) {
                    2 => BondOrder::Double,
                    3 => BondOrder::Triple,
                    4 => BondOrder::Aromatic,
                    _ => BondOrder::Single,
                };
                // A malformed or zero atom number skips the bond; it used to bond atom 1.
                let atom = |field: &str| {
                    field
                        .parse::<usize>()
                        .ok()
                        .and_then(|number| number.checked_sub(1))
                        .and_then(|index| ids.get(index))
                };
                if let (Some(a), Some(b)) = (atom(fields[2]), atom(fields[3])) {
                    let _ = molecule.add_bond(*a, *b, order);
                }
            }
            _ => {}
        }
    }
    if molecule.atoms.is_empty() {
        return Err(error("no V30 atoms"));
    }
    Ok(molecule)
}

/// Write an SDF with one molecule per record.
pub fn to_sdf(documents: &[crate::document::ChemDocument]) -> String {
    let mut output = String::new();
    for document in documents {
        output.push_str(&to_v2000(&document.molecule, &document.name));
        output.push_str("$$$$\n");
    }
    output
}

/// Read every record of an SDF.
pub fn from_sdf(text: &str) -> Vec<Result<Molecule2D, MolfileError>> {
    text.split("$$$$")
        .filter(|record| !record.trim().is_empty())
        .map(from_molfile)
        .collect()
}

/// Write a simple RXN with one reactant and one product.
pub fn to_rxn(reactants: &[Molecule2D], products: &[Molecule2D]) -> String {
    let mut output = format!("$RXN\n  egui-workbench\n\n  {}  {}\n", reactants.len(), products.len());
    for molecule in reactants.iter().chain(products) {
        output.push_str("$MOL\n");
        output.push_str(&to_v2000(molecule, "mol"));
    }
    output
}

/// Read a simple RXN into reactants and products.
///
/// # Errors
/// When a `$MOL` block is malformed.
pub fn from_rxn(text: &str) -> Result<(Vec<Molecule2D>, Vec<Molecule2D>), MolfileError> {
    let counts = text
        .lines()
        .find(|line| line.split_whitespace().count() == 2)
        .ok_or_else(|| error("missing RXN counts"))?;
    let numbers = counts
        .split_whitespace()
        .filter_map(|part| part.parse::<usize>().ok())
        .collect::<Vec<_>>();
    let mut blocks = text.split("$MOL").skip(1).map(from_molfile);
    let (Some(reactants), Some(products)) = (numbers.first(), numbers.get(1)) else {
        return Err(error("invalid RXN counts"));
    };
    let mut reactant_molecules = Vec::new();
    for _ in 0..*reactants {
        reactant_molecules.push(blocks.next().ok_or_else(|| error("missing reactant"))??);
    }
    let mut product_molecules = Vec::new();
    for _ in 0..*products {
        product_molecules.push(blocks.next().ok_or_else(|| error("missing product"))??);
    }
    Ok((reactant_molecules, product_molecules))
}

fn error(message: &str) -> MolfileError {
    MolfileError {
        message: message.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn water() -> Molecule2D {
        let mut molecule = Molecule2D::new();
        let o = molecule.add_atom(Element::from_symbol("O").expect("O"), [0.0, 0.0, 0.0]);
        let h1 = molecule.add_atom(Element::from_symbol("H").expect("H"), [0.9, 0.0, 0.0]);
        let h2 = molecule.add_atom(Element::from_symbol("H").expect("H"), [-0.9, 0.0, 0.0]);
        molecule.add_bond(o, h1, BondOrder::Single).expect("bond");
        molecule.add_bond(o, h2, BondOrder::Single).expect("bond");
        molecule
    }

    #[test]
    fn v2000_round_trips() {
        let source = water();
        let text = to_v2000(&source, "water");
        let molecule = from_molfile(&text).expect("parse");
        assert_eq!(molecule.atoms.len(), 3);
        assert_eq!(molecule.bonds.len(), 2);
    }

    #[test]
    fn v3000_round_trips() {
        let source = water();
        let text = to_v3000(&source, "water");
        let molecule = from_molfile(&text).expect("parse");
        assert_eq!(molecule.atoms.len(), 3);
        assert_eq!(molecule.bonds.len(), 2);
    }

    #[test]
    fn sdf_round_trips_two_records() {
        let document = crate::document::ChemDocument {
            id: egui_workbench_chem_core::DocumentId::new(1),
            name: "water".to_owned(),
            molecule: water(),
            superatoms: Vec::new(),
            annotations: Vec::new(),
        };
        let text = to_sdf(&[document.clone(), document]);
        let records = from_sdf(&text);
        assert_eq!(records.len(), 2);
        assert!(
            records
                .iter()
                .all(|record| record.as_ref().is_ok_and(|molecule| molecule.atoms.len() == 3))
        );
    }

    #[test]
    fn rxn_round_trips() {
        let text = to_rxn(&[water()], &[water()]);
        let (reactants, products) = from_rxn(&text).expect("parse");
        assert_eq!(reactants.len(), 1);
        assert_eq!(products.len(), 1);
    }
}
