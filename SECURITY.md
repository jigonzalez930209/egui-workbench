# Security

## Reporting

Report vulnerabilities privately through GitHub Security Advisories on
`jigonzalez930209/egui-workbench`. Do not open a public issue for a vulnerability.

Include the affected crate/package and version, a minimal reproduction, and the impact.
We aim to acknowledge within five working days.

## Scope

- `egui-workbench`, `egui-workbench-host`, Python wheels, npm package, C ABI, remote
  server, agent MCP server, and extensions.
- The extension sandbox: no WASI, no ambient filesystem or network, capability-checked
  imports.
- The agent permission model: read-only default, command allow-lists, confirmation
  tokens, activity log.
- File parsers: CSV, workbook, Arrow/Parquet, SQLite, spec, and manifest readers, and the
  chemistry readers (molfile and SDF, SMILES, CDXML, XYZ, PDB, mmCIF, CIF, POSCAR, cube).

## Practices

- `cargo audit` runs in the manual CI job; the strict suite runs locally before a
  release tag. The 1.0 checklist also records a manual `cargo deny` review.
- The single tag workflow runs the basic tests, docs, npm, crates, and wheels from one
  run; duplicate push and per-package workflows were removed.
- Spec/protocol/extension API versions are checked and rejected when unsupported.
- Extension checksums are verified on install and reload, against the bytes that are
  loaded. A signed manifest is refused unless the application installs a verifier with
  `ExtensionManager::with_signature_verifier`.
- The remote server binds `127.0.0.1` by default and has no built-in authentication;
  deploy it behind the application's own auth and TLS. It enforces handshake and write
  timeouts, a 32-connection cap, and a 64 MiB frame cap that is read incrementally.
- The SQLite connector is read-only; connectors and the viewer cap what they read into
  memory (512 MiB per file, 5,000,000 SQLite rows); query and `when` parsers bound their
  nesting; session files are written atomically through an exclusively created temporary.
- Readers of untrusted files (molfile, CIF, mmCIF, cube, PDB, SMILES, CDXML) return an
  error instead of panicking, and bound the memory a header can request.
- MCP is stdio-only and opens no socket.

## Known limitations

- `calamine` expands a worksheet before this crate can check its size, so a crafted
  `.xlsx` can use a lot of memory. Open untrusted workbooks in a separate process.

- An extension that does not export `alloc` receives the command id at offset 1024 of its
  memory, which can overwrite module data. Export `alloc(len) -> ptr`.
- `ExtensionManager` does not ship a signature algorithm. The application chooses the
  scheme and key and supplies the verifier.

- The model-facing agent can propose harmful patches; approvals are the application's
  control. See `docs-site/reference/agent-threat-model.md`.
- Screen-reader walkthroughs are manual and recorded in
  `docs-site/reference/accessibility-report.md`.
