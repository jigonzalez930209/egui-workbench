//! Atom color schemes for the 3D viewport.

use crate::representation::vdw_radius;
use crate::scene::Atom3D;

/// How atoms are colored.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ColorScheme {
    /// The CPK/Jmol palette from the shared periodic table.
    #[default]
    Cpk,
    /// A restrained Jmol-like palette with darker carbon.
    Jmol,
    /// One color per chain, with a stable hash.
    Chain,
    /// One color per residue name and number.
    Residue,
    /// Blue for negative and red for positive partial charge.
    PartialCharge,
    /// The blue-white-red B-factor ramp.
    Temperature,
}

impl ColorScheme {
    /// All schemes, in menu order.
    pub const ALL: [Self; 6] = [
        Self::Cpk,
        Self::Jmol,
        Self::Chain,
        Self::Residue,
        Self::PartialCharge,
        Self::Temperature,
    ];

    /// A short label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Cpk => "CPK",
            Self::Jmol => "Jmol",
            Self::Chain => "Chain",
            Self::Residue => "Residue",
            Self::PartialCharge => "Partial charge",
            Self::Temperature => "B-factor",
        }
    }

    /// The RGB color for `atom`.
    pub fn color(self, atom: &Atom3D) -> [u8; 3] {
        match self {
            Self::Cpk => atom.element.cpk_color(),
            Self::Jmol => jmol_color(atom.element.symbol(), atom.element.cpk_color()),
            Self::Chain => {
                let chain = atom
                    .residue
                    .as_ref()
                    .map(|residue| residue.chain.as_str())
                    .unwrap_or_default();
                ramp(hash_string(chain) % 360)
            }
            Self::Residue => {
                let residue = atom.residue.as_ref();
                let key = residue.map_or_else(String::new, |residue| format!("{}:{}", residue.name, residue.number));
                ramp(hash_string(&key) % 360)
            }
            Self::PartialCharge => charge_color(atom.charge),
            Self::Temperature => temperature_color(atom.b_factor),
        }
    }

    /// The radius used by space-filling views.
    pub fn radius(self, atom: &Atom3D) -> f64 {
        vdw_radius(atom.element)
    }
}

fn jmol_color(symbol: &str, fallback: [u8; 3]) -> [u8; 3] {
    match symbol {
        "C" => [144, 144, 144],
        "H" => [255, 255, 255],
        "N" => [48, 80, 255],
        "O" => [255, 13, 13],
        "S" => [255, 200, 50],
        "P" => [255, 128, 0],
        "F" => [144, 224, 80],
        "Cl" => [31, 240, 31],
        "Br" => [166, 41, 41],
        "I" => [148, 0, 148],
        "Fe" => [224, 102, 51],
        "Zn" => [125, 128, 176],
        "Mg" => [138, 255, 0],
        "Ca" => [61, 255, 0],
        _ => fallback,
    }
}

fn charge_color(charge: f64) -> [u8; 3] {
    let magnitude = (charge.abs() / 0.5).clamp(0.0, 1.0) as f32;
    let base = [180.0, 180.0, 180.0];
    let target = if charge < 0.0 {
        [40.0, 80.0, 255.0]
    } else {
        [255.0, 50.0, 40.0]
    };
    mix(base, target, magnitude)
}

fn temperature_color(b_factor: f64) -> [u8; 3] {
    let normalized = ((b_factor - 10.0) / 60.0).clamp(0.0, 1.0) as f32;
    if normalized < 0.5 {
        mix([40.0, 70.0, 220.0], [240.0, 240.0, 240.0], normalized * 2.0)
    } else {
        mix([240.0, 240.0, 240.0], [220.0, 40.0, 30.0], (normalized - 0.5) * 2.0)
    }
}

fn ramp(hue: u32) -> [u8; 3] {
    let h = (hue % 360) as f32 / 60.0;
    let x = 1.0 - (h % 2.0 - 1.0).abs();
    let (r, g, b) = match h as u32 {
        0 => (1.0, x, 0.0),
        1 => (x, 1.0, 0.0),
        2 => (0.0, 1.0, x),
        3 => (0.0, x, 1.0),
        4 => (x, 0.0, 1.0),
        _ => (1.0, 0.0, x),
    };
    [
        (70.0 + r * 150.0) as u8,
        (70.0 + g * 150.0) as u8,
        (70.0 + b * 150.0) as u8,
    ]
}

fn mix(from: [f32; 3], to: [f32; 3], t: f32) -> [u8; 3] {
    [0, 1, 2].map(|index| (from[index] + (to[index] - from[index]) * t) as u8)
}

fn hash_string(text: &str) -> u32 {
    text.bytes()
        .fold(2166136261, |hash, byte| (hash ^ u32::from(byte)).wrapping_mul(16777619))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::Vec3;
    use egui_workbench_chem_core::{AtomId, Element};

    fn atom(element: &str) -> Atom3D {
        Atom3D::new(
            AtomId::new(1, 1),
            Element::from_symbol(element).expect("element"),
            Vec3::ZERO,
            0.4,
        )
    }

    #[test]
    fn cpk_and_jmol_differ_for_sulfur() {
        let sulfur = atom("S");
        assert_ne!(ColorScheme::Cpk.color(&sulfur), ColorScheme::Jmol.color(&sulfur));
    }

    #[test]
    fn charge_and_temperature_ramps_are_in_range() {
        let mut carbon = atom("C");
        carbon.charge = -0.4;
        let blue = ColorScheme::PartialCharge.color(&carbon);
        carbon.charge = 0.4;
        let red = ColorScheme::PartialCharge.color(&carbon);
        assert!(blue[2] > blue[0]);
        assert!(red[0] > red[2]);
        carbon.b_factor = 80.0;
        let hot = ColorScheme::Temperature.color(&carbon);
        carbon.b_factor = 5.0;
        let cold = ColorScheme::Temperature.color(&carbon);
        assert!(hot[0] > cold[0]);
    }

    #[test]
    fn chain_colors_are_stable() {
        let mut carbon = atom("C");
        carbon.residue = Some(crate::scene::Residue3D {
            name: "ALA".to_owned(),
            chain: "A".to_owned(),
            number: 1,
        });
        assert_eq!(ColorScheme::Chain.color(&carbon), ColorScheme::Chain.color(&carbon));
    }
}
