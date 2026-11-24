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
- File parsers: CSV, workbook, Arrow/Parquet, SQLite, spec, and manifest readers.

## Practices

- `cargo audit` runs in the manual CI job; the strict suite runs locally before a
  release tag. The 1.0 checklist also records a manual `cargo deny` review.
- The single tag workflow runs the basic tests, docs, npm, crates, and wheels from one
  run; duplicate push and per-package workflows were removed.
- Spec/protocol/extension API versions are checked and rejected when unsupported.
- Extension checksums are verified on install and reload; signatures are carried for
  applications with a signing policy.
- The remote server binds `127.0.0.1` by default and has no built-in authentication;
  deploy it behind the application's own auth and TLS.
- MCP is stdio-only and opens no socket.

## Known limitations

- The model-facing agent can propose harmful patches; approvals are the application's
  control. See `docs-site/reference/agent-threat-model.md`.
- Screen-reader walkthroughs are manual and recorded in
  `docs-site/reference/accessibility-report.md`.
