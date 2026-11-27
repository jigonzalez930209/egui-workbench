# Rename: `egui-workbench` → `crisol`

> **Estado:** instrucciones aprobadas, pendientes de ejecutar.
> **Regla:** este documento no se ejecuta a medias. El repo viejo no se archiva hasta que el nuevo compila, pasa la suite y está publicado en GitHub.

## 1. Decisión

- **Marca nueva:** Crisol.
- **Repo nuevo:** `$HOME/github/crisol`, público en `https://github.com/jigonzalez930209/crisol`.
- **Repo viejo:** `https://github.com/jigonzalez930209/egui-workbench`, queda como archivo histórico, read-only.
- **Versión:** `egui-workbench` se congela en `1.3.0`; Crisol arranca en `0.1.3`.
- **El antiguo “2.0” es Crisol 1.0.** No hay un Crisol 2.0 en este plan.
- **Química:** congelada. No se toca en el rename; sigue como plugins.
- **Publicación:** desactivada hasta cargar los tokens. Este rename no publica nada.

### Por qué nos movemos

1. La marca `egui-workbench` ata el producto a una dependencia y a una etapa del proyecto.
2. El core debe ser egui-free y estable; el repo viejo mezcla egui en la API pública.
3. La línea 1.x se queda en `1.3.0`; Crisol necesita una numeración nueva, empezando en `0.1.3`.
4. El repo nuevo nace con historia limpia y una política de commits explícita: **un archivo por commit**.
5. La publicación bajo `crisol` está disponible en crates.io y npm; `egui-workbench` ya no representa el alcance real.

## 2. Alcance

### Entra

- Copiar todo el árbol actual, menos `.git/`, `target/` y artefactos locales.
- Renombrar paquetes, crates, binarios, módulos, variables de entorno, rutas de configuración, URLs y textos de marca.
- Reorganizar los directorios `crates/egui-workbench-*` a `crates/crisol-*`.
- Crear el repo público y subir `main`.
- Archivar el repo viejo con un aviso que explica el movimiento.

### No entra

- Cambiar comportamiento, API o formato de datos.
- Extraer todavía `crisol-core` (eso es el roadmap del core).
- Publicar en crates.io, PyPI o npm.
- Tocar la química salvo el rename mecánico de sus crates.
- Reescribir la historia del repo viejo.

## 3. Preflight

Ejecutar en el repo viejo, con el árbol limpio:

```bash
cd "$HOME/github/egui-workbench"
git status --porcelain          # debe estar vacío
git rev-parse HEAD              # guardar este SHA: SOURCE_SHA
cargo xtask check               # baseline obligatorio
```

Guardar el baseline:

```bash
SOURCE_SHA="$(git rev-parse HEAD)"
mkdir -p "$HOME/github/crisol-baseline"
git bundle create "$HOME/github/crisol-baseline/egui-workbench.bundle" --all
git ls-files > "$HOME/github/crisol-baseline/manifest.txt"
cargo metadata --no-deps --format-version 1 > "$HOME/github/crisol-baseline/metadata.json"
```

Anotar en `crisol-baseline/README.md`:

- número de tests y snapshots;
- salida de `cargo tree` por crate;
- tamaño del binario release;
- resultado de `cargo xtask check`;
- fecha y SHA.

Crear un tag local en el repo viejo, sin push todavía:

```bash
git tag -a pre-crisol -m "Last egui-workbench commit before the Crisol rename"
```

**Congelar el repo viejo:** desde este momento no entra trabajo nuevo. Todo va al repo nuevo.

## 4. Estructura objetivo

```text
$HOME/github/crisol/
├── Cargo.toml                 # package crisol + workspace
├── Cargo.lock                 # regenerado, no copiado
├── src/                       # UI crate (se queda en la raíz)
├── assets/
├── locales/
├── benches/
├── tests/
├── examples/
├── xtask/
├── crates/
│   ├── crisol-core/           # se crea en la fase 1 del roadmap
│   ├── crisol-host/
│   ├── crisol-remote/
│   ├── crisol-agent/
│   ├── crisol-extensions/
│   ├── crisol-c/
│   ├── crisol-python/
│   ├── crisol-wasm/
│   ├── crisol-chem-core/
│   ├── crisol-chem2d/
│   ├── crisol-mol3d/
│   └── crisol-platform-probe/
├── docs/
├── docs-site/
├── clients/
├── web/
├── fuzz/
├── skills/crisol/
├── templates/app/
├── .claude-plugin/
└── .github/
```

