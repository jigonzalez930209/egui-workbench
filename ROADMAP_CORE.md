# Roadmap: publicación de `crisol-core`

> **Estado:** plan activo. Empieza después del rename descrito en
> [`docs/RENAME_CRISOL.md`](docs/RENAME_CRISOL.md).
> **Objetivo:** publicar un core estable, egui-free, sobre el que construir la UI y las
> aplicaciones, empezando por `crisol-core 0.1.3` y terminando en `1.0.0`.

## 1. Decisión

- La marca es **Crisol**; el crate raíz es `crisol` y el core es `crisol-core`.
- `egui-workbench` queda archivado en `1.3.0`.
- Crisol arranca en **`0.1.3`**. No hereda la numeración 1.x.
- El hito que el roadmap viejo llamaba **2.0** pasa a ser **Crisol 1.0**.
- El core no conoce egui. La UI sí, y depende del core.
- La química queda congelada como plugin; no entra en el core.
- La publicación empieza por el core y es incremental: `0.1.3`, `0.2.0`, `0.3.0`,
  `0.4.0`, `0.5.0`, `1.0.0`.

### Relación con el roadmap viejo

`ROADMAP.md` sigue siendo útil como historia y como especificación de la UI. Su decisión
de partir en 2.0 queda reemplazada por este documento: el split ocurre ahora, en Crisol
0.1.3, y el antiguo 2.0 se publica como Crisol 1.0. `docs/FIX_PLAN.md` es una auditoría
cerrada y no un plan activo.

## 2. Reglas del core

### Dependencias

`crisol-core` no puede depender de:

- `egui`, `eframe`, `egui-wgpu`, `egui_kittest`;
- `epaint`, `emath`, `ecolor`;
- ningún crate que reexporte tipos de egui.

La guardia es explícita:

```bash
cargo tree -p crisol-core --all-features --edges normal,build,dev \
  | grep -E 'egui|eframe|epaint|emath|ecolor' && exit 1 || true
```

### API

El core puede devolver:

- datos, eventos, errores, estados, comandos, patches, frames;
- tipos propios: `Rgba`, `Key`, `Modifiers`, `Rect` solo si hace falta y es propio.

El core no puede devolver:

- `Ui`, `Response`, `Context`, `Color32`, `Stroke`, `TextureHandle`, `Vec2`, `Rect` de
  egui, ni ningún tipo de `epaint`/`emath`/`ecolor`.

### Tests

- El core se prueba headless: sin ventana, sin GPU, sin `egui_kittest`.
- Los tests de adaptación viven en `crisol`.
- Los snapshots visuales viven en la UI, no en el core.

### Commits

La extracción mantiene la misma regla del rename: **un archivo por commit**.

- Nunca `git add -A`, `git add .` ni `git commit -a`.
- Antes de cada commit, `git diff --cached --name-only` debe mostrar exactamente una ruta.
- Formato:

```text
extract(core): move <new-path> from egui-workbench@<sha>

Old path: <old-path>
Source: https://github.com/jigonzalez930209/egui-workbench/blob/<sha>/<old-path>
```

- `Cargo.lock` se commitea solo, después de los manifiestos.
- La historia intermedia no tiene que compilar; se verifica por crate y al final de cada
  fase.
- No se reescribe la historia del repo viejo; el SHA de origen queda en cada mensaje.

### Features

- Default liviano: sin Arrow, Parquet, SQLite, calamine ni wasmi.
- Cada conector pesado detrás de su feature.
- `crisol-core` debe compilar en `wasm32-unknown-unknown` con las features por defecto.

## 3. Arquitectura objetivo

```text
crisol-core                     (sin egui, estable)
├── crisol                      (UI: widgets, chrome, plot, viewer)
│   ├── crisol-host             (runtime + vista del spec)
│   ├── crisol-c
│   ├── crisol-python
│   └── crisol-wasm
├── crisol-remote               (protocolo del core; servidor sobre host)
├── crisol-agent                (tipos del core; MCP sobre host)
├── crisol-extensions           (manifest/ABI del core; runtime wasmi + UI)
└── crisol-chem-*               (plugins aparte)
```

### Módulos del core

