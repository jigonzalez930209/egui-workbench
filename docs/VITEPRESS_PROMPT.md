# Prompt: documentación VitePress de `egui-workbench`

> Copia todo lo que hay debajo de la línea y pásaselo a la IA que va a escribir la
> documentación. Dale acceso de lectura al repositorio completo (o adjunta `src/`,
> `templates/app/`, `examples/`, `docs/`, `WORKBENCH.md`, `CHANGELOG.md` y
> `tests/snapshots/`).

---

## Rol y objetivo

Eres un redactor técnico senior especializado en Rust y en interfaces inmediatas
(egui). Vas a crear la documentación completa de **egui-workbench**, una librería Rust
para construir aplicaciones de escritorio con aspecto de IDE/workbench (al estilo
VS Code / MD View) sobre `egui 0.37` y `eframe`. La documentación se publica con
**VitePress** y debe permitir que un desarrollador que no conoce la librería construya
una aplicación real de principio a fin, y que uno avanzado la personalice a fondo.

Escribe la documentación en **inglés** (el código, los nombres y los comentarios de la
librería están en inglés). Mantén un tono directo, sin marketing.

## Reglas de exactitud (obligatorias)

1. **No inventes API.** Antes de documentar cualquier tipo, método, campo o variante,
   ábrelo en `src/`. Si algo no existe, no lo documentes. Si dudas, búscalo con grep.
2. Cada ejemplo de código debe **compilar** contra la librería tal como está. Usa los
   caminos de importación reales (`egui_workbench::components::prelude::*`,
   `egui_workbench::components::plot::{...}`, `egui_workbench::app::{...}`, etc.).
   Los ejemplos cortos deben poder pegarse dentro de un `fn ui(ui: &mut egui::Ui)`; los
   largos deben ser programas completos con `main`.
3. Copia los valores por defecto, tamaños y colores desde el código (`metrics.rs`,
   `style.rs`, `config.rs`, `theme/palette.rs`), no de memoria.
4. Respeta el contrato de la librería: **los componentes no poseen estado de
   aplicación**. Reciben `&mut` estado y devuelven el cambio en el mismo frame
   (`bool`, `Option<T>`, `Response`, `Vec<Event>`). Explícalo y respétalo en todos los
   ejemplos.
5. Toda la app compone `Command`s: menús, paleta y atajos producen el mismo enum, y un
   único `App::handle` muta el estado. Los ejemplos de aplicación deben seguir ese
   patrón (míralo en `templates/app/src/app.rs` y `commands.rs`).
6. El proyecto debe usar el **parche del fork de egui** (`[patch.crates-io]` del
   `Cargo.toml` raíz) con las mismas revisiones; documenta por qué y cópialo literal.

## Fuentes que debes leer primero

| Fuente | Para qué |
|---|---|
| `WORKBENCH.md` | Especificación completa: principios, anatomía de la ventana, temas, tamaños, reglas de interacción, i18n, testing, checklist |
| `docs/ARCHITECTURE.md`, `docs/COMPONENTS.md`, `docs/ICONS.md`, `docs/NEW_PROJECT.md` | Guías existentes (reutiliza y amplía, no contradigas) |
| `src/lib.rs` y `src/components/mod.rs` | Mapa de la API pública y doc comments (`//!`, `///`) |
| `src/components/plot/` | Gráficos: `mod.rs`, `config.rs`, `series.rs`, `fit.rs`, `axis.rs`, `export.rs` |
| `src/app/` | Runtime: `show_workbench`, `WorkbenchApp`, `ThemeState`, `Documents`, `Keymap`, `CloseGuard` |
| `templates/app/` | La app de arranque que genera `cargo xtask new`; es el mejor ejemplo real |
| `templates/app/src/gallery.rs`, `charts.rs`, `chart_settings/`, `icon_browser.rs` | Ejemplos de cada componente en uso |
| `examples/catalog`, `examples/showcase` | Ejemplos ejecutables (`cargo run --example …`) |
| `tests/` | Comportamiento garantizado (úsalo para afirmar comportamientos y valores) |
| `tests/snapshots/*.png` | Capturas de referencia reales de la UI, temas claro y oscuro |
| `CHANGELOG.md` | Qué existe y qué cambió |

## Estructura del sitio VitePress

Genera un proyecto VitePress completo en `docs-site/`:

