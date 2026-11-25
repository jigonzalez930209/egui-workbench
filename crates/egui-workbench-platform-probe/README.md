# Platform probe

A standalone native probe for the manual `Platform pixels` workflow. It intentionally
does not live under the root `examples/` directory: examples inherit the root crate's
dev-dependencies (`egui_kittest`, `criterion`, wgpu test backends), which made the Windows
compile take hours.

```bash
cargo run --manifest-path crates/egui-workbench-platform-probe/Cargo.toml -- \
  --screenshot artifacts/probe.png
```

Windows CI uses `--features wgpu` with `WORKBENCH_WGPU=1` because the hosted Windows image
has no desktop OpenGL.
