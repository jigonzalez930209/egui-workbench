//! The boundary to `sci-form` and `xtalforge-core`.

use crate::model::{Molecule, UnitCell};

/// Why an external chemistry engine failed.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum BridgeError {
    /// The engine is not available in this build or deployment.
    Unavailable(String),
    /// The engine rejected the input.
    InvalidInput(String),
    /// The engine failed while running.
    Failed(String),
}

impl std::fmt::Display for BridgeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unavailable(message) | Self::InvalidInput(message) | Self::Failed(message) => {
                formatter.write_str(message)
            }
        }
    }
}

impl std::error::Error for BridgeError {}

/// The `sci-form` operations the UI needs.
///
/// Implementations are supplied by the embedding application; the core crate only
/// defines the contract and never links the engine.
pub trait SciFormBridge: Send + Sync {
    /// Optimize the geometry and return a new molecule with updated coordinates.
    ///
    /// # Errors
    /// When the engine is unavailable or rejects the molecule.
    fn optimize(&self, molecule: &Molecule) -> Result<Molecule, BridgeError>;

    /// Generate up to `count` conformers.
    ///
    /// # Errors
    /// When the engine is unavailable or rejects the molecule.
    fn conformers(&self, molecule: &Molecule, count: usize) -> Result<Vec<Molecule>, BridgeError>;
}

/// The `xtalforge-core` operations the crystal viewer needs.
pub trait XtalBridge: Send + Sync {
    /// Apply space-group operations to fractional coordinates and return the Cartesian
    /// positions of the expanded cell.
    ///
    /// # Errors
    /// When the engine is unavailable or rejects the cell or operations.
    fn expand_cell(
        &self,
        cell: &UnitCell,
        operations: &[[[f64; 3]; 3]],
        fractional: &[[f64; 3]],
    ) -> Result<Vec<[f64; 3]>, BridgeError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::element::Element;
    use crate::model::Molecule;

    struct Mock;

    impl SciFormBridge for Mock {
        fn optimize(&self, molecule: &Molecule) -> Result<Molecule, BridgeError> {
            Ok(molecule.clone())
        }

        fn conformers(&self, molecule: &Molecule, count: usize) -> Result<Vec<Molecule>, BridgeError> {
            Ok((0..count).map(|_| molecule.clone()).collect())
        }
    }

    impl XtalBridge for Mock {
        fn expand_cell(
            &self,
            cell: &UnitCell,
            _operations: &[[[f64; 3]; 3]],
            fractional: &[[f64; 3]],
        ) -> Result<Vec<[f64; 3]>, BridgeError> {
            Ok(fractional
                .iter()
                .map(|point| cell.fractional_to_cartesian(*point))
                .collect())
        }
    }

    #[test]
    fn bridges_are_object_safe_and_return_molecules() {
        let bridge: Box<dyn SciFormBridge> = Box::new(Mock);
        let mut molecule = Molecule::new();
        molecule.add_atom(Element::UNKNOWN, [0.0; 3]);
        assert_eq!(bridge.conformers(&molecule, 3).expect("conformers").len(), 3);
        let xtal: Box<dyn XtalBridge> = Box::new(Mock);
        let cell = UnitCell::new(1.0, 1.0, 1.0, 90.0, 90.0, 90.0).expect("cell");
        let points = xtal.expand_cell(&cell, &[], &[[0.5, 0.0, 0.0]]).expect("cell");
        assert_eq!(points, vec![[0.5, 0.0, 0.0]]);
    }
}