```text
crisol-core/src/
  lib.rs
  ids.rs
  color.rs
  metrics.rs
  theme.rs
  locale.rs
  time.rs
  error.rs
  storage.rs
  commands/
    key.rs
    chord.rs
    command.rs
    registry.rs
    when.rs
    keybindings.rs
  state/
    documents.rs
    history.rs
    journal.rs
    layout.rs
    session.rs
    recovery.rs
    close_guard.rs
    theme_state.rs
  events.rs
  jobs.rs
  data/
    dataset.rs
    column.rs
    source.rs
    query/
    link.rs
    pyramid.rs
    stream.rs
    csv.rs
    workbook.rs
    arrow_ipc.rs
    parquet_file.rs
    sqlite.rs
    ring.rs
    channel.rs
  spec/
    host.rs
    widget.rs
    patch.rs
    schema.rs
  protocol/
    message.rs
    frame.rs
    data_buffer.rs
  extensions/
    manifest.rs
    capability.rs
    contribution.rs
    api.rs
  agent/
    permissions.rs
    access.rs
    audit.rs
```

### Features

```toml
default = ["serde", "data", "spec", "protocol"]

serde      = ["dep:serde", "dep:serde_json"]
schema     = ["dep:schemars"]
data       = []
csv        = []
workbook   = ["dep:calamine"]
arrow      = ["dep:arrow"]
parquet    = ["arrow", "dep:parquet", "dep:bytes"]
sqlite     = ["dep:rusqlite"]
ring       = []
channel    = []
spec       = []
protocol   = []
extensions = []
agent      = []
jobs       = []
storage    = []
```

## 4. Versiones

| Línea | Arranque | Estable | Notas |
|---|---:|---:|---|
| `egui-workbench` | `1.3.0` | — | Archivada; no recibe cambios. |
| `crisol` (UI) | `0.1.3` | `1.0.0` | El antiguo 2.0 es este 1.0. |
| `crisol-core` | `0.1.3` | `1.0.0` | Primera publicación al final de la fase 1. |
| `crisol-host` | `0.1.3` | `1.0.0` | Se publica después del core. |
| `crisol-remote` | `0.1.3` | `1.0.0` | Depende del protocolo del core. |
| `crisol-agent` | `0.1.3` | `1.0.0` | Tipos del core; MCP en el crate. |
| `crisol-extensions` | `0.1.3` | `1.0.0` | Manifest/ABI en el core. |
| `crisol-chem-*` | `0.1.3` | congelado | Plugins; no bloquean el core. |

Después de la primera publicación, cada crate puede avanzar por separado. El core no
espera a la UI ni la UI al core. La tabla de compatibilidad se publica en el README de
cada release. La UI puede quedarse en `0.1.3` hasta el 1.0 o publicar `0.x` cuando adopte
una fase del core; no está obligada a seguir cada versión.

Superficies versionadas aparte:

| Superficie | Versión | Regla |
|---|---:|---|
| Host spec | `1` | Un host rechaza un spec más nuevo. |
| Protocolo | `1` | Un host rechaza un protocolo más nuevo. |
| ABI de extensiones | `0.15` | El manifest declara el rango. |
| C ABI | nueva | Se renombra a `crisol_*`; se sube la versión. |
| Python | abi3 CPython 3.9+ | Sin cambios. |
| npm | ESM | Sin cambios. |

## 5. Fases

### Fase 0 — Repo Crisol y congelación

**Versión:** `0.1.3` (sin publicar).

Tareas:

- Ejecutar `docs/RENAME_CRISOL.md`.
- Crear `$HOME/github/crisol` y el repo público.
- Archivar `egui-workbench` con aviso y enlace.
- Congelar química.
- Actualizar `ROADMAP.md`, `VERSIONING.md` y `README.md`.
- Desactivar los jobs de publicación hasta tener tokens.

**Salida:** `crisol` 0.1.3 compila, pasa `cargo xtask check`, el repo viejo está archivado y no se publicó nada.

### Fase 1 — `crisol-core 0.1.3`: fundamentos y datos

**Objetivo:** crear el core real y publicar la primera versión.

Tareas:

- Crear `crates/crisol-core` con `#![forbid(unsafe_code)]`, `#![deny(missing_docs)]`,
  edition 2024 y MSRV 1.95.
- Añadir `cargo xtask core-check` con la guardia de dependencias y la matriz de features.
- Mover `src/data/**` completo y sus tests.
- Separar el conector `workbook` del visor: el conector solo depende de `calamine`; el
  visor se queda en la UI.
- Mover `metrics`, `localization`, `Theme`, `ThemeId`, `ThemeMode`, paleta y la lógica de
  Theme Studio, con un `Rgba` propio.
- Mover `storage` atómico: extraer `atomic_write` de `app/session.rs` a
  `crisol-core/src/storage.rs`, con las migraciones de workspace.
