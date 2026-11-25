//! The periodic table entries a molecular viewer needs.

use serde::{Deserialize, Serialize};

/// A chemical element, stored as its atomic number (`0` means unknown).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Element(u8);

impl Element {
    /// The unknown element (`X`).
    pub const UNKNOWN: Self = Self(0);

    /// Element by atomic number, when it is in `1..=118`.
    pub const fn new(atomic_number: u8) -> Option<Self> {
        if atomic_number >= 1 && atomic_number <= 118 {
            Some(Self(atomic_number))
        } else {
            None
        }
    }

    /// Element by symbol, case-insensitively (`"C"`, `"cl"`, `"Og"`).
    pub fn from_symbol(symbol: &str) -> Option<Self> {
        if symbol.eq_ignore_ascii_case("X") {
            return Some(Self::UNKNOWN);
        }
        ELEMENTS
            .iter()
            .position(|entry| entry.0.eq_ignore_ascii_case(symbol))
            .and_then(|index| Self::new(u8::try_from(index + 1).ok()?))
    }

    /// Atomic number (`0` for [`Self::UNKNOWN`]).
    pub const fn atomic_number(self) -> u8 {
        self.0
    }

    /// The element symbol.
    pub fn symbol(self) -> &'static str {
        ELEMENTS
            .get(usize::from(self.0.saturating_sub(1)))
            .map_or("X", |entry| entry.0)
    }

    /// Covalent radius in Angstrom.
    pub fn covalent_radius(self) -> f64 {
        ELEMENTS
            .get(usize::from(self.0.saturating_sub(1)))
            .map_or(0.7, |entry| entry.1)
    }

    /// The CPK/Jmol color as RGB.
    pub fn cpk_color(self) -> [u8; 3] {
        ELEMENTS
            .get(usize::from(self.0.saturating_sub(1)))
            .map_or([128, 128, 128], |entry| entry.2)
    }
}

impl std::fmt::Display for Element {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.symbol())
    }
}

