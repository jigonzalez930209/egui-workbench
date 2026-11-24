# Support

- **Platforms:** Linux (X11 and Wayland), macOS, Windows, and the WebAssembly host.
- **Rust:** 1.95 and the current stable toolchain.
- **Python:** CPython 3.9+ with abi3 wheels.
- **Browsers:** current Chrome, Edge, Firefox, and Safari for the wasm host; the remote
  renderer needs WebSocket and WebGL.
- **Data:** CSV, workbook, Arrow IPC/Parquet, SQLite, ring/channel connectors, NumPy,
  pandas, Polars, and PyArrow.
- **Accessibility:** automated contrast and spec audits are green; the manual
  screen-reader checklist is in `docs-site/reference/accessibility-report.md`.

Report bugs with a minimal spec or script and the version from `cargo metadata` / the
package metadata. Security issues go through GitHub Security Advisories, not public
issues.