El crate raíz se queda en la raíz para no mezclar el rename con una reorganización de workspace. `crisol-core` se añade después como hermano en `crates/`.

## 5. Mapa de nombres

### Paquetes

| Antes | Después |
|---|---|
| `egui-workbench` | `crisol` |
| `egui-workbench-host` | `crisol-host` |
| `egui-workbench-remote` | `crisol-remote` |
| `egui-workbench-agent` | `crisol-agent` |
| `egui-workbench-extensions` | `crisol-extensions` |
| `egui-workbench-c` | `crisol-c` |
| `egui-workbench-python` | `crisol-python` |
| `egui-workbench-wasm` | `crisol-wasm` |
| `egui-workbench-chem-core` | `crisol-chem-core` |
| `egui-workbench-chem2d` | `crisol-chem2d` |
| `egui-workbench-mol3d` | `crisol-mol3d` |
| `egui-workbench-platform-probe` | `crisol-platform-probe` |
| `egui-workbench-core` | `crisol-core` (nuevo, fase 1) |

### Identificadores Rust

| Antes | Después |
|---|---|
| `egui_workbench` | `crisol` |
| `egui_workbench_host` | `crisol_host` |
| `egui_workbench_remote` | `crisol_remote` |
| `egui_workbench_agent` | `crisol_agent` |
| `egui_workbench_extensions` | `crisol_extensions` |
| `egui_workbench_c` | `crisol_c` |
| `egui_workbench_python` | `crisol_python` |
| `egui_workbench_wasm` | `crisol_wasm` |
| `egui_workbench_chem_core` | `crisol_chem_core` |
| `egui_workbench_chem2d` | `crisol_chem2d` |
| `egui_workbench_mol3d` | `crisol_mol3d` |
| `egui_workbench_platform_probe` | `crisol_platform_probe` |

### Binarios, bindings y marca

| Antes | Después |
|---|---|
| `egui-workbench` (bin) | `crisol` |
| `egui-workbench-mcp` | `crisol-mcp` |
| `egui-workbench-remote` (bin) | `crisol-remote` |
| `egui_workbench.h` | `crisol.h` |
| `libegui_workbench_c` | `libcrisol_c` |
| `egui_workbench` (módulo Python) | `crisol` |
| `egui-workbench` (PyPI) | `crisol-workbench` |
| `egui-workbench` (npm) | `crisol` |
| `egui-workbench` (skill/plugin) | `crisol` |

### Entorno y rutas

| Antes | Después |
|---|---|
| `EGUI_WORKBENCH_CONFIRM` | `CRISOL_CONFIRM` |
| `EGUI_WORKBENCH_SIDECAR` | `CRISOL_SIDECAR` |
| `~/.config/egui-workbench` | `~/.config/crisol` |
| `~/.local/share/egui-workbench` | `~/.local/share/crisol` |
| `~/.cache/egui-workbench` | `~/.cache/crisol` |

Añadir una migración de una sola vez: si existe el directorio viejo y no el nuevo, copiarlo; aceptar las variables viejas como fallback durante una release y avisar por log.

### URLs

| Antes | Después |
|---|---|
| `github.com/jigonzalez930209/egui-workbench` | `github.com/jigonzalez930209/crisol` |
| `jigonzalez930209.github.io/egui-workbench/` | `jigonzalez930209.github.io/crisol/` |
| `docs.rs/egui-workbench` | `docs.rs/crisol` |
| `crates.io/crates/egui-workbench` | `crates.io/crates/crisol` |

### Lo que NO se renombra

- La dependencia `egui`, `eframe`, `egui-wgpu`, `egui_kittest` ni sus paths `egui::`.
- El contenido histórico de `CHANGELOG.md` y de las notas de release viejas.
- El nombre del repo viejo dentro de `ARCHIVED.md` y del aviso de archivo.
- `LICENSE-APACHE` y `LICENSE-MIT` (solo revisar el año si hace falta).
- Nombres de terceros: Material Icon Theme, Lucide, egui, etc.

