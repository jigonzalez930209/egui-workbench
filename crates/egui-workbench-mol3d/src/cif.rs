//! Crystallographic CIF reading with space-group expansion.

use egui_workbench_chem_core::{Element, UnitCell};

use crate::formats::FormatError;
use crate::scene::Mol3dScene;

/// Read a CIF, expanding the asymmetric unit through its symmetry operations.
///
/// Supports the `_atom_site` loop with fractional or Cartesian coordinates and
/// `_symmetry_equiv_pos_as_xyz` / `_space_group_symop_operation_xyz` operators.
///
/// # Errors
/// When the cell or the atom-site loop is missing or malformed.
pub fn parse_cif(text: &str) -> Result<Mol3dScene, FormatError> {
    let cell = read_cell(text)?;
    let operations = read_operations(text);
    let rows = atom_rows(text)?;
    let mut scene = Mol3dScene::default().with_cell(cell);
    for row in rows {
        let base = row.position;
        let generated = if operations.is_empty() {
            vec![base]
        } else {
            operations
                .iter()
                .map(|operation| apply_operation(operation, base))
                .collect()
        };
        for fractional in generated {
            let fractional = fractional.map(|value| value.rem_euclid(1.0));
            let cartesian = cell.fractional_to_cartesian(fractional);
            let duplicate = scene.atoms.iter().any(|atom| {
                (atom.position.x - cartesian[0]).abs() < 1.0e-3
                    && (atom.position.y - cartesian[1]).abs() < 1.0e-3
                    && (atom.position.z - cartesian[2]).abs() < 1.0e-3
                    && atom.element == row.element
            });
            if duplicate {
                continue;
            }
            let id = scene.push_atom(row.element, cartesian);
            if let Some(atom) = scene.atoms.iter_mut().rev().find(|atom| atom.id == id) {
                atom.label = Some(row.label.clone());
                atom.occupancy = row.occupancy;
            }
        }
    }
    if scene.atoms.is_empty() {
        return Err(error("no CIF atoms"));
    }
    scene.infer_bonds();
    Ok(scene)
}

/// A parsed symmetry operation `x' = a x + b y + c z + d`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SymmetryOperation {
    /// Linear coefficients for x, y, and z.
    pub coefficients: [[f64; 3]; 3],
    /// Translation for each output axis.
    pub translation: [f64; 3],
}

impl SymmetryOperation {
    /// Apply the operation to fractional coordinates.
    pub fn apply(&self, point: [f64; 3]) -> [f64; 3] {
        [0, 1, 2].map(|axis| {
            self.coefficients[axis][0] * point[0]
                + self.coefficients[axis][1] * point[1]
                + self.coefficients[axis][2] * point[2]
                + self.translation[axis]
        })
    }
}

/// Parse `x,y,z`-style symmetry expressions.
///
/// # Errors
/// When an expression has the wrong number of components or an unknown token.
pub fn parse_operation(expression: &str) -> Result<SymmetryOperation, FormatError> {
    let parts = expression.split(',').collect::<Vec<_>>();
    if parts.len() != 3 {
        return Err(error("symmetry operation needs three components"));
    }
    let mut operation = SymmetryOperation {
        coefficients: [[0.0; 3]; 3],
        translation: [0.0; 3],
    };
    for (axis, part) in parts.iter().enumerate() {
        parse_component(
            part,
            &mut operation.coefficients[axis],
            &mut operation.translation[axis],
        )?;
    }
    Ok(operation)
}

fn parse_component(text: &str, coefficients: &mut [f64; 3], translation: &mut f64) -> Result<(), FormatError> {
    let normalized = text.trim().replace(' ', "").replace('-', "+-");
    for term in normalized.split('+') {
        if term.is_empty() {
            continue;
        }
        let sign = if let Some(rest) = term.strip_prefix('-') {
            if rest.is_empty() {
                continue;
            }
            -1.0
        } else {
            1.0
        };
        let body = term.trim_start_matches('-');
        let variable = body
            .chars()
            .rev()
            .find(|character| matches!(character.to_ascii_lowercase(), 'x' | 'y' | 'z'));
        match variable {
            Some(character) => {
                let axis = match character.to_ascii_lowercase() {
                    'x' => 0,
                    'y' => 1,
                    _ => 2,
                };
                let factor = body.trim_end_matches(|value: char| value.is_ascii_alphabetic());
                let factor = if factor.is_empty() { 1.0 } else { parse_number(factor)? };
                coefficients[axis] += sign * factor;
            }
            None => *translation += sign * parse_number(body)?,
        }
    }
    Ok(())
}