- Re-exportar todo desde `crisol` para que la UI no cambie de API.
- Añadir `docs/CORE.md` con las reglas de la sección 2.

**Salida:**

- `crisol-core` compila con `--no-default-features`, `--all-features` y wasm.
- `cargo tree` no contiene egui.
- Los tests de datos, tema y localización viven en el core.
- La suite de la UI sigue verde.
- `cargo publish --dry-run -p crisol-core` pasa.
- Se publica `crisol-core 0.1.3` cuando el token esté cargado.

**Publicación:** `crisol-core-v0.1.3`.

### Fase 2 — `crisol-core 0.2.0`: comandos e input

**Objetivo:** sacar egui del sistema de comandos.

Tareas:

- Crear `Key` y `Modifiers` propios, con `FromStr`, `Display` y nombres estables.
- Mover `command`, `chord`, `keybindings`, `registry` y `when`.
- Cambiar `shortcut(&egui::Context)` por `handle_key(key, modifiers, context)`.
- Mantener el estado de secuencias (`Ctrl+K Ctrl+S`) en el core.
- Implementar el adaptador en `crisol`: `egui::Key` → `crisol_core::Key`,
  `egui::Modifiers` → `crisol_core::Modifiers`, eventos → `handle_key`.
- Mover solo los metadatos de la paleta; el widget se queda en la UI.

**Salida:**

- Todos los tests de keybindings, conflictos, `when` y secuencias pasan en el core.
- La paleta y el editor de atajos de la UI se comportan igual.
- `cargo publish --dry-run -p crisol-core` pasa.

**Publicación:** `crisol-core-v0.2.0`.

### Fase 3 — `crisol-core 0.3.0`: estado de aplicación

**Objetivo:** que una aplicación pueda vivir sin UI.

Tareas:

- Mover `Documents`, `History`, `CommandJournal`, `Macro`, `MacroLibrary`, `Layout`,
  `Workspace`, `WorkspaceDocument`, `SessionPaths`, `Recovery`, `HotExit`, `Stash` y
  `StashDocument`.
- Separar `CloseGuard` en estado puro + adaptador de ventana.
- Separar `ThemeState` en estado puro + aplicación de estilos en la UI.
- Mover la persistencia versionada y las migraciones.
- Mantener `ThemeId`, `Layout`, `WorkspaceDocument` y payloads serializables.

**Salida:**

- Un test headless abre un workspace, lo modifica, lo guarda, lo recupera y hace
  round-trip.
- La UI usa los mismos tipos re-exportados.
- `cargo publish --dry-run -p crisol-core` pasa.

**Publicación:** `crisol-core-v0.3.0`.

### Fase 4 — `crisol-core 0.4.0`: spec, protocolo, extensiones y agente

**Objetivo:** que remote, agent y extensions dependan del core para los tipos.

Tareas:

- Mover `HostSpec`, `WidgetSpec`, `Patch`, `Message`, `Request`, `Response`, frames y
  `DataBuffer`.
- Mover `Manifest`, `ApiRange`, `Capability`, contribuciones y errores de extensión.
- Mover permisos de agente, accessibility tree y audit log.
- Generar JSON Schema desde el core y guardar snapshots.
- Mantener los fixtures de conformidad y el protocolo `1`.
- `crisol-remote`, `crisol-agent` y `crisol-extensions` importan los tipos del core; el
  runtime y la UI se quedan donde están.

**Salida:**

- El schema y los frames no cambian.
- Los clientes de conformidad pasan.
- Ningún crate de tipos depende de egui.
- `cargo publish --dry-run -p crisol-core` pasa.

**Publicación:** `crisol-core-v0.4.0`.

### Fase 5 — `crisol-core 0.5.0`: jobs, eventos y hardening

**Objetivo:** completar las primitivas que faltan y congelar la API.

Tareas:

- Generalizar `JobQueue`, `JobHandle`, `Progress`, `CancellationToken` y `Waker`
  (inspirado en `chem-core`, pero sin química).
- Añadir `EventQueue` con waker inyectable.
- Revisar toda la API pública: nombres, defaults, errores, `#[non_exhaustive]`, docs.
- Añadir fuzz de CSV, workbook, query y spec.
- Ejecutar matriz de features, MSRV, wasm, `cargo deny`, `cargo semver-checks` y
  `cargo doc -D warnings`.
- Escribir `MIGRATION_CORE.md`.

**Salida:**

