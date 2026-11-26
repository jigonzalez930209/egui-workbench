//! Molecular file readers for the 3D viewer.

use egui_workbench_chem_core::{BondId, BondOrder, Element, UnitCell};

use crate::scene::{Bond3D, Mol3dScene};

/// Why a molecular file could not be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormatError {
    /// Human-readable reason.
    pub message: String,
}

impl std::fmt::Display for FormatError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for FormatError {}

/// Read one or more XYZ frames.
///
/// # Errors
/// When a frame is truncated or an element is unknown.
pub fn parse_xyz(text: &str) -> Result<Vec<Mol3dScene>, FormatError> {
    let lines = text.lines().collect::<Vec<_>>();
    let mut frames = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        if lines[index].trim().is_empty() {
            index += 1;
            continue;
        }
        let count = lines[index]
            .trim()
            .parse::<usize>()
            .map_err(|_| error("invalid XYZ atom count"))?;
        index += 2;
        let mut scene = Mol3dScene::default();
        for _ in 0..count {
            let line = lines.get(index).ok_or_else(|| error("truncated XYZ frame"))?;
            let mut fields = line.split_whitespace();
            let symbol = fields.next().ok_or_else(|| error("missing XYZ element"))?;
            let element = Element::from_symbol(symbol).ok_or_else(|| error("unknown XYZ element"))?;
            let x = number(fields.next())?;
            let y = number(fields.next())?;
            let z = number(fields.next())?;
            scene.push_atom(element, [x, y, z]);
            index += 1;
        }
        scene.infer_bonds();
        frames.push(scene);
    }
    if frames.is_empty() {
        return Err(error("empty XYZ file"));
    }
    Ok(frames)
}

/// Read a PDB file, using `CONECT` records when present and inferred bonds otherwise.
///
/// # Errors
/// When an atom line is malformed.
pub fn parse_pdb(text: &str) -> Result<Mol3dScene, FormatError> {
    let mut scene = Mol3dScene::default();
    let mut serials = std::collections::BTreeMap::<u32, egui_workbench_chem_core::AtomId>::new();
    for line in text.lines() {
        if line.starts_with("ATOM  ") || line.starts_with("HETATM") {
            let serial = line.get(6..11).and_then(|part| part.trim().parse::<u32>().ok());
            let x = line.get(30..38).and_then(|part| part.trim().parse::<f64>().ok());
            let y = line.get(38..46).and_then(|part| part.trim().parse::<f64>().ok());
            let z = line.get(46..54).and_then(|part| part.trim().parse::<f64>().ok());
            let element = line
                .get(76..78)
                .map(str::trim)
                .filter(|symbol| !symbol.is_empty())
                .and_then(Element::from_symbol)
                .or_else(|| {
                    line.get(12..16)
                        .map(str::trim)
                        .and_then(|name| name.chars().next())
                        .and_then(|first| Element::from_symbol(&first.to_string()))
                });
            if let (Some(serial), Some(x), Some(y), Some(z), Some(element)) = (serial, x, y, z, element) {
                let id = scene.push_atom(element, [x, y, z]);
                // The atom just pushed is the last one; scanning from the front made PDB
                // loading quadratic.
                if let Some(atom) = scene.atoms.iter_mut().rev().find(|atom| atom.id == id) {
                    atom.label = line
                        .get(12..16)
                        .map(str::trim)
                        .filter(|name| !name.is_empty())
                        .map(str::to_owned);
                    atom.residue = Some(crate::scene::Residue3D {
                        name: line.get(17..20).map(str::trim).unwrap_or_default().to_owned(),
                        chain: line.get(21..22).map(str::trim).unwrap_or_default().to_owned(),
                        number: line
                            .get(22..26)
                            .and_then(|part| part.trim().parse::<i32>().ok())
                            .unwrap_or(0),
                    });
                    atom.b_factor = line
                        .get(60..66)
                        .and_then(|part| part.trim().parse::<f64>().ok())
                        .unwrap_or(0.0);
                    atom.occupancy = line
                        .get(54..60)
                        .and_then(|part| part.trim().parse::<f64>().ok())
                        .unwrap_or(1.0);
                }
                serials.insert(serial, id);
            }
        } else if let Some(rest) = line.strip_prefix("CONECT") {
            let numbers = rest
                .split_whitespace()
                .filter_map(|part| part.parse::<u32>().ok())
                .collect::<Vec<_>>();
            let Some(from) = numbers.first().and_then(|serial| serials.get(serial)) else {
                continue;
            };
            for serial in numbers.iter().skip(1) {
                let Some(to) = serials.get(serial) else {
                    continue;
                };
                if scene
                    .bonds
                    .iter()
                    .any(|bond| (bond.a.min(bond.b), bond.a.max(bond.b)) == (*from, *to))
                {
                    continue;
                }
                scene.bonds.push(Bond3D {
                    id: BondId::new(scene.bonds.len() as u32 + 1, 1),
                    a: *from,
                    b: *to,
                    order: BondOrder::Single,
                });
            }
        }
    }
    if scene.atoms.is_empty() {
        return Err(error("no PDB atoms"));
    }
    if scene.bonds.is_empty() {
        scene.infer_bonds();
    }
    Ok(scene)
}

