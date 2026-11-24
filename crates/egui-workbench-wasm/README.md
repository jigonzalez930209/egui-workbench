# egui-workbench wasm

The WebAssembly host for `egui-workbench`. `wasm-pack build --target web` produces the
ESM module and the wasm binary; `js/index.js` is the npm wrapper with `createWorkbench`
and `connectRemote`. TypeScript types for the versioned spec are generated from the host
JSON Schema by `scripts/generate-types.mjs`.