## 6. Política de commits

**Un commit = un archivo.** Sin excepciones durante el rename.

- Nunca `git add -A`, `git add .` ni `git commit -a`.
- Antes de cada commit: `git diff --cached --name-only` debe mostrar exactamente una ruta.
- Formato:

```text
import(<scope>): add <new-path> from egui-workbench@<sha>

Old path: <old-path>
Source: https://github.com/jigonzalez930209/egui-workbench/blob/<sha>/<old-path>
```

o, para una edición manual:

```text
rename(<scope>): apply Crisol naming to <path>
```

Scopes: `core`, `ui`, `host`, `remote`, `agent`, `extensions`, `bindings`, `chem`, `docs`, `ci`, `xtask`, `workspace`.

Helper obligatorio:

```bash
commit_file() {
  local file="$1" message="$2"
  if git rev-parse --verify HEAD >/dev/null 2>&1; then
    git reset -q
  fi
  git add -f -- "$file"
  local staged
  staged="$(git diff --cached --name-only | wc -l | tr -d ' ')"
  if [ "$staged" -ne 1 ]; then
    echo "ERROR: expected 1 staged file for $file, got $staged" >&2
    exit 1
  fi
  git commit -m "$message"
}
```

Notas:

- La historia intermedia **no tiene que compilar**: es una historia de importación. La verificación se hace por crate y al final.
- `Cargo.lock` se regenera y se commitea solo, al final.
- Si un archivo necesita dos cambios lógicos, se hacen en el mismo commit porque sigue siendo un archivo; no se mezclan dos archivos.
- No se hace `--amend` después de push ni `force-push` a `main`.

## 7. Orden de ejecución

### Paso 1 — Crear el repo local

```bash
export GH_USER="jigonzalez930209"
export SRC="$HOME/github/egui-workbench"
export DST="$HOME/github/crisol"
export SOURCE_SHA="$(git -C "$SRC" rev-parse HEAD)"

mkdir -p "$DST"
cd "$DST"
git init -b main
```

### Paso 2 — Primer commit: `.gitignore`

Con `commit_file` definido (sección 6), el repo nuevo necesita ignorar `target/` y los
artefactos de build antes de cualquier compilación. Copiar `.gitignore` del repo viejo,
aplicarle el rename de las rutas de Python/wasm y commitearlo solo:

```bash
cp "$SRC/.gitignore" "$DST/.gitignore"
perl -0pi -e '
  s/crates\/egui-workbench-python/crates\/crisol-python/g;
  s/crates\/egui-workbench-wasm/crates\/crisol-wasm/g;
' "$DST/.gitignore"
commit_file ".gitignore" "build(repo): add Crisol .gitignore"
```

El resto de archivos de gobierno (README, CHANGELOG, licencias, toolchain, etc.) se
importan en el paso 3 y se editan en el paso 5. El script salta cualquier archivo que ya
esté rastreado, así que `.gitignore` no se duplica.

### Paso 3 — Importar el árbol

Usar el script de la sección 8. Importa los archivos rastreados del repo viejo, aplica el rename mecánico y commitea uno por uno.

Antes de ejecutarlo, commitea `docs/RENAME_CRISOL.md` y `ROADMAP_CORE.md` en el repo viejo (un archivo por commit) para que `git ls-files` los vea. Si prefieres no tocarlos allí, cópialos a mano al repo nuevo antes del primer push.

Exclusiones:

- `Cargo.lock` (se regenera).
- `.git/`, `target/`, `docs-site/node_modules/`, artefactos de build.
- `docs/RENAME_CRISOL.md` y `ROADMAP_CORE.md` se importan sin reescritura (son las instrucciones).
- `README.md`, `CHANGELOG.md`, `ROADMAP.md`, `VERSIONING.md`, `WORKBENCH.md` se importan sin reescritura y se editan a mano después.

### Paso 4 — Regenerar el lockfile