/// Read the `_atom_site` loop of an mmCIF file.
///
/// # Errors
/// When the atom-site headers are missing.
pub fn parse_mmcif(text: &str) -> Result<Mol3dScene, FormatError> {
    let lines = text.lines().map(str::trim).collect::<Vec<_>>();
    // Real files open with other loops; use the one whose first header is `_atom_site.`.
    let Some((headers, mut cursor)) =
        lines
            .iter()
            .enumerate()
            .filter(|(_, line)| **line == "loop_")
            .find_map(|(index, _)| {
                let mut headers = Vec::new();
                let mut cursor = index + 1;
                while let Some(line) = lines.get(cursor)
                    && line.starts_with('_')
                {
                    headers.push((*line).to_owned());
                    cursor += 1;
                }
                headers
                    .first()
                    .is_some_and(|header| header.starts_with("_atom_site."))
                    .then_some((headers, cursor))
            })
    else {
        return Err(error("no atom_site loop"));
    };
    let column = |name: &str| headers.iter().position(|header| *header == name);
    let x_column = column("_atom_site.Cartn_x");
    let y_column = column("_atom_site.Cartn_y");
    let z_column = column("_atom_site.Cartn_z");
    let element_column = column("_atom_site.type_symbol");
    let (Some(x_column), Some(y_column), Some(z_column), Some(element_column)) =
        (x_column, y_column, z_column, element_column)
    else {
        return Err(error("missing atom_site Cartesian columns"));
    };
    let mut scene = Mol3dScene::default();
    while let Some(line) = lines.get(cursor) {
        if line.starts_with('#') || line.starts_with("loop_") || line.starts_with('_') {
            break;
        }
        let fields = line.split_whitespace().collect::<Vec<_>>();
        // Every column that is read must exist; checking only the element column let a
        // short row index past the end.
        if fields.len() <= element_column.max(x_column).max(y_column).max(z_column) {
            cursor += 1;
            continue;
        }
        let element = Element::from_symbol(fields[element_column]).ok_or_else(|| error("unknown mmCIF element"))?;
        let x = fields[x_column].parse::<f64>().map_err(|_| error("bad mmCIF x"))?;
        let y = fields[y_column].parse::<f64>().map_err(|_| error("bad mmCIF y"))?;
        let z = fields[z_column].parse::<f64>().map_err(|_| error("bad mmCIF z"))?;
        scene.push_atom(element, [x, y, z]);
        cursor += 1;
    }
    if scene.atoms.is_empty() {
        return Err(error("no mmCIF atoms"));
    }
    scene.infer_bonds();
    Ok(scene)
}

/// A fixed-width column, or `None` when it is out of range or splits a character.
fn column(line: &str, from: usize, to: usize) -> Option<&str> {
    line.get(from..to).map(str::trim)
}