```
docs-site/
  package.json                # vitepress como devDependency, scripts dev/build/preview
  .vitepress/
    config.ts                 # nav, sidebar, búsqueda local, editLink, lastUpdated,
                              # tema claro/oscuro, shiki con resaltado de Rust y TOML
    theme/
      index.ts                # extiende el tema por defecto
      custom.css              # colores alineados con GitHub Light/Dark del workbench
      components/
        Preview.vue           # captura (claro/oscuro según el tema del sitio) + código
        ApiTable.vue          # tabla de props/métodos con tipo, defecto y descripción
        TokenSwatch.vue       # muestra de color de un token de tema
  public/screenshots/         # capturas (ver "Capturas")
  index.md                    # home con hero, features y una captura del workbench
  guide/…
  components/…
  charts/…
  theming/…
  app/…
  recipes/…
  reference/…
```

### Secciones y páginas (mínimo)

**1. Guide (empezar)**
- `guide/introduction.md`: qué es, qué no es (shell, no contenido), filosofía
  (immediate mode + estado del caller, comandos, tokens), capturas.
- `guide/installation.md`: requisitos (Rust 1.95+, edition 2024), dependencias,
  `[patch.crates-io]` literal, features `material-icons` y `lucide-icons`, tamaño
  binario aproximado de cada feature.
- `guide/quick-start.md`: `cargo xtask new ../my-app --name my-app --title "My App"`,
  `cargo run`, recorrido por los ficheros generados (`main.rs`, `app.rs`, `ui.rs`,
  `commands.rs`, `model.rs`, `content.rs`, `settings.rs`, `gallery.rs`) y qué borrar
  para empezar el producto real.
- `guide/first-app.md`: tutorial paso a paso desde cero (sin el template): ventana sin
  decoraciones, `show_workbench` con título, explorador, pestañas, barra de estado,
  un `Command` enum, atajos con `Keymap`, paleta, ajustes persistidos, guardado con
  `CloseGuard`. Código completo al final.
- `guide/architecture.md`: capas (`foundation`, `chrome`, `widgets`, `components`,
  `app`), orden fijo de paneles, flujo de un frame, de dónde salen los colores y
  tamaños.
- `guide/concepts.md`: contrato de estado, comandos, tokens, componentes vs funciones
  (API de builders vs API de funciones originales), accesibilidad (roles AccessKit),
  sin repintado en reposo, movimiento reducido.

**2. Components** (una página por componente o grupo). Cada página con esta plantilla:
1. Qué es y cuándo usarlo (y cuándo no).
2. `<Preview>` con la captura en claro y oscuro.
3. Ejemplo mínimo.
4. Ejemplo realista (con estado en un struct del modelo y manejo del resultado).
5. Tabla de la API (`ApiTable`): constructor, cada método del builder con tipo,
   valor por defecto y efecto, y el tipo de retorno de `show`.
6. Variantes, tamaños y estados (deshabilitado, seleccionado, foco, hover).
7. Accesibilidad (rol y etiqueta que expone) y teclado.
8. Notas de diseño (tamaños de `metrics`, tokens usados) y errores comunes.

Componentes a cubrir (todos los exportados): `Button`, `IconButton`, `CloseButton`,
`ButtonVariant`/`Size`/`IconButtonSize`/`Tone`, `TextField`, `TextArea`, `Switch`,
`Checkbox`, `Select`, `Stepper`, `SegmentedControl`, `Label`, `Badge`, `Banner`,
`Card`, `Separator`, `Popover`, `Toasts`/`Toast` (tipos, posiciones, acciones,
`loading` → `update`, pausa en hover, eventos), `Menu`, `MenuButton`, `MenuItem`,
`MenuToggle`, `MenuIconButton`, `NavTab`, `TabStrip`, `Tree` (`TreeNode`, `TreeState`,
`RowIcon`, teclado, menú contextual), `CommandPalette`, `Dialog`, `ConfirmDialog`,
`NameDialog`, `ProgressDialog`, `StatusBar`, `IconPreview`, `IconGrid` (columnas
automáticas, `IconSize::Auto`, virtualización), `split_panes`, `show_empty_state`,
keyboard menu, settings dialog, choice window.

**3. Charts** (sección propia, es la parte más extensa):
- `charts/overview.md`: capacidades, captura de la sección Charts de la galería.
- `charts/series.md`: `Series::line`, `area`, `step`, `scatter` (7 `Marker`), `bars`,
  `function`, `hline`, `vline`, `.y_errors`, `.fill_to`, `.style(LineStyle)`, datos
  prestados (`Cow`), valores no finitos que cortan líneas.