fn parse_number(text: &str) -> Result<f64, FormatError> {
    if let Some((numerator, denominator)) = text.split_once('/') {
        let numerator = numerator.parse::<f64>().map_err(|_| error("bad fraction"))?;
        let denominator = denominator.parse::<f64>().map_err(|_| error("bad fraction"))?;
        if denominator == 0.0 || !numerator.is_finite() || !denominator.is_finite() {
            return Err(error("bad fraction"));
        }
        Ok(numerator / denominator)
    } else {
        text.parse::<f64>().map_err(|_| error("bad symmetry number"))
    }
}

fn apply_operation(operation: &SymmetryOperation, point: [f64; 3]) -> [f64; 3] {
    operation.apply(point)
}

fn read_cell(text: &str) -> Result<UnitCell, FormatError> {
    let value = |tag: &str| -> Option<f64> {
        text.lines()
            .find(|line| line.trim_start().starts_with(tag))
            .and_then(|line| line.split_whitespace().nth(1))
            .map(strip_uncertainty)
            .and_then(|value| value.parse::<f64>().ok())
    };
    let a = value("_cell_length_a").ok_or_else(|| error("missing CIF cell length a"))?;
    let b = value("_cell_length_b").ok_or_else(|| error("missing CIF cell length b"))?;
    let c = value("_cell_length_c").ok_or_else(|| error("missing CIF cell length c"))?;
    let alpha = value("_cell_angle_alpha").unwrap_or(90.0);
    let beta = value("_cell_angle_beta").unwrap_or(90.0);
    let gamma = value("_cell_angle_gamma").unwrap_or(90.0);
    UnitCell::new(a, b, c, alpha, beta, gamma).map_err(|error| FormatError {
        message: format!("invalid CIF cell: {error}"),
    })
}

fn strip_uncertainty(value: &str) -> &str {
    value.split('(').next().unwrap_or(value)
}

fn read_operations(text: &str) -> Vec<SymmetryOperation> {
    let mut operations = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        let expression = if trimmed.starts_with('_') {
            let mut fields = trimmed.split_whitespace();
            let tag = fields.next().unwrap_or_default();
            if tag == "_symmetry_equiv_pos_as_xyz"
                || tag == "_space_group_symop_operation_xyz"
                || tag == "_symmetry_equiv_pos_site_id"
            {
                let remainder = fields.collect::<Vec<_>>().join(" ");
                if tag == "_symmetry_equiv_pos_site_id" {
                    None
                } else {
                    Some(remainder)
                }
            } else {
                None
            }
        } else {
            let candidate = trimmed.trim_matches(|character| character == '\'' || character == '"');
            if candidate.contains(',')
                && candidate
                    .chars()
                    .all(|character| "xyzXYZ+-*/,0123456789. ".contains(character))
            {
                Some(candidate.to_owned())
            } else {
                None
            }
        };
        if let Some(expression) = expression {
            let expression = expression.trim_matches(|character| character == '\'' || character == '"');
            if let Ok(operation) = parse_operation(expression)
                && !operations.contains(&operation)
            {
                operations.push(operation);
            }
        }
    }
    operations
}

struct AtomRow {
    label: String,
    element: Element,
    position: [f64; 3],
    occupancy: f64,
}

