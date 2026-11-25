# egui-workbench-mol3d

A 3D molecular and crystal viewport for `egui-workbench`, built on the shared
`egui-workbench-chem-core` molecule.

This 0.1.0 milestone provides the foundation:

- `Mol3dScene` with atoms, bonds, radii, and crystal-cell edges from a shared molecule.
- `Camera3D` with an arcball quaternion (no gimbal lock), perspective/orthographic
  projection, pan, cursor zoom, fit, and screen/world rays.
- Ray-sphere picking with selection and hover.
- A depth-sorted `egui::Painter` viewport with depth cueing, bond order, CPK colors, and
  selection rings, usable in any dockable egui panel and on WASM.
- `Mol3dView::show` wires camera interaction, picking, selection, and painting.

The analytic `egui::PaintCallback` impostor renderer (wgpu spheres/cylinders with exact
`frag_depth` and an ID-buffer picking pass) is the next milestone; the painter viewport is
the reference and the fallback, so applications can adopt the panel now.

## Example

```rust
use egui_workbench_mol3d::{Mol3dScene, Mol3dView};

let scene = Mol3dScene::from_molecule(&molecule);
let mut view = Mol3dView::new(scene);
// In an egui update:
// view.show(ui);
```
