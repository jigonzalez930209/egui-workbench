# Archived: egui-workbench

This repository is the original home of the project. It is archived and read-only.

Development continues at **Crisol**:
https://github.com/jigonzalez930209/crisol

The last release here is **v1.3.0**.

## Why we moved

- The project outgrew the `egui-workbench` name and now targets a broader
  scientific workbench, with an egui-free core and a separate UI layer.
- The 1.x line couples its public API to egui. Crisol starts a new line at
  `0.1.3` so the core can reach a real 1.0 without egui breaking it.
- The new repository starts with a clean history and a one-file-per-commit
  import policy (assets are grouped by folder).
- `crisol` is available as a crate and npm package, which makes publishing and
  discovery coherent with the new architecture.

The old history, releases, and issues remain here for reference.