- API freeze candidate.
- Ningún `unwrap`/`expect`/`panic!` en producción.
- Documentación completa.
- Presupuestos de datos verdes.
- `cargo publish --dry-run -p crisol-core` pasa.

**Publicación:** `crisol-core-v0.5.0`.

### Fase 6 — `crisol-core 1.0.0`

**Objetivo:** publicar la base estable.

Tareas:

- Cerrar la API y el schema.
- Verificar que dos aplicaciones internas compilan contra el core.
- Publicar `crisol-core 1.0.0`.
- Actualizar la tabla de compatibilidad y la guía de migración.
- Añadir un ejemplo headless: carga un spec, aplica patches, ejecuta comandos, guarda
  workspace y consulta datos, sin egui.

**Salida:** `crisol-core 1.0.0` publicado; `cargo semver-checks` sin diff inexplicado.

**Publicación:** `crisol-core-v1.0.0`.

### Fase 7 — `crisol 1.0.0` (el antiguo 2.0)

**Objetivo:** que la UI adopte el core estable.

Tareas:

- `crisol` 1.0 depende de `crisol-core 1.0`.
- Eliminar duplicados y deprecaciones.
- Actualizar template, skill, docs y ejemplos.
- Publicar `crisol 1.0.0`.
- Mantener `egui-workbench` 1.3.0 archivado y sin cambios.

**Salida:** UI 1.0 publicada; migración documentada; el antiguo “2.0” queda cumplido como
Crisol 1.0.

## 6. Publicación incremental

### Nombres

Comprobado el 2026-11-27:

- crates.io: `crisol`, `crisol-core`, `crisol-host`, `crisol-remote`, `crisol-agent`,
  `crisol-extensions`, `crisol-c`, `crisol-python`, `crisol-wasm`, `crisol-chem-core`,
  `crisol-chem2d`, `crisol-mol3d` están libres.
- npm: `crisol` está libre.
- PyPI: `crisol-workbench` y `pycrisol` están libres; `crisol` está ocupado.

Recomendación:

- Crate Rust: `crisol`, `crisol-core`, etc.
- PyPI: `crisol-workbench`, módulo `crisol`.
- npm: `crisol`.
- C: header `crisol.h`, librería `libcrisol_c`, símbolos `crisol_*`.

### Requisitos

- Cuenta de crates.io con email verificado.
- Token con permiso de publicación.
- `cargo publish --dry-run` verde.
- `README.md`, licencia, descripción, `repository`, `homepage`, `documentation`,
  `keywords` y `categories` en cada crate.
- `Cargo.toml` con `include` acotado para no empaquetar `target/` ni docs-site.

### Orden

1. `crisol-core`
2. `crisol-host`
3. `crisol-remote`
4. `crisol-agent`
5. `crisol-extensions`
6. `crisol`
7. `crisol-c`, `crisol-python`, `crisol-wasm`
8. `crisol-chem-core`, `crisol-chem2d`, `crisol-mol3d`

Esperar a que el índice vea cada versión antes de publicar el dependiente. El workflow
actual ya reintenta; conservar esa lógica.

### Tags y releases

- Core: `crisol-core-v0.1.3`, `crisol-core-v0.2.0`, ...
- UI: `crisol-v0.1.3`, `crisol-v1.0.0`, ...
- Bindings: `crisol-python-v0.1.3`, etc.
- Cada tag tiene su GitHub Release con notas y enlace a docs.rs.
- No usar un único `v0.1.3` para todo el workspace.

### Workflow

El `release.yml` actual asume que todos los crates comparten versión y publica todo en un
tag `v*`. Para Crisol:

- Partir en `release-core.yml` (tag `crisol-core-v*`) y `release-ui.yml` (tag `crisol-v*`).
- Mantener `workflow_dispatch` como ensayo sin publicar.
- Los jobs de publish deben exigir el secreto y el tag correcto.
- Hasta que existan los tokens, `publish` queda en `workflow_dispatch` y con `if: false`
  o un guard equivalente.
- El gate fuerte sigue siendo local: `cargo xtask check` y `cargo xtask core-check`.

### Tokens

```bash
# Local
cargo login
# o
export CARGO_REGISTRY_TOKEN="..."

# GitHub
gh secret set CARGO_REGISTRY_TOKEN
gh secret set PYPI_API_TOKEN
gh secret set NPM_TOKEN
```

Nunca commitear tokens. No publicar desde una máquina sin `cargo publish --dry-run` verde.

## 7. CI local

`cargo xtask core-check` debe ejecutar:

