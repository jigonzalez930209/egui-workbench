# egui-workbench-mol3d

> Part of the egui-workbench **chemistry plugins**, versioned with the library (`1.2.x`):
> `egui-workbench-chem-core`, `egui-workbench-chem2d`, and `egui-workbench-mol3d`. Guide:
> <https://jigonzalez930209.github.io/egui-workbench/guide/chemistry.html>

```toml
[dependencies]
egui-workbench-mol3d = "1.2"
```

A 3D molecular, biomolecular, and crystallographic viewport for `egui-workbench`, built on
the shared `egui-workbench-chem-core` model.

## Rendering

- Arcball `Camera3D` with no gimbal lock, perspective/orthographic switching, pan, cursor
  zoom, whole-scene fit, selection focus, and direct/reciprocal axis alignment.
- Analytic GPU sphere and capsule/cylinder impostors through `egui::PaintCallback` and
  `egui-wgpu` (feature `gpu`), rendered into an offscreen color+depth target with exact
  fragment depth and composited into egui. The CPU painter remains the reference/fallback.
- GPU ID-buffer picking: a depth-tested `R32Uint` pass selects atoms and bonds, reads back
  the texel under the pointer synchronously on native, and falls back to CPU ray picking
  on WASM, orthographic cameras, or stalled readbacks.
- Per-selection representations: ball & stick, licorice, space filling (van der Waals),
  and wireframe, with multiple and dative/coordination bonds.
- Color schemes: CPK, Jmol, chain, residue, partial charge, and B-factor.
- Depth cueing, crystallographic axes, cell edges, coordination polyhedra, Miller planes,
  accessible-pore probes, cartoon ribbons, interaction dashes, and translucent surfaces.
- Frustum culling through a spatial grid and point LOD above 2,000 atoms.

## Reading and metrology

- `.xyz` (single and multi-frame), extended XYZ with `Lattice=`, `.mol`/`.sdf` V2000,
  `.pdb` with `CONECT`, `.mmcif` atom-site loops, `.cif` with cell parameters and symmetry
  operators, and VASP `POSCAR`/`CONTCAR`.
- Distance, bond angle, dihedral, ring/fragment centroid, and least-squares mean plane,
  with labels and normals drawn on the egui layer.
- Hydrogen bonds, π–π stacking, salt bridges, and short contacts.
- Protein CA traces with Catmull-Rom ribbon meshes and helix/sheet/loop assignment.

## Crystallography

- `CellAxes` direct and reciprocal vectors, cell-box rendering, and crystallographic axis
  gizmos.
- Supercell expansion (`na × nb × nc`), slab filtering by `(hkl)`, Miller-plane
  visualization, and probe-grid pore/channel highlight.
- Convex-hull coordination polyhedra around metal centers with shape naming.

## Surfaces, orbitals, and dynamics

- Marching-tetrahedra isosurfaces from regular scalar grids.
- Hydrogen-like 1s, 2s, 2p, and 3d orbital sampling; positive/negative lobes.
- Gaussian `.cube` volume reader for orbitals and electrostatic maps.
- Solvent-accessible and smoothed solvent-excluded surface grids.
- Multi-frame trajectories with interpolation and looping playback, plus normal-mode
  displacement animation.

## Offscreen and integration

- Deterministic software renderer for headless figures: 4K/8K, transparent background,
  supersampling, clipping half-spaces, SSAO, toon shading, and silhouette edges.
- `SceneDescriptor` for light remote synchronization (geometric hash, cell, camera,
  representation, colors, selection, measurements, polyhedra, and planes).
- Typed `Mol3dAction` API for agents: `load_cif`, `set_supercell`, `toggle_polyhedra`,
  `focus_atoms`, `render_orbital`, `measure_distance`, `toggle_miller_plane`, and
  `capture_png`.
- Native and WASM builds (`cargo check --target wasm32-unknown-unknown`).

The interactive GPU path renders spheres and bond cylinders with analytic ray hits and an
exact depth buffer, then composites through a fullscreen texture pass; the ID-buffer pick
runs in the same geometry with `frag_depth`. WASM and orthographic cameras use the CPU
painter/ray-picking fallback. The offscreen software renderer remains the reference for
regression images.

## Limits for untrusted input

A `.cube` header cannot ask for a grid larger than the file. A supercell is at most 32
copies per axis and 2,000,000 atoms, pore sampling at most 1,000,000 points, a coordination
polyhedron uses its 16 nearest neighbors, an orbital grid is at most 160 points per edge,
and a CPU render is at most 8,192 pixels per side and 36 megapixels. The dihedral follows
the IUPAC sign (anti is 180 degrees).

## Examples

```bash
cargo run -p egui-workbench-mol3d --example viewer -- molecule.cif
cargo run -p egui-workbench-mol3d --example render -- figure.png molecule.pdb 3840 2160
```

The viewer exposes file loading, representations, colors, supercells, polyhedra, Miller
planes, pores, cartoons, interactions, orbitals, measurements, and 4K export.

## Basic use

```rust
use egui_workbench_mol3d::{Mol3dScene, Mol3dView};

let scene = Mol3dScene::from_molecule(&molecule);
let mut view = Mol3dView::new(scene);
// In an egui update:
// view.show(ui);
```
