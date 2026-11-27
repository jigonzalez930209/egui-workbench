# Chemistry plugins

> A shared molecular document model, a 2D structure editor, and a 3D molecular and crystal viewer, published as three companion crates with the library. Live page: <https://jigonzalez930209.github.io/egui-workbench/guide/chemistry.html>

Three optional crates add chemistry to an egui-workbench application. They are plugins in
the sense that the library does not depend on them: add the ones you need, drive them from
your own `Command` enum, and leave the rest out. They share the library's version
(`1.2.x`) and are published together.

| Crate | What it is | Depends on |
|---|---|---|
| `egui-workbench-chem-core` | The egui-free document model, ids, selection bus, events, bridges, and background jobs | `serde` |
| `egui-workbench-chem2d` | A 2D structure editor, SMILES and molfile readers, properties, and SVG/PNG export | `chem-core`, `egui` |
| `egui-workbench-mol3d` | A 3D molecular, biomolecular, and crystal viewer with an offscreen renderer | `chem-core`, `egui` |

```toml
[dependencies]
egui-workbench-chem2d = "1.2"   # also brings egui-workbench-chem-core
egui-workbench-mol3d = "1.2"    # add the `gpu` feature for the wgpu renderer
```

`chem2d` and `mol3d` use the same atoms, bonds, selection, and document id, so a 2D editor
and a 3D viewer can show one molecule and stay in step. Both build for
`wasm32-unknown-unknown`.

## The shared model (`chem-core`)

```rust
use egui_workbench_chem_core::{BondOrder, ChemDocument, DocumentId, Element};

let mut document = ChemDocument::new(DocumentId::new(1), "water");
let oxygen = document
    .molecule
    .add_atom(Element::from_symbol("O").expect("oxygen"), [0.0, 0.0, 0.0]);
let hydrogen = document
    .molecule
    .add_atom(Element::from_symbol("H").expect("hydrogen"), [0.96, 0.0, 0.0]);
document.molecule.add_bond(oxygen, hydrogen, BondOrder::Single)?;
document.molecule.validate()?; // finite coordinates, live atoms and bonds
```

- **Ids are generational.** `AtomId` and `BondId` carry a slot and a generation, so an id
  kept after its atom was removed never resolves to the atom that reused the slot.
  Removing an atom frees its bonds' slots as well.
- **A selection bus** (`ChemSelection`, `SelectionMode`) and typed events
  (`ChemEvent`, `EventQueue`) connect the 2D and 3D panels. `EventQueue::with_waker` calls
  `request_repaint` when an event arrives.
- **Background jobs** (`JobQueue`, `Job`, `JobHandle`, `CancellationToken`) run
  conformer or surface work off the UI thread with progress and cooperative cancellation.
  A job that panics reports `JobError::Failed("the job panicked: …")` and still wakes the UI.
- **Bridges.** `SciFormBridge` and `XtalBridge` define the boundary to `sci-form` and
  `xtalforge-core`. The crate never links them; the application supplies an implementation.

## The 2D editor (`chem2d`)

```rust
use egui_workbench_chem2d::{Chem2dEditor, ChemDocument, DocumentId};

let document = ChemDocument::new(DocumentId::new(1), "untitled");
let mut editor = Chem2dEditor::new(document);
// In an egui update: editor.show(ui);
```

Edits go through a deterministic `Command` and a `History` with undo and redo. A command
that fails halfway is rolled back as a whole, so an error never leaves a half-applied edit.

```rust
use egui_workbench_chem2d::{Command, History};
use egui_workbench_chem_core::ChemSelection;

let mut history = History::new(); // unlimited; History::with_limit(n) caps the steps
let mut selection = ChemSelection::new();
history.execute(
    &mut document,
    &mut selection,
    &Command::InsertSmiles { text: "c1ccccc1".to_owned(), center: [0.0, 0.0] },
)?;
history.undo(&mut document, &mut selection);
```

### Drawing and editing

The bond tool follows the ChemDraw and MolView workflow. Click empty space to draw the
first bond, then click either end to sprout the next bond at 120°, which keeps a chain
zig-zagging. Dragging from an atom or empty space chooses the direction explicitly; the
endpoint snaps to 30° and 1.5 Å, a ghost atom previews it, and the gesture is one undo
step. Drag atom-to-atom to connect two existing atoms. An ending atom uses
`Chem2dEditor::draw_element`; the bond uses `bond_order` and `bond_stereo`. Clicking an
existing bond applies the current style, and the single tool on a single bond makes it
double (clicking again makes it single). A double bond is two separated lines; in a ring
the second line goes inside the ring. The atom tool changes an existing atom when it is
clicked. `EditorTool::Charge` is the MolView `e+`/`e−` tool.