```bash
cargo fmt --all --check
cargo clippy -p crisol-core --all-targets --all-features -- -D warnings
cargo test -p crisol-core --all-features
cargo check -p crisol-core --no-default-features
cargo check -p crisol-core --all-features
cargo check -p crisol-core --all-features --target wasm32-unknown-unknown
RUSTDOCFLAGS="-D warnings" cargo doc -p crisol-core --no-deps
cargo +1.95 check -p crisol-core --all-features --locked
cargo deny check
cargo semver-checks -p crisol-core   # tras el baseline 1.0
```

Y la guardia:

```bash
cargo tree -p crisol-core --all-features --edges normal,build,dev \
  | grep -E 'egui|eframe|epaint|emath|ecolor' && exit 1 || true
```

Además:

- snapshots de JSON Schema;
- fixtures de protocolo;
- fuzz de 60 s por target;
- presupuestos de datos;
- `cargo package --list -p crisol-core` sin archivos inesperados.

El CI remoto sigue siendo un smoke test manual de Ubuntu. No se automatiza más.

## 8. Riesgos

| Riesgo | Mitigación |
|---|---|
| El core se convierte en “otra UI framework” | No renderiza, no interpreta widgets; solo estado, datos y protocolo. |
| `Key`/`Modifiers` rompen atajos | Adaptador exhaustivo y tests de comportamiento en la UI. |
| `Rgba` cambia el output visual | Conversión testeada y snapshots de UI intactos. |
| Features de datos explotan | Default liviano; matriz acotada; `cargo hack` o script. |
| Spec/protocolo se rompen al moverlos | Re-exports, snapshots de schema y fixtures de conformidad. |
| Release con versiones distintas | Publicar core primero; tabla de compatibilidad; tags por crate. |
| Jobs en wasm | `std::thread` solo en nativo; cola cooperativa o feature `threads`. |
| La química distrae | Congelada hasta core 1.0. |
| Faltan tokens y el release falla | Publicación desactivada; dry-run local; tokens después. |
| Nombres ocupados | Nombres comprobados; recheck antes de publicar. |

## 9. Decisiones pendientes

1. **¿Un solo crate core o `core` + `data` + `protocol`?** Recomendación: uno solo con
   features; si el peso molesta, separar después sin romper por re-exports.
2. **¿`Rgba` propio o `ecolor`?** Recomendación: propio, para que “sin egui” sea literal.
3. **¿`Key` propio o `keyboard-types`?** Recomendación: propio, pequeño y mapeado a egui
   en la UI.
4. **¿Jobs con threads o abstracción de executor?** Recomendación: `Waker` + feature
   `threads`; wasm cooperativo.
5. **¿Core 1.0 a la vez que UI 1.0?** Recomendación: core 1.0 primero; UI 1.0 lo adopta.
6. **¿La matemática de plots entra al core?** Recomendación: no en 1.0; evaluar después
   con demanda real.
7. **¿La versión de arranque es 0.1.3 para todo el workspace?** Este plan dice sí. Si se
   prefiere que `crisol-core` empiece en `0.1.0`, solo cambia la tabla y el primer tag.

## 10. Primer sprint

1. Ejecutar el rename (`docs/RENAME_CRISOL.md`).
2. Crear `crates/crisol-core` y `cargo xtask core-check`.
3. Mover `src/data/**`, `metrics`, `localization`, tokens de tema y `storage`.
4. Re-exportar desde `crisol` y correr la suite completa.
5. Escribir `docs/CORE.md` con las reglas y la matriz de features.
6. Medir `cargo tree` y tiempo de compilación antes/después.
7. `cargo publish --dry-run -p crisol-core`.
8. Publicar `crisol-core 0.1.3` cuando el token esté cargado.
9. Abrir el diseño de `Key`/`Modifiers` y el adaptador de input.

## 11. Definition of done de `crisol-core 1.0`

- [ ] `cargo tree` no contiene egui ni su ecosistema.
- [ ] La API pública está congelada y documentada.
- [ ] Tests headless, fuzz y presupuestos verdes.
- [ ] MSRV 1.95 y wasm verificados.
- [ ] `cargo semver-checks` sin diff inexplicado.
- [ ] `crisol-core 1.0.0` publicado en crates.io.
- [ ] `MIGRATION_CORE.md` publicado.
- [ ] Dos aplicaciones internas compilan contra el core.
- [ ] `crisol 1.0.0` adopta el core y publica.
- [ ] `egui-workbench` sigue archivado en 1.3.0, sin cambios.
