# egui-workbench-chem2d

> Part of the egui-workbench **chemistry plugins**, versioned with the library (`1.2.x`):
> `egui-workbench-chem-core`, `egui-workbench-chem2d`, and `egui-workbench-mol3d`. Guide:
> <https://jigonzalez930209.github.io/egui-workbench/guide/chemistry.html>

```toml
[dependencies]
egui-workbench-chem2d = "1.2"
```

A vector 2D chemistry editor built on `egui::Painter` and the shared
`egui-workbench-chem-core` model.

## Features

- `ChemDocument`, `Molecule2D`, `Atom2D`, and `Bond2D` on the shared molecule, with
  serde round-trips for documents, annotations, and superatoms.
- DPI-independent `Camera2D` with pan, cursor-centred zoom, grid, and fit.
- Painter rendering for single/double/triple/aromatic/dative/dotted bonds and
  wedged/hashed/wavy/crossed stereochemistry, with CPK atom colours. Bonds stop at the
  heteroatom label, labels and stroke widths scale with zoom, the grid has minor, major,
  and origin lines, and aromatic circles use the bond colour.
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
- `layout::optimize_layout`, the geometry optimizer used by `Command::CleanLayout`
  (`Ctrl+Shift+K`): a deterministic canonical layout with 1.5 Å bonds, 120° chain angles,
  ring-specific interior angles, fused-ring placement, branch clearance, and packed
  components. It keeps the drawing centred and a second call is a no-op.
- SMILES read/write with the same 2D layout, Molfile V2000/V3000, SDF, RXN, and a compact
  CDXML reader/writer.
- Molecular formula, average and exact mass, elemental analysis, double-bond
  equivalents, simple stereo descriptors, and valence validation.
- SVG and PNG export (96/300/600 DPI) with ACS, RSC, Wiley, and dark styles.
- Serializable `DocumentDelta` operations for the remote server and agent.
- Deterministic command history for atoms, bonds, templates, superatoms, annotations,
  rotation, mirroring, layout cleanup, and SMILES insertion.
- `Chem2dEditor` with click/rectangle/lasso selection, move, erase, and drag feedback.
  A selection draws a box that can be dragged to move the whole part; the round handle
  above it rotates the part around its centroid, both as one undo step.
  The bond tool works like ChemDraw and MolView: click empty space for the first bond,
  click an atom to sprout the next bond at 120°, or drag for an explicit direction
  (30° snapping, ghost endpoint). Clicking a single bond with the single tool makes it
  double; clicking again makes it single. Double bonds are drawn as two separated lines,
  with the second line inside a ring. Each gesture is one undo step.
- Ring tools attach to a clicked atom with one bond, or stamp freely on empty canvas.
  `EditorTool::AttachFragment` plus `fragment_smiles` attaches a whole parsed SMILES
  fragment to the clicked atom, oriented outward so it does not grow over the existing
  molecule.
- Skeletal rendering: carbon vertices are unlabelled, heteroatoms keep their CPK colour,
  and custom labels are drawn as whole groups. An aromatic ring uses one circle and its
  ring bonds are drawn as single lines, so a Kekulé structure is never shown with both
  circles and double bonds. The SVG/PNG export uses the same convention.
- Atom and bond editing without a dialog: type an element symbol over a selection or
  hovered atom, `+`/`-` adjust formal charge, `1`–`4` set bond order (also over a hovered
  bond), `W`/`H` set a wedge or hash, arrow keys nudge, and `Ctrl+A` selects all.
  Double-click an atom for element, charge, and custom-label fields.
- `EditorTool::Charge` exposes the MolView `e+`/`e−` tools. Custom atom labels (`Ph`,
  `COOH`, a residue name) render in the editor, SVG, and PNG; `Ctrl+V` inserts clipboard
  SMILES at the pointer, or attaches it when the pointer is over an atom.
  `set_selected_element`, `adjust_selected_charge`,
  `set_selected_label`, `set_selected_bond_order`, `set_selected_bond_stereo`,
  `clear_document`, and `execute` expose the same edits to a toolbar.
- The `editor` example is laid out like MolView: a vector-only icon bar, bond/ring/chain
  palette on the left, atom palette plus a periodic table on the right, and canvas in the
  middle. Icons are drawn with `egui::Painter`; no icon font or asset registration is
  required.
- A chemistry compendium in `library` groups amino acids and their side chains, sugars,
  nucleobases, rings and heterocycles, functional groups, solvents, ligands/chelators, and
  lipids. Every entry carries SMILES and is validated by a test; clicking one arms the
  attach tool, then a click on an atom attaches it or a click on empty canvas places it.
- Works on native and WASM (`cargo check --target wasm32-unknown-unknown`).

## Example

```bash
cargo run -p egui-workbench-chem2d --example editor
```

The example exposes every tool, the live properties panel, SMILES insertion, superatom
expansion, layout cleanup, and SVG/PNG export.

## Limits for untrusted input

Readers return an error for malformed input. Deltas accept finite coordinates within 10^6
of the origin and at most 100,000 atoms; a chain command adds at most 1,000 atoms; a PNG
export is capped at 100 million pixels and 1,200 dpi. See the guide for the chemistry
caveats (aromatic heteroatoms, stereo labels).

## Basic use

```rust
use egui_workbench_chem2d::{Chem2dEditor, ChemDocument, DocumentId};

let document = ChemDocument::new(DocumentId::new(1), "untitled");
let mut editor = Chem2dEditor::new(document);
// In an egui update:
// editor.show(ui);
```