Ring tools attach a ring to a clicked atom with one bond, or stamp it freely on empty
canvas. To attach a whole complex fragment, put its SMILES in
`Chem2dEditor::fragment_smiles` and select `EditorTool::AttachFragment`; clicking an atom
attaches and orients it outward. `Ctrl+V` over an atom does the same. The helper methods
`attach_ring` and `attach_smiles` expose it directly.

The canvas is skeletal: carbon vertices are unlabelled, heteroatoms and custom labels are
drawn in their CPK colour, and bonds stop at the label instead of running underneath it.
Labels, bond widths, and wedges scale with the camera zoom; the grid has minor, major, and
origin lines. An aromatic ring draws one circle and its ring bonds are single lines, so a
Kekulé structure never shows circles and double bonds at the same time. The SVG/PNG export
follows the same convention.

`layout::optimize_layout` (the `Clean` tool, `Ctrl+Shift+K`, also reachable as
`Command::CleanLayout`) is the 2D geometry optimizer. It builds a deterministic canonical
layout with 1.5 Å bonds, 120° chain angles, ring-specific interior angles (108° for a
five-membered ring, 120° for a six-membered one), fused-ring placement, and clearance for
terminal branches. It keeps the drawing centred, and optimizing the same graph twice is a
no-op.

A selection draws a dashed bounding box. Drag inside it to translate the whole selected
part; drag the round handle above the box to rotate around its centroid. A simple click
inside the box keeps the selection. Both gestures are one undo step.

For keyboard editing, the canvas takes focus after a click. Typing an element symbol over
a selection or a hovered atom changes it (`Cl` works as a two-letter symbol), `+` and `-`
adjust the formal charge, `1`–`4` set the bond order of the selected or hovered bond, `W`
and `H` set a wedge or hash, the arrow keys nudge by 0.1 Å (1 Å with Shift), and `Ctrl+A`
selects everything. Double-clicking an atom opens fields for its element, charge, and
custom label. `Ctrl+V` attaches clipboard SMILES to the hovered atom, or inserts it at the
pointer when no atom is hovered.

The same operations are available to a custom toolbar through `Chem2dEditor::execute` and
the `set_selected_*` helpers; every one goes through the history. `clear_document` is the
trash tool. The `editor` example arranges them as a MolView-style utility bar, bond palette
on the left, and atom palette with a periodic table on the right. Every tool is an icon
drawn with `egui::Painter`, including the bond, stereo, ring, chain, charge, amino-acid,
sugar, and functional-group icons; no icon font or asset registration is required.

### Chemistry libraries

`LIBRARY` is a compendium grouped by chemical type: amino acids and their side chains,
sugars, nucleobases, rings and heterocycles, functional groups, solvents, ligands and
chelators, and lipids. Every `LibraryEntry` carries a name, a SMILES string, and whether
it is a free molecule or a substituent. A test parses every entry and checks representative formulas, so a typo is
a test failure rather than an empty menu.

Clicking an entry in the example arms `EditorTool::AttachFragment`; the next click on an
atom attaches the structure there, and a click on empty canvas places it at the pointer.
The same path works for a custom SMILES string.

### Reading and writing

| Format | Read | Write |
|---|---|---|
| SMILES | `parse_smiles` (organic subset, brackets, branches, ring closures 1-9, aromatic atoms) | `to_smiles` |
| Molfile V2000 and V3000 | `from_molfile`, `from_v3000` | `to_v2000`, `to_v3000` |
| SDF and RXN | `from_sdf`, `from_rxn` | `to_sdf`, `to_rxn` |
| CDXML (a subset) | `from_cdxml` | `to_cdxml` |
| SVG and PNG | | `to_svg`, `to_png` (96, 300, 600 dpi; ACS, RSC, Wiley, dark) |

Readers return an error for malformed input and never panic on it. A V3000 charge is the
`CHG=` property, as other programs write it; a V2000 `M  CHG` line carries at most eight
atoms. A CDXML node without an `Element` attribute is carbon, as ChemDraw writes it.

### Properties and validation

```rust
use egui_workbench_chem2d::properties::valence_issues;
use egui_workbench_chem2d::{parse_smiles, properties};

let molecule = parse_smiles("CC(=O)O")?.to_molecule();
let found = properties(molecule.as_core());
assert_eq!(found.formula, "C2H4O2"); // Hill order; carbon-free compounds sort alphabetically
assert!(valence_issues(molecule.as_core()).is_empty());
```

`properties` returns the formula, average and exact mass (monoisotopic for the common
elements, the average mass for the rest), elemental analysis, and double-bond equivalents.
Implicit hydrogens come from one function that the atom labels and the formula share.
Formal charge changes the valence the way each group does it: `NH4+` and `H3O+` gain a
bond, `NH2-` and `OH-` lose one, `BH4-` gains one, and a carbocation and a carbanion are
both trivalent.

Known limits of the chemistry:

