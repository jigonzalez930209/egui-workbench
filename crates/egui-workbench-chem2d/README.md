# egui-workbench-chem2d

A vector 2D chemistry editor built on `egui::Painter` and the shared
`egui-workbench-chem-core` model.

Version 0.1.0 covers the first milestone:

- `ChemDocument`, `Molecule2D`, `Atom2D`, and `Bond2D` on top of the shared molecule.
- A DPI-independent `Camera2D` with pan, cursor-centred zoom, optional grid, and fit.
- Painter rendering for single, double, triple, aromatic, dative, wedged, hashed,
  wavy, and crossed bonds, with CPK atom colors and contextual heteroatom labels
  (`OH` vs `HO`, implicit hydrogens, formal charges).
- Selection by click, rectangle, and lasso, with replace/add/subtract/toggle modes.
- A deterministic command history with unlimited undo/redo for add atom, add bond,
  delete selection, set bond order/stereochemistry, and translate fragment.
- `Chem2dEditor`, which wires the camera, painter, tools, history, and keyboard
  shortcuts (`Ctrl+Z`, `Ctrl+Shift+Z`, Delete).

Later milestones add templates, SSSR/aromaticity perception, superatoms, reaction
arrows, SMILES/Molfile/CDXML, publication export, 3D synchronization, and agent tools.

## Example

```rust
use egui_workbench_chem2d::{Chem2dEditor, ChemDocument, DocumentId};

let document = ChemDocument::new(DocumentId::new(1), "untitled");
let mut editor = Chem2dEditor::new(document);
// In an egui update:
// editor.show(ui);
```