- `charts/axes-and-ticks.md`: `Scale::Linear/Log10`, límites fijos (`x_limits`,
  `AxisConfig::min/max`), `major_step`, `Minor::{Auto, Off, Divisions(n)}`, decimales,
  notación de los ejes log, regla de etiquetas sin solaparse.
- `charts/interaction.md`: pan, rueda (Shift/Alt), zoom por caja, selección,
  doble clic, herramientas (`PlotTool`), barra, leyenda, hover y `PlotResponse`
  (`hovered`, `clicked`, `pointer`, `selection_changed`, `view`, `frame`,
  `screen_pos`, `data_pos`). Ejemplo de anotaciones dibujadas encima.
- `charts/configuration.md`: `PlotConfig` campo por campo (`PlotColors`,
  `SeriesStyle`, `sanitize`, `series_style_mut`), cómo guardar la config en el modelo
  y construir un diálogo de ajustes (resume `templates/app/src/chart_settings/`).
- `charts/fits.md`: `Fit::new`, `FitKind` (lineal, polinómico 1–8, exponencial,
  potencia, logarítmico, media móvil), `eval`, `equation`, `r_squared`, `series`,
  `points_in` con la selección.
- `charts/export.md`: `to_svg`, `to_png`, `ExportOptions`, `export_button`,
  `settings_button`, patrón "construir el `Plot` en una función y usarlo para `show` y
  para exportar", fuentes en el PNG, ejemplo con guardado a disco y toast.
- `charts/live-data.md`: `x_window`, buffer con tope, `request_repaint` solo mientras
  hay streaming.
- `charts/performance.md`: decimación M4 por columna de píxel, datos ordenados,
  1 M de puntos, coste por frame, recomendaciones.

**4. Theming & customization**
- `theming/themes.md`: los 9 `ThemeId` (familias GitHub, Dracula, One Dark Pro),
  `apply_id`, `apply_theme` (seguir al sistema), `ThemeState`, cambio en vivo.
- `theming/tokens.md`: cada campo de `Theme` con `TokenSwatch` por tema, cómo se mapean
  a `egui::Visuals` (`theme/apply.rs`), `theme::current`, `readable_on`,
  `contrast_ratio` y la garantía WCAG AA de los tests.
- `theming/metrics.md`: todas las constantes de `metrics` con su uso.
- `theming/icons.md`: `Icons` (UI glyphs, Lucide por nombre, `register` de SVG propios,
  `paint`/`put`/`show`, `texture_sized` y nitidez HiDPI), `MaterialIcons`.
- `theming/custom-themes.md`: temas propios con `Theme::with_accent`, el struct
  `Theme` con sintaxis de actualización, `Theme::accessibility_issues`, `theme::mix`,
  `theme::apply_tokens`, `ThemeState::set_custom` y cómo re-derivar el acento al
  cambiar de tema o al seguir al sistema (ver `templates/app/src/appearance.rs`).
  Capturas: `tests/snapshots/components-custom-accent.png`.
- `theming/style-and-density.md`: `style::WorkbenchStyle` campo por campo,
  `Density::{Compact, Default, Comfortable}` con sus alturas resultantes, presets
  `sharp()`/`rounded()`, `style::set`/`style::get`, qué componentes lo leen y qué no
  (el chrome de ventana mantiene tamaños fijos, explica por qué). Capturas:
  `components-compact-sharp.png`, `components-comfortable-rounded.png`.
- `theming/customization.md`: las tres capas (tema → estilo global → overrides por
  instancia: `Button::fill/text_color/stroke/radius`, `IconButton::color/radius`,
  `Card::fill/stroke/radius`, `Badge::radius`, `Tone::Custom`, `Banner::color`,
  `PlotConfig`), fuentes propias, iconos propios y layouts propios con
  `WorkbenchApp`. Incluye una receta "marca propia" de principio a fin.

**5. Application runtime**
- `app/workbench.md`: `WorkbenchApp` (cada método, valores por defecto),
  `show_workbench`, ids de paneles, orden fijo (título → explorador → panel derecho →
  pestañas → barra de estado → panel inferior → contenido → overlays),
  `SIDEBAR_MIN_WIDTH`, `CONTENT_MIN_WIDTH`, `RIGHT_PANEL_MIN_WIDTH`,
  `BOTTOM_PANEL_MIN_HEIGHT`, comportamiento responsive. Ejemplo con Inspector y
  Output como en `templates/app/src/panels.rs`.