- **Aromatic heteroatoms.** An aromatic N, O, S, or Se has no implicit hydrogen count
  without a Kekulé structure (pyridine N has none, pyrrole N has one). Draw pyrrole in
  Kekulé form, as the template does, or set the hydrogens explicitly. Kekulé five-membered
  rings with one N, O, S, or Se and two double bonds count as aromatic, and so do Kekulé
  rings of 6, 10, or 14 atoms; four- and eight-membered alternating rings do not.
- **Stereo descriptors.** `stereo_descriptors` labels every stereo bond. It is a label,
  not a CIP assignment.

### Superatoms

`Ph`, `COOH`, `Boc`, `OTs`, `bpy`, `phen`, and `EDTA` expand to their atoms and contract
back, keeping external bonds. Each entry is checked by a test for valence, connectivity, and
ring size: `Boc` is `C(=O)OC(CH3)3`, `OTs` is p-toluenesulfonate with its aryl ring, `bpy`
is 2,2'-bipyridine, `phen` is 1,10-phenanthroline with the nitrogens facing each other, and
`EDTA` carries four `CH2-COOH` arms on its two nitrogens.

### Limits for untrusted input

A `DocumentDelta` (from an agent or a remote client) must carry finite coordinates within
10⁶ of the origin, and a document accepts at most 100,000 atoms through deltas. A chain
command adds at most 1,000 atoms. A SMILES bracket atom allows at most eight explicit
hydrogens. A PNG export is capped at 100 million pixels and 1,200 dpi. Layout cleanup
(`optimize_layout`, with `relax` kept as an alias) is a single canonical-placement pass
plus a fixed leaf-clearance pass, so a large molecule does not freeze the UI; the optimizer
is deterministic, keeps the document centred, and is idempotent.

## The 3D viewer (`mol3d`)

```rust
use egui_workbench_mol3d::{Mol3dScene, Mol3dView, parse_xyz};

let scene: Mol3dScene = parse_xyz(text)?.remove(0);
let mut view = Mol3dView::new(scene);
// In an egui update: view.show(ui);
let png = view.capture_png(1920, 1080, false)?; // the software renderer needs no GPU
```

| Area | What it covers |
|---|---|
| Formats | XYZ (also multi-frame and extended with `Lattice=`), MOL/SDF V2000, PDB with `CONECT`, mmCIF atom sites, CIF with symmetry, POSCAR/CONTCAR, Gaussian cube |
| Representations | Ball and stick, licorice, space filling, wireframe; CPK, Jmol, chain, residue, charge, and B-factor colors |
| Measurements | Distance, angle, dihedral, centroid, and mean plane |
| Biomolecules | CA traces and cartoon ribbons, hydrogen bonds, π–π stacking, salt bridges, short contacts |
| Crystals | Supercells, `(hkl)` slabs and planes, pores, coordination polyhedra |
| Surfaces and dynamics | Isosurfaces, hydrogen-like orbitals, solvent surfaces, trajectories, normal modes |
| Rendering | An arcball camera, optional wgpu impostors (`gpu` feature), a CPU fallback, and an offscreen renderer |

The dihedral follows the IUPAC sign: an anti arrangement is 180°, syn is 0°, and a
clockwise quarter turn viewed from the second atom to the third is +90°.

Interactions are found within 4.6 Å, and atoms within three bonds of each other are never
paired: the ends of a bond angle or a torsion sit closer than a van der Waals contact by
construction. The π–π test needs two planar five- or six-membered rings of C, N, O, or S
whose centroids are 3.2–4.6 Å apart and whose normals are nearly parallel.

### Driving the viewer from an agent

`Mol3dAction` is a serializable action set for the agent and remote surfaces: `load_cif`,
`set_supercell`, `toggle_polyhedra`, `focus_atoms`, `render_orbital`, `measure_distance`,
`toggle_miller_plane`, and `capture_png`. `SceneDescriptor` summarizes a scene for remote
synchronization.

### Limits for untrusted input

Readers return an error for truncated or malformed input. A `.cube` header cannot ask for a
grid larger than the file holds. A supercell is limited to 32 copies per axis and
2,000,000 atoms (a request above that returns the scene unchanged), pore sampling to
1,000,000 points, a coordination polyhedron to its 16 nearest neighbors, and an orbital
grid to 160 points per edge. A CPU render is at most 8,192 pixels on a side and 36 million
pixels, supersampling at most 4×; a larger request renders at the largest size that fits.
The pore and polyhedron searches do not consider periodic images, so results near the
edges of a cell are approximate.

## Examples

```bash
cargo run -p egui-workbench-chem2d --example editor
cargo run -p egui-workbench-mol3d --example viewer -- molecule.cif
cargo run -p egui-workbench-mol3d --example render -- figure.png molecule.pdb 3840 2160
```

The snippets on this page are compiled and run by each crate's `tests/docs.rs`.