```bash
cd "$DST"
cargo generate-lockfile
commit_file "Cargo.lock" "build(workspace): regenerate Cargo.lock for Crisol"
```

### Paso 5 — Ediciones manuales, un archivo por commit

| Archivo | Qué cambiar |
|---|---|
| `Cargo.toml` | `name`, `description`, `repository`, `homepage`, `documentation`, `keywords`, `categories`. |
| `README.md` | Hero, badges, install, enlaces, sección “Origin”. |
| `CHANGELOG.md` | `## [0.1.3]` con “Rename to Crisol”; historial viejo debajo. |
| `ROADMAP.md` | Enlazar `ROADMAP_CORE.md`; decir que el antiguo 2.0 es Crisol 1.0. |
| `VERSIONING.md` | Core y UI con versiones independientes; política de egui solo para la UI. |
| `WORKBENCH.md` | Título y referencias de crate; sigue siendo la spec de la shell. |
| `docs-site/.vitepress/config.*` | `title`, `base: "/crisol/"`, nav y links. |
| `docs-site/package.json` | `name`, `repository`. |
| `.github/workflows/*.yml` | Nombres de crates, tag, URLs; **desactivar publish** hasta tener tokens. No conservar la comprobación de versión única: partir en `release-core.yml` y `release-ui.yml` (ver `ROADMAP_CORE.md`). |
| `.github/ISSUE_TEMPLATE/*` | Repo y links. |
| `templates/app/Cargo.toml` | `crisol = "0.1.3"`. |
| `xtask/src/main.rs` | Variable de template `workbench_path` → `crisol_path`; versión por defecto `0.1.3`. |
| `skills/crisol/SKILL.md` | `name: crisol`, `metadata.crate: crisol`, `metadata.version: 0.1.3`, URLs y ejemplos. |
| `.claude-plugin/plugin.json` | `name`, `version: 0.1.3`, `description`, `homepage`, `repository`. |
| `.claude-plugin/marketplace.json` | Nombre, versión y descripción. |
| `crates/crisol-python/pyproject.toml` | `name = "crisol-workbench"`, `version = "0.1.3"`, `module-name = "crisol._native"`, URLs. |
| `crates/crisol-python/python/crisol/*` | Imports y nombres internos. |
| `crates/crisol-wasm/package.json` | `name = "crisol"`, `version: 0.1.3`, descripción, repo. |
| `crates/crisol-c/include/crisol.h` | Guardas, prefijo `crisol_`, versión ABI. |
| `crates/crisol-c/src/*` | Símbolos `wb_*` → `crisol_*` (decisión manual). |
| `clients/go/*` | `CRISOL_SIDECAR`, URLs y nombres. |
| `web/*` | Título, package name, links. |
| `docs/*.md` | Referencias, ejemplos, links. |

Cada fila se commitea por separado. Si una fila incluye varios archivos, se parte en varios commits.

### Paso 6 — Verificación

```bash
cd "$DST"
HISTORICAL=(
  ':!docs/RENAME_CRISOL.md'
  ':!ROADMAP_CORE.md'
  ':!CHANGELOG.md'
  ':!ROADMAP.md'
  ':!README.md'
  ':!docs/FIX_PLAN.md'
  ':!docs/MIGRATION_0.4.md'
  ':!docs/MIGRATION_0.5.md'
  ':!docs-site/reference/changelog.md'
  ':!docs-site/reference/migration-0.4.md'
  ':!docs-site/reference/migration.md'
  ':!docs-site/reference/roadmap.md'
)
git grep -n -E 'egui-workbench|egui_workbench|EGUI_WORKBENCH|EguiWorkbench' -- . "${HISTORICAL[@]}"
```

Solo se permiten referencias históricas en esos archivos. Los docs de migración y el changelog conservan los nombres viejos a propósito; si quieres, se les añade un banner que diga “nombres históricos”.

Después, la suite completa:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --lib
cargo xtask check
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
cargo check --workspace --all-features
cargo check -p crisol --no-default-features
cargo deny check
```

Y la prueba de humo:

```bash
cargo run -p crisol-host --bin crisol
```

La ventana debe decir **Crisol**, no `egui-workbench`.

### Paso 7 — Crear el repo público

```bash
gh auth status
gh repo create "$GH_USER/crisol" \
  --public \
  --source=. \
  --remote=origin \
  --description "Crisol: an egui-free core and desktop workbench for scientific applications."
