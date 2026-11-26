//! Composable molecular representations, applied per atom selection.

use std::collections::BTreeSet;

use egui_workbench_chem_core::{AtomId, Element};

use crate::scene::Atom3D;

/// How a subset of atoms and their bonds are drawn.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Representation {
    /// Small spheres on sticks; the default molecular view.
    #[default]
    BallAndStick,
    /// Thick sticks with no atom spheres.
    Licorice,
    /// Van der Waals spheres with no bonds.
    SpaceFilling,
    /// Thin lines only.
    Wireframe,
}

impl Representation {
    /// All representations, in menu order.
    pub const ALL: [Self; 4] = [Self::BallAndStick, Self::Licorice, Self::SpaceFilling, Self::Wireframe];

    /// A short label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::BallAndStick => "Ball & stick",
            Self::Licorice => "Licorice",
            Self::SpaceFilling => "Space filling",
            Self::Wireframe => "Wireframe",
        }
    }

    /// Whether atom spheres draw with this representation.
    pub const fn shows_atoms(self) -> bool {
        !matches!(self, Self::Licorice | Self::Wireframe)
    }

    /// Whether bonds draw with this representation.
    pub const fn shows_bonds(self) -> bool {
        !matches!(self, Self::SpaceFilling)
    }

    /// The atom display radius in Angstrom.
    pub fn atom_radius(self, atom: &Atom3D) -> f64 {
        match self {
            Self::BallAndStick => atom.radius.max(0.25),
            Self::SpaceFilling => vdw_radius(atom.element),
            Self::Licorice | Self::Wireframe => 0.0,
        }
    }

    /// The bond radius in Angstrom.
    pub const fn bond_radius(self) -> f64 {
        match self {
            Self::BallAndStick => 0.13,
            Self::Licorice => 0.24,
            Self::Wireframe => 0.035,
            Self::SpaceFilling => 0.0,
        }
    }
}

/// A representation applied to a subset of atoms; the last matching rule wins.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepresentationRule {
    /// Atoms covered by the rule.
    pub atoms: BTreeSet<AtomId>,
    /// Representation for those atoms.
    pub representation: Representation,
}

/// The default representation plus per-selection overrides.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepresentationSet {
    /// Representation used when no rule matches.
    pub default: Representation,
    /// Per-selection overrides, evaluated in order.
    pub rules: Vec<RepresentationRule>,
}

impl Default for RepresentationSet {
    fn default() -> Self {
        Self {
            default: Representation::BallAndStick,
            rules: Vec::new(),
        }
    }
}

impl RepresentationSet {
    /// Apply `representation` to `atoms`, replacing any earlier rule for that set.
    pub fn assign(&mut self, atoms: impl IntoIterator<Item = AtomId>, representation: Representation) {
        let atoms = atoms.into_iter().collect::<BTreeSet<_>>();
        if atoms.is_empty() {
            return;
        }
        self.rules.retain(|rule| rule.atoms != atoms);
        self.rules.push(RepresentationRule { atoms, representation });
    }

    /// Remove every override.
    pub fn clear_rules(&mut self) {
        self.rules.clear();
    }

    /// The representation for `atom`.
    pub fn for_atom(&self, atom: AtomId) -> Representation {
        self.rules
            .iter()
            .rev()
            .find(|rule| rule.atoms.contains(&atom))
            .map_or(self.default, |rule| rule.representation)
    }
}

/// A van der Waals radius in Angstrom.
pub fn vdw_radius(element: Element) -> f64 {
    let radius = match element.symbol() {
        "H" => 1.20,
        "C" => 1.70,
        "N" => 1.55,
        "O" => 1.52,
        "F" => 1.47,
        "P" => 1.80,
        "S" => 1.80,
        "Cl" => 1.75,
        "Br" => 1.85,
        "I" => 1.98,
        "B" => 1.92,
        "Si" => 2.10,
        "Se" => 1.90,
        "Na" => 2.27,
        "K" => 2.75,
        "Ca" => 2.31,
        "Mg" => 2.05,
        "Zn" => 2.39,
        "Fe" => 2.44,
        "Cu" => 2.40,
        "Au" => 2.44,
        _ => 0.0,
    };
    if radius > 0.0 {
        radius
    } else {
        (element.covalent_radius() * 1.65).max(0.9)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::Vec3;

    fn atom(element: &str) -> Atom3D {
        Atom3D::new(
            AtomId::new(1, 1),
            Element::from_symbol(element).expect("element"),
            Vec3::ZERO,
            0.4,
        )
    }

    #[test]
    fn rules_override_the_default_and_the_last_rule_wins() {
        let mut set = RepresentationSet::default();
        let a = AtomId::new(1, 1);
        let b = AtomId::new(2, 1);
        set.assign([a, b], Representation::SpaceFilling);
        set.assign([a], Representation::Wireframe);
        assert_eq!(set.for_atom(a), Representation::Wireframe);
        assert_eq!(set.for_atom(b), Representation::SpaceFilling);
        assert_eq!(set.for_atom(AtomId::new(3, 1)), Representation::BallAndStick);
    }

    #[test]
    fn space_filling_uses_van_der_waals_radii() {
        assert!(Representation::SpaceFilling.atom_radius(&atom("C")) > 1.5);
        assert!(Representation::Wireframe.atom_radius(&atom("C")) == 0.0);
    }
}