fn atom_rows(text: &str) -> Result<Vec<AtomRow>, FormatError> {
    let lines = text.lines().map(str::trim).collect::<Vec<_>>();
    for (start, line) in lines.iter().enumerate() {
        if *line != "loop_" {
            continue;
        }
        let mut headers = Vec::new();
        let mut cursor = start + 1;
        while let Some(header) = lines.get(cursor)
            && header.starts_with('_')
        {
            headers.push((*header).to_owned());
            cursor += 1;
        }
        let index = |name: &str| headers.iter().position(|header| header == name);
        let Some(label_column) = index("_atom_site_label") else {
            continue;
        };
        let fract = (
            index("_atom_site_fract_x"),
            index("_atom_site_fract_y"),
            index("_atom_site_fract_z"),
        );
        let cartn = (
            index("_atom_site.Cartn_x"),
            index("_atom_site.Cartn_y"),
            index("_atom_site.Cartn_z"),
        );
        // All three axes of one coordinate system, not just the first: a loop with only
        // `_fract_x` used to reach an `expect` below.
        let has_fract = matches!(fract, (Some(_), Some(_), Some(_)));
        let has_cartn = matches!(cartn, (Some(_), Some(_), Some(_)));
        if !has_fract && !has_cartn {
            continue;
        }
        let element_column = index("_atom_site_type_symbol");
        let occupancy_column = index("_atom_site.occupancy");
        let mut rows = Vec::new();
        while let Some(line) = lines.get(cursor) {
            if line.is_empty() || *line == "#" || line.starts_with("loop_") || line.starts_with('_') {
                break;
            }
            let fields = line.split_whitespace().collect::<Vec<_>>();
            if fields.len() < headers.len() {
                cursor += 1;
                continue;
            }
            let label = fields[label_column].to_owned();
            let element = element_column
                .and_then(|column| fields.get(column))
                .and_then(|symbol| Element::from_symbol(symbol))
                .or_else(|| Element::from_symbol(&label.chars().take(2).collect::<String>()))
                .or_else(|| label.chars().next().and_then(|c| Element::from_symbol(&c.to_string())))
                .ok_or_else(|| error("unknown CIF element"))?;
            // Coordinates carry a standard uncertainty, as in `0.1234(5)`.
            let coordinate = |column: usize, name: &str| {
                strip_uncertainty(fields[column])
                    .parse::<f64>()
                    .ok()
                    .filter(|value| value.is_finite())
                    .ok_or_else(|| error(name))
            };
            let position = if let (Some(x), Some(y), Some(z)) = fract {
                [
                    coordinate(x, "bad CIF fract x")?,
                    coordinate(y, "bad CIF fract y")?,
                    coordinate(z, "bad CIF fract z")?,
                ]
            } else if let (Some(x), Some(y), Some(z)) = cartn {
                [
                    coordinate(x, "bad CIF x")?,
                    coordinate(y, "bad CIF y")?,
                    coordinate(z, "bad CIF z")?,
                ]
            } else {
                continue;
            };
            let occupancy = occupancy_column
                .and_then(|column| fields.get(column))
                .and_then(|value| strip_uncertainty(value).parse().ok())
                .unwrap_or(1.0);
            rows.push(AtomRow {
                label,
                element,
                position,
                occupancy,
            });
            cursor += 1;
        }
        if !rows.is_empty() {
            return Ok(rows);
        }
    }
    Err(error("no CIF atom_site loop"))
}

fn error(message: &str) -> FormatError {
    FormatError {
        message: message.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CIF: &str = "_cell_length_a 10.0\n_cell_length_b 10.0\n_cell_length_c 10.0\n_cell_angle_alpha 90\n_cell_angle_beta 90\n_cell_angle_gamma 90\n_symmetry_equiv_pos_as_xyz 'x,y,z'\n_symmetry_equiv_pos_as_xyz '-x,-y,-z'\nloop_\n_atom_site_label\n_atom_site_type_symbol\n_atom_site_fract_x\n_atom_site_fract_y\n_atom_site_fract_z\n_atom_site.occupancy\nNa1 Na 0.0 0.0 0.0 1.0\nCl1 Cl 0.5 0.5 0.5 1.0\n";

    #[test]
    fn cif_expands_symmetry_and_reads_the_cell() {
        let scene = parse_cif(CIF).expect("cif");
        assert_eq!(scene.atoms.len(), 2);
        assert_eq!(scene.cell.expect("cell").a, 10.0);
    }

    #[test]
    fn a_symmetry_loop_with_quoted_expressions_parses() {
        let text = "_cell_length_a 5\n_cell_length_b 5\n_cell_length_c 5\n_cell_angle_alpha 90\n_cell_angle_beta 90\n_cell_angle_gamma 90\nloop_\n_symmetry_equiv_pos_as_xyz\n  'x, y, z'\n  '-x, -y, -z'\nloop_\n_atom_site_label\n_atom_site_type_symbol\n_atom_site_fract_x\n_atom_site_fract_y\n_atom_site_fract_z\nC1 C 0.25 0.25 0.25\n";
        let scene = parse_cif(text).expect("cif loop");
        assert_eq!(scene.atoms.len(), 2);
    }

    #[test]
    fn symmetry_expressions_parse_fractions() {
        let operation = parse_operation("-x+1/2,y+1/2,-z").expect("operation");
        let point = operation.apply([0.25, 0.25, 0.25]);
        assert_eq!(point, [0.25, 0.75, -0.25]);
    }
}