/// Read a V2000 molfile or the first SDF record.
///
/// # Errors
/// When the counts or atom lines are malformed.
pub fn parse_mol(text: &str) -> Result<Mol3dScene, FormatError> {
    let lines = text.lines().collect::<Vec<_>>();
    let counts = lines
        .iter()
        .position(|line| line.contains("V2000"))
        .ok_or_else(|| error("missing V2000 counts"))?;
    let count_line = lines[counts];
    let atom_count = count_line
        .get(0..3)
        .and_then(|part| part.trim().parse::<usize>().ok())
        .ok_or_else(|| error("bad mol atom count"))?;
    let bond_count = count_line
        .get(3..6)
        .and_then(|part| part.trim().parse::<usize>().ok())
        .ok_or_else(|| error("bad mol bond count"))?;
    let mut scene = Mol3dScene::default();
    let mut ids = Vec::new();
    // `str::get`, not `[a..b]`: a multi-byte character inside a column would panic.
    for line in lines.iter().skip(counts + 1).take(atom_count) {
        let coordinate = |from: usize, to: usize, name: &str| {
            column(line, from, to)
                .and_then(|part| part.parse::<f64>().ok())
                .filter(|value| value.is_finite())
                .ok_or_else(|| error(name))
        };
        let x = coordinate(0, 10, "bad mol x")?;
        let y = coordinate(10, 20, "bad mol y")?;
        let z = coordinate(20, 30, "bad mol z")?;
        let symbol = column(line, 31, 34).ok_or_else(|| error("short mol atom line"))?;
        let element = Element::from_symbol(symbol).ok_or_else(|| error("unknown mol element"))?;
        ids.push(scene.push_atom(element, [x, y, z]));
    }
    if ids.len() < atom_count {
        return Err(error("truncated mol atom block"));
    }
    for line in lines.iter().skip(counts + 1 + atom_count).take(bond_count) {
        let atom = |from: usize, to: usize| {
            column(line, from, to)
                .and_then(|part| part.parse::<usize>().ok())
                .and_then(|number| number.checked_sub(1))
                .and_then(|index| ids.get(index))
        };
        let order = match column(line, 6, 9).and_then(|part| part.parse::<u8>().ok()).unwrap_or(1) {
            2 => BondOrder::Double,
            3 => BondOrder::Triple,
            4 => BondOrder::Aromatic,
            _ => BondOrder::Single,
        };
        // A bad or zero atom number skips the bond; it used to bond atom 1.
        if let (Some(a), Some(b)) = (atom(0, 3), atom(3, 6)) {
            scene.bonds.push(Bond3D {
                id: BondId::new(scene.bonds.len() as u32 + 1, 1),
                a: *a,
                b: *b,
                order,
            });
        }
    }
    Ok(scene)
}

/// Read a VASP POSCAR/CONTCAR.
///
/// # Errors
/// When the lattice, species, or coordinate block is malformed.
pub fn parse_poscar(text: &str) -> Result<Mol3dScene, FormatError> {
    let lines = text.lines().collect::<Vec<_>>();
    if lines.len() < 8 {
        return Err(error("short POSCAR"));
    }
    let scale = lines[1].trim().parse::<f64>().unwrap_or(1.0);
    let mut vectors = [[0.0; 3]; 3];
    for (index, vector) in vectors.iter_mut().enumerate() {
        let fields = lines[2 + index].split_whitespace().collect::<Vec<_>>();
        for (axis, field) in fields.iter().take(3).enumerate() {
            vector[axis] = field.parse::<f64>().map_err(|_| error("bad POSCAR lattice"))? * scale;
        }
    }
    let cell = cell_from_vectors(vectors);
    let species = lines[5].split_whitespace().collect::<Vec<_>>();
    let counts = lines[6]
        .split_whitespace()
        .filter_map(|part| part.parse::<usize>().ok())
        .collect::<Vec<_>>();
    let mut cursor = 7;
    let direct = lines[cursor].trim().to_ascii_lowercase().starts_with('d');
    if !direct {
        cursor += 1;
    }
    cursor += 1;
    let mut scene = Mol3dScene::default().with_cell(cell);
    let mut atom_index = 0;
    for (species_index, count) in counts.iter().enumerate() {
        let element = species
            .get(species_index)
            .and_then(|symbol| Element::from_symbol(symbol))
            .ok_or_else(|| error("unknown POSCAR species"))?;
        for _ in 0..*count {
            let fields = lines
                .get(cursor)
                .ok_or_else(|| error("truncated POSCAR"))?
                .split_whitespace()
                .collect::<Vec<_>>();
            let x = fields
                .first()
                .and_then(|value| value.parse::<f64>().ok())
                .unwrap_or(0.0);
            let y = fields.get(1).and_then(|value| value.parse::<f64>().ok()).unwrap_or(0.0);
            let z = fields.get(2).and_then(|value| value.parse::<f64>().ok()).unwrap_or(0.0);
            let position = if direct {
                [0, 1, 2].map(|axis| x * vectors[0][axis] + y * vectors[1][axis] + z * vectors[2][axis])
            } else {
                [x * scale, y * scale, z * scale]
            };
            scene.push_atom(element, position);
            cursor += 1;
            atom_index += 1;
        }
    }
    if atom_index == 0 {
        return Err(error("no POSCAR atoms"));
    }
    scene.infer_bonds();
    Ok(scene)
}

