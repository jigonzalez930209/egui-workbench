# egui-workbench-chem-core

The shared foundation for the chemistry crates: an egui-free molecular document model,
generational IDs, a selection bus, typed events, bridge traits for `sci-form` and
`xtalforge-core`, and cooperative background jobs.

This crate contains no UI and no GPU code. `egui-workbench-chem2d` and
`egui-workbench-mol3d` depend on it so a 2D editor and a 3D viewer share the same atoms,
bonds, selection, and document identity.

## What it provides

- `DocumentId`, `AtomId`, and `BondId`: stable identifiers with generational reuse.
- `Element`: the periodic table, covalent radii, and CPK colors.
- `Molecule` and `ChemDocument`: atoms, bonds, charges, bond order, and stereochemistry.
- `ChemSelection`: atom and bond sets with replace/add/toggle modes.
- `ChemEvent` and `EventQueue`: structure, selection, geometry, conformer, and cell events.
- `SciFormBridge` and `XtalBridge`: the integration boundary for the external engines.
- `JobQueue`, `JobHandle`, `JobContext`, and `CancellationToken`: cooperative background
  work with progress and cancellation. `JobQueue::with_waker` and
  `EventQueue::with_waker` call `egui::Context::request_repaint` on progress, completion,
  or a new event.

## Example

```rust
use egui_workbench_chem_core::{ChemDocument, DocumentId, Element};
use egui_workbench_chem_core::model::BondOrder;

let mut document = ChemDocument::new(DocumentId::new(1), "water");
let oxygen = document.molecule.add_atom(Element::O, [0.0, 0.0, 0.0]);
let hydrogen = document.molecule.add_atom(Element::H, [0.96, 0.0, 0.0]);
document
    .molecule
    .add_bond(oxygen, hydrogen, BondOrder::Single)
    .expect("bond");
assert_eq!(document.molecule.atoms.len(), 2);
```