git push -u origin main
```

Configurar en GitHub:

- Topics: `rust`, `egui`, `gui`, `desktop`, `scientific-computing`, `workbench`, `data`.
- Pages: source GitHub Actions; `DOCS_BASE=/crisol/`.
- Actions: dejar `release.yml` en `workflow_dispatch` hasta tener tokens.
- Issues: activados. Discussions: opcional.
- Social preview: logo de Crisol (opcional).

### Paso 8 — Archivar el repo viejo

En el repo viejo, antes de archivar:

1. Crear `ARCHIVED.md` (contenido abajo) y commitearlo solo.
2. Añadir el banner al inicio de `README.md` y commitearlo solo.
3. Push de ambos commits.
4. Archivar:

```bash
gh repo archive "$GH_USER/egui-workbench" --yes
```

Contenido de `ARCHIVED.md`:

```markdown
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
  import policy.
- `crisol` is available as a crate and npm package, which makes publishing and
  discovery coherent with the new architecture.

The old history, releases, and issues remain here for reference.
```

Banner para el `README.md` viejo:

```markdown
> **Archived.** This is the original `egui-workbench` repository. Development
> continues at [Crisol](https://github.com/jigonzalez930209/crisol). The last
> release here is v1.3.0. See [ARCHIVED.md](ARCHIVED.md) for why we moved.
```

### Paso 9 — Enlazar el origen desde el repo nuevo

En el `README.md` de Crisol, añadir una sección:

```markdown
## Origin

Crisol began as [`egui-workbench`](https://github.com/jigonzalez930209/egui-workbench),
now archived. The old repository keeps the 1.x history and releases; Crisol starts
a new line at 0.1.3 with an egui-free core.
```

## 8. Script de importación

> **Advertencia:** es una referencia, no un botón. Revisar el diff de cada archivo antes de commitear. El script no debe ejecutarse sobre un repo con cambios sin commitear.

<details>
<summary>Script de importación, un archivo por commit</summary>

```bash
#!/usr/bin/env bash
set -euo pipefail

SRC="${SRC:-$HOME/github/egui-workbench}"
DST="${DST:-$HOME/github/crisol}"
GH_USER="${GH_USER:-jigonzalez930209}"
SOURCE_SHA="$(git -C "$SRC" rev-parse HEAD)"
OLD_REPO="https://github.com/$GH_USER/egui-workbench"

mkdir -p "$DST"
cd "$DST"
git init -b main

commit_file() {
  local file="$1" message="$2"
  if git rev-parse --verify HEAD >/dev/null 2>&1; then
    git reset -q
  fi
  git add -f -- "$file"
  local staged
  staged="$(git diff --cached --name-only | wc -l | tr -d ' ')"
  if [ "$staged" -ne 1 ]; then
    echo "ERROR: expected 1 staged file for $file, got $staged" >&2
    exit 1
  fi
  git commit -m "$message"
}

scope() {
  case "$1" in
    crates/crisol-core/*) echo core ;;
    crates/crisol-host/*) echo host ;;
    crates/crisol-remote/*) echo remote ;;
    crates/crisol-agent/*) echo agent ;;
    crates/crisol-extensions/*) echo extensions ;;
    crates/crisol-c/*|crates/crisol-python/*|crates/crisol-wasm/*) echo bindings ;;
    crates/crisol-chem-*|crates/crisol-chem2d/*|crates/crisol-mol3d/*) echo chem ;;
    src/*|assets/*|locales/*) echo ui ;;
    docs/*|*.md) echo docs ;;
    .github/*|.githooks/*) echo ci ;;
    xtask/*) echo xtask ;;
    *) echo workspace ;;
  esac
}

new_path() {
  local p="$1"
  p="${p//crates\/egui-workbench-python\/python\/egui_workbench/crates\/crisol-python\/python\/crisol}"
  p="${p//crates\/egui-workbench-/crates\/crisol-}"
  p="${p//skills\/egui-workbench/skills\/crisol}"
  p="${p//egui-workbench-mcp.rs/crisol-mcp.rs}"
  p="${p//egui-workbench-remote.rs/crisol-remote.rs}"
  p="${p//egui-workbench.rs/crisol.rs}"
  p="${p//egui_workbench.h/crisol.h}"
  printf '%s\n' "$p"
}

is_text() {
  file --brief --mime "$1" | grep -qE '^text/|charset=(us-ascii|utf-8)' \
    || case "${1##*.}" in
         svg|md|rs|toml|json|yml|yaml|sh|py|pyi|ts|js|mjs|html|css|ftl|c|h|go|vue|mod) return 0 ;;
         *) return 1 ;;
       esac
}

rewrite() {
  perl -0pi -e '
    s/egui-workbench-chem-core/crisol-chem-core/g;
    s/egui-workbench-chem2d/crisol-chem2d/g;
    s/egui-workbench-mol3d/crisol-mol3d/g;
    s/egui-workbench-platform-probe/crisol-platform-probe/g;
    s/egui-workbench-extensions/crisol-extensions/g;
    s/egui-workbench-python/crisol-python/g;
    s/egui-workbench-remote/crisol-remote/g;
    s/egui-workbench-agent/crisol-agent/g;
    s/egui-workbench-host/crisol-host/g;
    s/egui-workbench-wasm/crisol-wasm/g;
    s/egui-workbench-c/crisol-c/g;
    s/egui-workbench/crisol/g;
    s/egui_workbench_chem_core/crisol_chem_core/g;
    s/egui_workbench_chem2d/crisol_chem2d/g;
    s/egui_workbench_mol3d/crisol_mol3d/g;
    s/egui_workbench_platform_probe/crisol_platform_probe/g;
    s/egui_workbench_extensions/crisol_extensions/g;
    s/egui_workbench_python/crisol_python/g;
    s/egui_workbench_remote/crisol_remote/g;
    s/egui_workbench_agent/crisol_agent/g;
    s/egui_workbench_host/crisol_host/g;
    s/egui_workbench_wasm/crisol_wasm/g;
    s/egui_workbench_c/crisol_c/g;
    s/egui_workbench/crisol/g;
    s/EGUI_WORKBENCH/CRISOL/g;
    s/EguiWorkbench/Crisol/g;
  ' "$1"
}

skip_rewrite() {
  case "$1" in
    docs/RENAME_CRISOL.md|ROADMAP_CORE.md|CHANGELOG.md|README.md|ROADMAP.md|VERSIONING.md|WORKBENCH.md|docs/FIX_PLAN.md|docs/MIGRATION_0.4.md|docs/MIGRATION_0.5.md|docs-site/reference/changelog.md|docs-site/reference/migration-0.4.md|docs-site/reference/migration.md|docs-site/reference/roadmap.md)
      return 0 ;;
    *) return 1 ;;
  esac
}

git -C "$SRC" ls-files -z | while IFS= read -r -d '' old; do
  [ "$old" = "Cargo.lock" ] && continue
  new="$(new_path "$old")"
  if git ls-files --error-unmatch "$new" >/dev/null 2>&1; then
    continue
  fi
  mkdir -p "$(dirname "$new")"
  cp -p "$SRC/$old" "$new"
  if ! skip_rewrite "$old" && is_text "$new"; then
    rewrite "$new"
  fi
  commit_file "$new" "import($(scope "$new")): add $new from egui-workbench@$SOURCE_SHA

Old path: $old
Source: $OLD_REPO/blob/$SOURCE_SHA/$old"
done
```

</details>

## 9. Compatibilidad y migración

### Rust

- No hay promesa de compatibilidad con `egui-workbench` 1.x. Crisol 0.1.3 es una línea nueva.
- Los nombres de crate cambian; los usuarios actualizan `Cargo.toml` y los `use`.
- Mantener `egui-workbench` 1.3.0 publicado y congelado; no yankearlo.

### Configuración y datos

- Copiar `~/.config/egui-workbench` a `~/.config/crisol` en el primer arranque.
- Aceptar `EGUI_WORKBENCH_*` como fallback durante una release; emitir un warning.
- Los archivos de workspace/sesión que guarden el nombre de la app deben aceptar el valor viejo al leer y escribir el nuevo.
- Si el protocolo o el spec incluyen el nombre en un magic string, aceptar ambos durante la transición; si no, no cambiar la versión.

### Bindings

- **C:** renombrar header, librería y símbolos `wb_*` → `crisol_*`; subir la versión ABI. Es un break asumido en 0.1.3.
- **Python:** módulo `crisol`; distribución PyPI `crisol-workbench` (`crisol` está ocupado). No publicar todavía.
- **npm:** paquete `crisol` (disponible). Ajustar `out-name` de wasm-pack y los tipos.
- **MCP:** servidor `crisol`; conservar alias de tool names solo si un cliente viejo lo necesita.

## 10. Publicación y tokens

Este rename **no publica**. Los jobs de publish quedan desactivados hasta que existan los secretos.

Cuando toque:

```bash
# Local, nunca en el repo
cargo login
# o
export CARGO_REGISTRY_TOKEN="..."

# GitHub
gh secret set CARGO_REGISTRY_TOKEN
gh secret set PYPI_API_TOKEN
gh secret set NPM_TOKEN
```

Reglas:

- No commitear `~/.cargo/credentials.toml` ni tokens.
- `cargo publish --dry-run` antes de cada publicación real.
- Orden: `crisol-core` primero; después `crisol-host`, `crisol-remote`, `crisol-agent`, `crisol-extensions`; al final `crisol` y bindings.
- Esperar a que el índice de crates.io vea la versión antes de publicar dependientes.
- Tags separados por crate: `crisol-core-v0.1.3`, `crisol-v0.1.3`, etc.

## 11. Verificación final

- [ ] `git status` limpio en el repo nuevo.
- [ ] `git log --oneline` muestra un archivo por commit (revisar una muestra).
- [ ] `git grep` de nombres viejos solo en archivos históricos.
- [ ] `cargo metadata` lista solo paquetes `crisol*`.
- [ ] `cargo xtask check` verde.
- [ ] `cargo doc` sin warnings.
- [ ] `cargo check --workspace --all-features` verde.
- [ ] `cargo check -p crisol --no-default-features` verde.
- [ ] `cargo deny check` verde.
- [ ] `cargo xtask style-lint` verde.
- [ ] `python3 docs-site/scripts/build-skill.py --check` verde.
- [ ] `cd docs-site && npm ci && npm run build` verde con `base: "/crisol/"`.
- [ ] `cd crates/crisol-wasm && npm run build && npm run types:check` verde.
- [ ] `cargo build -p crisol-c --release` verde y el header se llama `crisol.h`.
- [ ] El binario `crisol` abre y muestra “Crisol”.
- [ ] El template `cargo xtask new` compila contra `crisol`.
- [ ] La docs-site compila con `base: "/crisol/"`.
- [ ] El repo viejo tiene `ARCHIVED.md` y el banner, y está archivado.
- [ ] El `README.md` nuevo enlaza al repo viejo como origen.
- [ ] No se publicó nada en crates.io/PyPI/npm.

## 12. Rollback

- El repo viejo no se toca hasta el paso 8; si algo falla antes, se sigue trabajando allí.
- Guardar el bundle de `crisol-baseline`; permite restaurar el estado exacto.
- Si el repo nuevo queda mal, se borra y se recrea; no se fuerza push al viejo.
- Si el rename de un crate rompe la suite, se revierte ese archivo con `git revert <commit>` (un archivo) y se reinvestiga.

## 13. Definition of done

1. `$HOME/github/crisol` existe, compila y pasa `cargo xtask check`.
2. El repo público `jigonzalez930209/crisol` está creado y con `main` subido.
3. La historia del repo nuevo es un archivo por commit.
4. El repo viejo está archivado, con aviso y enlace al nuevo.
5. No hay referencias a `egui-workbench` fuera de los archivos históricos permitidos.
6. La publicación sigue desactivada y documentada.
7. `ROADMAP_CORE.md` es la guía activa para extraer y publicar `crisol-core`.