fn cell_from_vectors(vectors: [[f64; 3]; 3]) -> UnitCell {
    let length = |vector: [f64; 3]| (vector[0] * vector[0] + vector[1] * vector[1] + vector[2] * vector[2]).sqrt();
    let angle = |a: [f64; 3], b: [f64; 3]| {
        let dot = a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
        (dot / (length(a) * length(b)).max(1.0e-9))
            .clamp(-1.0, 1.0)
            .acos()
            .to_degrees()
    };
    UnitCell::new(
        length(vectors[0]),
        length(vectors[1]),
        length(vectors[2]),
        angle(vectors[1], vectors[2]),
        angle(vectors[0], vectors[2]),
        angle(vectors[0], vectors[1]),
    )
    .unwrap_or(UnitCell {
        a: 1.0,
        b: 1.0,
        c: 1.0,
        alpha: 90.0,
        beta: 90.0,
        gamma: 90.0,
    })
}

fn number(field: Option<&str>) -> Result<f64, FormatError> {
    field
        .ok_or_else(|| error("missing coordinate"))?
        .parse::<f64>()
        .map_err(|_| error("bad coordinate"))
}

fn error(message: &str) -> FormatError {
    FormatError {
        message: message.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xyz_water_parses_and_infers_bonds() {
        let text = "3\nwater\nO 0 0 0\nH 0.96 0 0\nH -0.24 0.93 0\n";
        let frames = parse_xyz(text).expect("xyz");
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].atoms.len(), 3);
        assert!(!frames[0].bonds.is_empty());
    }

    #[test]
    fn pdb_uses_element_and_conect() {
        let text = "ATOM      1  O   HOH A   1       0.000   0.000   0.000  1.00  0.00           O\nATOM      2  H   HOH A   1       0.960   0.000   0.000  1.00  0.00           H\nCONECT    1    2\n";
        let scene = parse_pdb(text).expect("pdb");
        assert_eq!(scene.atoms.len(), 2);
        assert_eq!(scene.bonds.len(), 1);
    }

    #[test]
    fn mmcif_atom_site_parses() {
        let text = "loop_\n_atom_site.Cartn_x\n_atom_site.Cartn_y\n_atom_site.Cartn_z\n_atom_site.type_symbol\n0 0 0 O\n1 0 0 H\n#\n";
        let scene = parse_mmcif(text).expect("mmcif");
        assert_eq!(scene.atoms.len(), 2);
    }

    #[test]
    fn poscar_parses_direct_coordinates() {
        let text = "Si\n1.0\n5 0 0\n0 5 0\n0 0 5\nSi\n2\nDirect\n0 0 0\n0.5 0.5 0.5\n";
        let scene = parse_poscar(text).expect("poscar");
        assert_eq!(scene.atoms.len(), 2);
        assert!(scene.cell.is_some());
    }

    #[test]
    fn molfile_parses() {
        let text = "water\n  test\n\n  3  2  0  0  0  0  0  0  0  0999 V2000\n    0.0000    0.0000    0.0000 O   0  0\n    0.9600    0.0000    0.0000 H   0  0\n   -0.2400    0.9300    0.0000 H   0  0\n  1  2  1  0\n  1  3  1  0\nM  END\n";
        let scene = parse_mol(text).expect("mol");
        assert_eq!(scene.atoms.len(), 3);
        assert_eq!(scene.bonds.len(), 2);
    }
}
