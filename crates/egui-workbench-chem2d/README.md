# egui-workbench-chem2d

A vector 2D chemistry editor built on `egui::Painter` and the shared
`egui-workbench-chem-core` model.

## Features

- `ChemDocument`, `Molecule2D`, `Atom2D`, and `Bond2D` on the shared molecule, with
  serde round-trips for documents, annotations, and superatoms.
- DPI-independent `Camera2D` with pan, cursor-centred zoom, grid, and fit.
- Painter rendering for single/double/triple/aromatic/dative/dotted bonds and
  wedged/hashed/wavy/crossed stereochemistry, with CPK atom colors.
- Contextual heteroatom labels (`OH`/`HO`, `NH2`/`H2N`), implicit hydrogens, and formal
  charges.
- A smallest-first cycle basis (SSSR), aromaticity detection, inward double-bond
  orientation, and aromatic circles.
- Ring stamps (benzene, cyclohexane, cyclopentadiene, pyridine, pyrrole, chair, boat),
  zig-zag chains, angular snapping, rotate, and mirror.
- Superatom library (`Ph`, `COOH`, `Boc`, `OTs`, `bpy`, `phen`, `EDTA`) with reversible
  expansion and contraction that preserves external bonds.
- Reaction arrows, equilibrium/resonance/retrosynthesis/failed/fishhook arrows,
  conditions boxes with sub/superscript formatting, polymer brackets, and semantic
  reaction schemes.
- SMILES read/write with a force-directed 2D layout, Molfile V2000/V3000, SDF, RXN, and
  a compact CDXML reader/writer.
- Molecular formula, average and exact mass, elemental analysis, double-bond
  equivalents, simple stereo descriptors, and valence validation.
- SVG and PNG export (96/300/600 DPI) with ACS, RSC, Wiley, and dark styles.
- Serializable `DocumentDelta` operations for the remote server and agent.
- Deterministic command history for atoms, bonds, templates, superatoms, annotations,
  rotation, mirroring, layout cleanup, and SMILES insertion.
- `Chem2dEditor` with click/rectangle/lasso selection, move, erase, keyboard shortcuts
  (element keys, bond orders, Space rotate), and drag feedback.
- Works on native and WASM (`cargo check --target wasm32-unknown-unknown`).

## Example

```bash
cargo run -p egui-workbench-chem2d --example editor
```

The example exposes every tool, the live properties panel, SMILES insertion, superatom
expansion, layout cleanup, and SVG/PNG export.

## Basic use

```rust
use egui_workbench_chem2d::{Chem2dEditor, ChemDocument, DocumentId};

let document = ChemDocument::new(DocumentId::new(1), "untitled");
let mut editor = Chem2dEditor::new(document);
// In an egui update:
// editor.show(ui);
```