type Entry = (&'static str, f64, [u8; 3]);

const ELEMENTS: [Entry; 118] = [
    ("H", 0.31, [255, 255, 255]),
    ("He", 0.28, [217, 255, 255]),
    ("Li", 1.28, [204, 128, 255]),
    ("Be", 0.96, [194, 255, 0]),
    ("B", 0.84, [255, 181, 181]),
    ("C", 0.76, [144, 144, 144]),
    ("N", 0.71, [48, 80, 248]),
    ("O", 0.66, [255, 13, 13]),
    ("F", 0.57, [144, 224, 80]),
    ("Ne", 0.58, [179, 227, 245]),
    ("Na", 1.66, [171, 92, 242]),
    ("Mg", 1.41, [138, 255, 0]),
    ("Al", 1.21, [191, 166, 166]),
    ("Si", 1.11, [240, 200, 160]),
    ("P", 1.07, [255, 128, 0]),
    ("S", 1.05, [255, 255, 48]),
    ("Cl", 1.02, [31, 240, 31]),
    ("Ar", 1.06, [128, 209, 227]),
    ("K", 2.03, [143, 64, 212]),
    ("Ca", 1.76, [61, 255, 0]),
    ("Sc", 1.70, [230, 230, 230]),
    ("Ti", 1.60, [191, 194, 199]),
    ("V", 1.53, [166, 166, 171]),
    ("Cr", 1.39, [138, 153, 199]),
    ("Mn", 1.39, [156, 122, 199]),
    ("Fe", 1.32, [224, 102, 51]),
    ("Co", 1.26, [240, 144, 160]),
    ("Ni", 1.24, [80, 208, 80]),
    ("Cu", 1.32, [200, 128, 51]),
    ("Zn", 1.22, [125, 128, 176]),
    ("Ga", 1.22, [194, 143, 143]),
    ("Ge", 1.20, [102, 143, 143]),
    ("As", 1.19, [189, 128, 227]),
    ("Se", 1.20, [255, 161, 0]),
    ("Br", 1.20, [166, 41, 41]),
    ("Kr", 1.16, [92, 184, 209]),
    ("Rb", 2.20, [112, 46, 176]),
    ("Sr", 1.95, [0, 255, 0]),
    ("Y", 1.90, [148, 255, 255]),
    ("Zr", 1.75, [148, 224, 224]),
    ("Nb", 1.64, [115, 194, 201]),
    ("Mo", 1.54, [84, 181, 181]),
    ("Tc", 1.47, [59, 158, 158]),
    ("Ru", 1.46, [36, 143, 143]),
    ("Rh", 1.42, [10, 125, 140]),
    ("Pd", 1.39, [0, 105, 133]),
    ("Ag", 1.45, [192, 192, 192]),
    ("Cd", 1.44, [255, 217, 143]),
    ("In", 1.42, [166, 117, 115]),
    ("Sn", 1.39, [102, 128, 128]),
    ("Sb", 1.39, [158, 99, 181]),
    ("Te", 1.38, [212, 122, 0]),
    ("I", 1.39, [148, 0, 148]),
    ("Xe", 1.40, [66, 158, 176]),
    ("Cs", 2.44, [87, 23, 143]),
    ("Ba", 2.15, [0, 201, 0]),
    ("La", 2.07, [112, 212, 255]),
    ("Ce", 2.04, [255, 255, 199]),
    ("Pr", 2.03, [217, 255, 199]),
    ("Nd", 2.01, [199, 255, 199]),
    ("Pm", 1.99, [163, 255, 199]),
    ("Sm", 1.98, [143, 255, 199]),
    ("Eu", 1.98, [97, 255, 199]),
    ("Gd", 1.96, [69, 255, 199]),
    ("Tb", 1.94, [11, 255, 199]),
    ("Dy", 1.92, [0, 255, 156]),
    ("Ho", 1.92, [0, 255, 102]),
    ("Er", 1.89, [0, 230, 46]),
    ("Tm", 1.90, [0, 212, 0]),
    ("Yb", 1.87, [0, 191, 0]),
    ("Lu", 1.87, [0, 171, 0]),
    ("Hf", 1.75, [77, 194, 255]),
    ("Ta", 1.70, [77, 166, 255]),
    ("W", 1.62, [33, 148, 214]),
    ("Re", 1.51, [38, 125, 171]),
    ("Os", 1.44, [38, 102, 150]),
    ("Ir", 1.41, [23, 84, 135]),
    ("Pt", 1.36, [208, 208, 224]),
    ("Au", 1.36, [255, 209, 35]),
    ("Hg", 1.32, [184, 184, 208]),
    ("Tl", 1.45, [166, 84, 77]),
    ("Pb", 1.46, [87, 89, 97]),
    ("Bi", 1.48, [158, 79, 181]),
    ("Po", 1.40, [171, 92, 0]),
    ("At", 1.50, [117, 79, 69]),
    ("Rn", 1.50, [66, 130, 150]),
    ("Fr", 2.60, [66, 0, 102]),
    ("Ra", 2.21, [0, 125, 0]),
    ("Ac", 2.15, [112, 171, 250]),
    ("Th", 2.06, [0, 186, 255]),
    ("Pa", 2.00, [0, 161, 255]),
    ("U", 1.96, [0, 143, 255]),
    ("Np", 1.90, [0, 128, 255]),
    ("Pu", 1.87, [0, 107, 255]),
    ("Am", 1.80, [61, 89, 255]),
    ("Cm", 1.69, [61, 61, 255]),
    ("Bk", 1.60, [61, 36, 255]),
    ("Cf", 1.60, [31, 0, 255]),
    ("Es", 1.60, [31, 0, 255]),
    ("Fm", 1.60, [31, 0, 255]),
    ("Md", 1.60, [31, 0, 255]),
    ("No", 1.60, [31, 0, 255]),
    ("Lr", 1.60, [31, 0, 255]),
    ("Rf", 1.57, [128, 128, 128]),
    ("Db", 1.49, [128, 128, 128]),
    ("Sg", 1.43, [128, 128, 128]),
    ("Bh", 1.41, [128, 128, 128]),
    ("Hs", 1.34, [128, 128, 128]),
    ("Mt", 1.29, [128, 128, 128]),
    ("Ds", 1.28, [128, 128, 128]),
    ("Rg", 1.21, [128, 128, 128]),
    ("Cn", 1.22, [128, 128, 128]),
    ("Nh", 1.36, [128, 128, 128]),
    ("Fl", 1.43, [128, 128, 128]),
    ("Mc", 1.62, [128, 128, 128]),
    ("Lv", 1.75, [128, 128, 128]),
    ("Ts", 1.65, [128, 128, 128]),
    ("Og", 1.57, [128, 128, 128]),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symbols_round_trip_and_are_case_insensitive() {
        for symbol in ["H", "C", "Cl", "Og"] {
            let element = Element::from_symbol(symbol).expect("element");
            assert_eq!(element.symbol(), symbol);
        }
        assert_eq!(Element::from_symbol("cl").map(Element::symbol), Some("Cl"));
        assert_eq!(Element::from_symbol("X"), Some(Element::UNKNOWN));
        assert!(Element::from_symbol("Qq").is_none());
    }

    #[test]
    fn carbon_has_the_cpk_color_and_a_covalent_radius() {
        let carbon = Element::from_symbol("C").expect("carbon");
        assert_eq!(carbon.atomic_number(), 6);
        assert_eq!(carbon.cpk_color(), [144, 144, 144]);
        assert!((carbon.covalent_radius() - 0.76).abs() < 1e-9);
    }
}
