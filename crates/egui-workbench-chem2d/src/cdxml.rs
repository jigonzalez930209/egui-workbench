//! A compact CDXML subset for the 2D editor.

use egui_workbench_chem_core::{BondOrder, Element};

use crate::document::Molecule2D;
use crate::molfile::MolfileError;

/// Write a CDXML document with one fragment.
pub fn to_cdxml(molecule: &Molecule2D) -> String {
    let mut output =
        String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<CDXML><page id=\"1\"><fragment id=\"2\">\n");
    for (index, atom) in molecule.atoms.iter().enumerate() {
        output.push_str(&format!(
            "  <n id=\"{}\" p=\"{:.4} {:.4}\" Element=\"{}\" Charge=\"{}\"/>\n",
            index + 3,
            atom.position[0],
            atom.position[1],
            atom.element.atomic_number(),
            atom.charge
        ));
    }
    for (index, bond) in molecule.bonds.iter().enumerate() {
        let a = molecule.atoms.iter().position(|atom| atom.id == bond.a).unwrap_or(0) + 3;
        let b = molecule.atoms.iter().position(|atom| atom.id == bond.b).unwrap_or(0) + 3;
        let order = match bond.order {
            BondOrder::Double => 2,
            BondOrder::Triple => 3,
            BondOrder::Aromatic => 4,
            _ => 1,
        };
        output.push_str(&format!(
            "  <b id=\"{}\" B=\"{a}\" E=\"{b}\" Order=\"{order}\"/>\n",
            molecule.atoms.len() + index + 3
        ));
    }
    output.push_str("</fragment></page></CDXML>\n");
    output
}

/// Read the atom and bond nodes of a CDXML fragment.
///
/// # Errors
/// When an atom or bond node is malformed.
pub fn from_cdxml(text: &str) -> Result<Molecule2D, MolfileError> {
    let mut molecule = Molecule2D::new();
    let mut ids = Vec::new();
    let mut by_xml = Vec::<(u32, usize)>::new();
    for tag in tags(text, "n") {
        let Some(id) = attribute(&tag, "id").and_then(|value| value.parse::<u32>().ok()) else {
            return Err(error("CDXML atom without id"));
        };
        let Some(element) = attribute(&tag, "Element")
            .and_then(|value| value.parse::<u8>().ok())
            .and_then(Element::new)
        else {
            return Err(error("CDXML atom without a valid element"));
        };
        let point = attribute(&tag, "p");
        let mut parts = point
            .as_deref()
            .unwrap_or("0 0")
            .split_whitespace()
            .filter_map(|part| part.parse::<f64>().ok());
        let x = parts.next().unwrap_or(0.0);
        let y = parts.next().unwrap_or(0.0);
        let atom = molecule.add_atom(element, [x, y, 0.0]);
        if let Some(charge) = attribute(&tag, "Charge").and_then(|value| value.parse::<i8>().ok())
            && let Some(entry) = molecule.atom_mut(atom)
        {
            entry.charge = charge;
        }
        ids.push(atom);
        by_xml.push((id, ids.len() - 1));
    }
    for tag in tags(text, "b") {
        let a = attribute(&tag, "B").and_then(|value| value.parse::<u32>().ok());
        let b = attribute(&tag, "E").and_then(|value| value.parse::<u32>().ok());
        let order = match attribute(&tag, "Order")
            .and_then(|value| value.parse::<u8>().ok())
            .unwrap_or(1)
        {
            2 => BondOrder::Double,
            3 => BondOrder::Triple,
            4 => BondOrder::Aromatic,
            _ => BondOrder::Single,
        };
        let (Some(a), Some(b)) = (a, b) else {
            continue;
        };
        let a = by_xml.iter().find(|(id, _)| *id == a).map(|(_, index)| *index);
        let b = by_xml.iter().find(|(id, _)| *id == b).map(|(_, index)| *index);
        if let (Some(a), Some(b)) = (a, b)
            && let (Some(a), Some(b)) = (ids.get(a), ids.get(b))
        {
            let _ = molecule.add_bond(*a, *b, order);
        }
    }
    if molecule.atoms.is_empty() {
        return Err(error("no CDXML atoms"));
    }
    Ok(molecule)
}

fn tags(text: &str, name: &str) -> Vec<String> {
    let opening = format!("<{name} ");
    let mut result = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find(&opening) {
        let tail = &rest[start..];
        let Some(end) = tail.find('>') else {
            break;
        };
        result.push(tail[..=end].to_owned());
        rest = &tail[end + 1..];
    }
    result
}

fn attribute(tag: &str, name: &str) -> Option<String> {
    let key = format!("{name}=\"");
    let start = tag.find(&key)? + key.len();
    let end = tag[start..].find('"')? + start;
    Some(tag[start..end].to_owned())
}

fn error(message: &str) -> MolfileError {
    MolfileError {
        message: message.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cdxml_round_trips_atoms_and_bonds() {
        let mut molecule = Molecule2D::new();
        let c = molecule.add_atom(Element::from_symbol("C").expect("C"), [0.0, 0.0, 0.0]);
        let o = molecule.add_atom(Element::from_symbol("O").expect("O"), [1.2, 0.0, 0.0]);
        molecule.add_bond(c, o, BondOrder::Double).expect("bond");
        let text = to_cdxml(&molecule);
        assert!(text.contains("Element=\"6\""));
        let restored = from_cdxml(&text).expect("parse");
        assert_eq!(restored.atoms.len(), 2);
        assert_eq!(restored.bonds.len(), 1);
        assert_eq!(restored.bonds[0].order, BondOrder::Double);
    }
}