- `app/commands-and-keymap.md`: `Keymap`, atajos, paleta, menús, keyboard menu (F10/Alt).
- `app/documents.md`: `Documents<T: Document>`, pestañas sucias, `CloseGuard`, choice window.
- `app/window.md`: ventana sin decoraciones, `native_options`, título, acciones de
  ventana, bordes de redimensionado.
- `app/settings-persistence.md`: cómo persiste el template sus ajustes.
- `app/i18n.md`: traducción en el punto de llamada, `…`, pseudo-locale, +40 % de texto.

**6. Recipes** (casos prácticos completos, cada uno con código y captura)
- Editor de texto con explorador de ficheros y pestañas.
- Panel de datos con gráfico en vivo, selección y exportación.
- Visor de imágenes/iconos con `IconGrid`.
- Diálogo de ajustes propio con secciones.
- Notificaciones de tareas largas (`Toasts::loading` → `update`).
- Marca propia: acento, densidad y esquinas persistidos en ajustes (resume
  `templates/app/src/appearance.rs` y la pestaña Settings → Shape).
- Layout tipo IDE con explorador, inspector, terminal/salida y barra de estado.
- Responsive: cómo diseñar contenido que salta de línea y pasa a una columna
  (patrones usados en la galería: `horizontal_wrapped` + `set_row_height`,
  dos columnas solo si `available_width() >= 620`).

**7. Reference**
- `reference/api-map.md`: tabla de toda la API pública agrupada por módulo
  (sale de `src/lib.rs`, `components/mod.rs`, `components/plot/mod.rs`, `app/mod.rs`).
- `reference/design-rules.md`: resumen de `WORKBENCH.md` §12 y §16 (checklist).
- `reference/testing.md`: `egui_kittest`, harness con `with_step_dt`, snapshots,
  `UPDATE_SNAPSHOTS=1`, E2E bajo Xvfb, `cargo xtask check`.
- `reference/xtask.md`: `new`, `check`, `snapshots`, `template-check`, `fmt`.
- `reference/changelog.md`: incluye `CHANGELOG.md`.
- `reference/faq.md`: errores típicos (doble copia de egui sin el parche, glifos que
  faltan en la fuente como `→`, filas que no saltan de línea por usar `ui.scope`,
  sliders de egui sobre superficies, repintado continuo).

## Capturas ("cómo se ve en la app")

- Usa primero las existentes en `tests/snapshots/` (componentes, workbench, paleta,
  ajustes, menú contextual, gráficos).
- Para lo que falte, añade tests de captura con `egui_kittest` siguiendo el patrón de
  `tests/snapshots_components.rs` y `tests/snapshots_plot.rs`: harness con
  `with_size`, `with_context_setup(|ctx| theme::apply_id(ctx, ThemeId::…))`,
  `harness.snapshot("nombre")`. Genera cada captura en **GitHub Light y GitHub Dark**,
  y guárdalas en `docs-site/public/screenshots/<sección>/<nombre>-{light,dark}.png`.
- Para la app completa, el template se puede renderizar igual: crea un `App` con
  `App::with_context`, llama a `open_section(GallerySection::…)` y dibuja con
  `show_workbench` dentro del harness.
- `Preview.vue` debe cambiar entre la captura clara y la oscura según el tema del sitio.
- Cada captura tiene texto alternativo descriptivo.

## Requisitos de calidad

- Ninguna página sin al menos un ejemplo de código y, si es visual, una captura.
- Bloques de código con título de fichero (`` ```rust [src/app.rs] ``) y resaltado de
  líneas cuando expliques una parte concreta.
- Usa `::: tip`, `::: warning` y `::: details` de VitePress para notas, trampas y
  código largo.
- Enlaza cada tipo mencionado a su página o a `reference/api-map.md`.
- Mantén la terminología de la librería (`Command`, `token`, `builder`, `tone`,
  `frame`, `view`).
- `npm run build` debe terminar sin enlaces rotos (`ignoreDeadLinks: false`).
- Añade un script `docs-site/scripts/check-examples.sh` que extraiga los bloques
  ```` ```rust ```` marcados como `<!-- compile -->` a un crate temporal y ejecute
  `cargo check`, para que los ejemplos no se pudran.

## Entregables

1. El proyecto `docs-site/` completo y construible (`npm install && npm run build`).
2. Los tests de captura nuevos (si los añades) y las imágenes en `public/screenshots/`.
3. Un `docs-site/README.md` con cómo ejecutar el sitio, regenerar capturas y comprobar
   los ejemplos.
4. Una lista final de lo que no pudiste verificar o documentar, con el motivo.
