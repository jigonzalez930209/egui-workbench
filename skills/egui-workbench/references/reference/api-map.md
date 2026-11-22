# API map

> The public surface grouped by module, with links to the guide pages. Live page: <https://jigonzalez930209.github.io/egui-workbench/reference/api-map.html>

Everything below is re-exported by `egui_workbench` unless a module path is shown. The
crate root keeps the original function API; `components` adds builders.

## Application runtime — `egui_workbench::app`

| Item | Kind | Page |
|---|---|---|
| `ThemeState` | struct | [Themes](../theming/themes.md) |
| `Documents<T>`, `Document` | struct, trait | [Documents](../app/documents.md) |
| `Keymap<C>` | struct | [Commands](../app/commands-and-keymap.md) |
| `CloseGuard` | struct | [Documents](../app/documents.md) |
| `WorkbenchApp`, `show_workbench` | trait, function | [Workbench layout](../app/workbench.md) |
| `TITLE_BAR_PANEL`, `SIDEBAR_PANEL`, `RIGHT_PANEL`, `TAB_STRIP_PANEL`, `STATUS_BAR_PANEL`, `BOTTOM_PANEL` | `&str` ids | [Workbench layout](../app/workbench.md) |
| `SIDEBAR_MIN_WIDTH`, `CONTENT_MIN_WIDTH`, `RIGHT_PANEL_MIN_WIDTH`, `BOTTOM_PANEL_MIN_HEIGHT` | `f32` | [Workbench layout](../app/workbench.md) |

## Theme — `egui_workbench::theme` and root

| Item | Kind |
|---|---|
| `Theme`, `ThemeId`, `ThemeMode` | struct, enum, enum |
| `apply`, `apply_id`, `apply_theme`, `apply_tokens` | functions |
| `current`, `effective_id`, `system_is_dark`, `motion_enabled`, `set_reduced_motion` | functions |
| `ANIMATION_TIME` | `f32` |
| `mix`, `relative_luminance`, `readable_on`, `contrast_ratio` | functions |

`Theme` methods: `light`, `dark`, `for_id`, `is_dark`, `with_accent`,
`accessibility_issues`, `border_stroke`, and the deprecated `radius` (use
`style::get(ctx).control()`).

## Style — `egui_workbench::style`

| Item | Kind |
|---|---|
| `WorkbenchStyle` | struct |
| `Density` | enum |
| `get(ctx)`, `set(ctx, style)` | functions |
| `TYPE_SCALE` | `[(TextStyle, f32); 5]` |

`WorkbenchStyle` fields: `density`, `control_radius`, `item_radius`, `inner_radius`,
`container_radius`, `dialog_radius`, `pill_radius`, `border_width`, `text_scale`.
Methods: `sharp`, `rounded`, `with_density`, `scaled`, `control_height`, `size_height`,
`size_padding`, `menu_row_height`, `list_row_height`, `tree_row_height`, `control`,
`item`, `inner`, `tight`, `mark`, `container`, `dialog`, `pill`, `border`, `text_scale`,
`font_size`, `font`, `mono`, `text`.

`Density`: `Compact`, `Default`, `Comfortable`; `factor`, `ALL`, `as_setting`,
`from_setting`.

## Metrics — `egui_workbench::metrics`

`ICON_SMALL`, `ICON_MEDIUM`, `ICON_LARGE`, `TITLE_HEIGHT`, `TAB_HEIGHT`,
`TAB_STRIP_PADDING`, `STATUS_HEIGHT`, `TREE_ROW_HEIGHT`, `TREE_INDENT`, `CHEVRON_SIZE`,
`SIDEBAR_WIDTH`, `ACTION_BUTTON`, `TOOL_BUTTON`, `SMALL_BUTTON`, `WINDOW_BUTTON_WIDTH`,
`WINDOW_BUTTON_HEIGHT`, `CONTROL_HEIGHT`, `MENU_ROW_HEIGHT`, `LIST_ROW_HEIGHT`,
`RESIZE_EDGE`.

## Window — root

`handle_window_edges`, `apply_window_action`, `is_maximized`, `set_window_title`,
`WindowAction`. `native_options` and `native_options_sized` are native-only
(`#[cfg(not(target_arch = "wasm32"))]`); wasm uses `eframe::WebRunner` and
`eframe::WebOptions` (see [Live demo](https://jigonzalez930209.github.io/egui-workbench/guide/live-demo.html)).

## Title bar — root

`show_title_bar`, `TitleBar`, `TitleBarOutput`, `WindowLabels`, `Pill`, `title_menu`.

## Buttons — root

`action_button`, `IconState`, `tool_button`, `tool_button_response`, `menu_tool_button`,
`small_button`, `segmented`, `Segment`, `close_glyph`.

## Controls — root

`text_field`, `primary_button`, `secondary_button`, `numeric_stepper`, `switch`, `combo`,
`compact_menu`, `menu_item`, `menu_button`, `menu_toggle`.

## Tabs, tree, status — root

| Area | Items |
|---|---|
| Tabs | `show_tab_strip`, `TabItem`, `TabIcon`, `TabEvent` |
| Tree | `section_header`, `show_tree`, `TreeNode`, `TreeState`, `TreeView`, `TreeEvent`, `RowIcon` |
| Status | `show_status_bar`, `StatusItem`, `ellipsize_start`, `show_empty_state`, `editor_scroll_style`, `with_editor_scrollbars` |
| Split | `split_panes`, `STACK_BELOW` |

## Dialogs, settings, palette, keyboard menu — root

| Area | Items |
|---|---|
| Dialogs | `dialog_frame`, `dialog_header`, `dialog_footer`, `show_name_prompt`, `NamePrompt`, `show_confirm_prompt`, `ConfirmPrompt`, `PromptAction`, `show_choice_window`, `show_progress`, `banner`, `popover_frame`, `show_popover` |
| Settings | `show_settings_dialog`, `SettingsDialog`, `settings_section`, `settings_row`, `settings_switch` |
| Palette | `show_palette`, `PaletteItem<T>`, `PaletteState`, `PaletteLabels`, `filter_palette` |
| Keyboard menu | `show_keyboard_menu`, `open_from_keyboard`, `KeyboardMenuState`, `MenuGroup<T>`, `MenuEntry<T>` |

## Icons — root

| Item | Kind |
|---|---|
| `Icons` | struct |
| `MaterialIcons` | struct (feature `material-icons`) |
| `FULL_UV` | `Rect` |

`Icons`: `new`, `bundled`, `register`, `lucide_names`, `lucide_search`, `texture`,
`texture_sized`, `paint`, `put`, `show`, `show_tinted`, and the `material` field.
`MaterialIcons`: `file_icon`, `folder_icon`, `texture`, `texture_sized`, `draw`.

## Components — `egui_workbench::components`

| Item | Kind |
|---|---|
| `Button`, `IconButton`, `CloseButton` | builders |
| `TextField`, `TextArea`, `Switch`, `Checkbox`, `Select<T>`, `Stepper`, `SegmentedControl` | builders |
| `Label`, `Badge`, `Banner`, `Card`, `Separator`, `Popover`, `StatusBar`, `NavTab` | builders |
| `Menu`, `MenuButton`, `MenuItem`, `MenuToggle`, `MenuIconButton` | builders |
| `TabStrip`, `Tree`, `CommandPalette` | builders |
| `Dialog`, `ConfirmDialog`, `NameDialog`, `ProgressDialog` | builders |
| `IconGrid`, `IconPreview`, `IconSize`, `IconLayout` | builders/enums |
| `Toasts`, `Toast`, `ToastKind`, `ToastPosition`, `ToastId`, `ToastEvent` | toasts |
| `Size`, `ButtonVariant`, `IconButtonSize`, `Tone`, `Orientation` | shared values |
| `Component`, `IconComponent` | traits |
| `prelude` | the common imports |
| `blocks` | composite re-exports |
| `plot` | see below |

`components::prelude` exports: `Badge`, `Banner`, `Button`, `ButtonVariant`, `Card`,
`Checkbox`, `CloseButton`, `CommandPalette`, `Component`, `ConfirmDialog`, `Dialog`,
`IconButton`, `IconButtonSize`, `IconComponent`, `IconGrid`, `IconLayout`, `IconPreview`,
`IconSize`, `Label`, `Menu`, `MenuButton`, `MenuIconButton`, `MenuItem`, `MenuToggle`,
`NameDialog`, `NavTab`, `Orientation`, `PaletteItem`, `PaletteLabels`, `PaletteState`,
`Popover`, `ProgressDialog`, `PromptAction`, `RowIcon`, `SegmentedControl`, `Select`,
`Separator`, `Size`, `StatusBar`, `Stepper`, `Switch`, `TabEvent`, `TabIcon`, `TabItem`,
`TabStrip`, `TextArea`, `TextField`, `Toast`, `ToastEvent`, `ToastKind`, `ToastPosition`,
`Toasts`, `Tone`, `Tree`, `TreeEvent`, `TreeNode`, `TreeState`.

## Charts — `egui_workbench::components::plot`

| Item | Kind |
|---|---|
| `Plot` | builder |
| `PlotState`, `PlotTool`, `PlotHover`, `PlotResponse`, `LegendPosition` | state/response |
| `Series`, `Marker`, `LineStyle`, `points_in` | data |
| `Fit`, `FitKind` | analysis |
| `Bounds`, `Scale` | axes |
| `AxisConfig`, `Minor`, `PlotColors`, `PlotConfig`, `SeriesStyle` | configuration |
| `ExportOptions`, `ExportFormat` | export |

`Plot` methods: `new`, `config`, `current_config`, `series_names`, `x_limits`,
`y_limits`, `settings_button`, `export_button`, `series`, `fit`, `title`, `x_label`,
`y_label`, `x_scale`, `y_scale`, `width`, `height`, `legend`, `grid`, `toolbar`,
`x_window`, `include`, `show`, `to_svg`, `to_png`.

## Root re-exports

```rust
pub use eframe;
pub use egui;
pub use theme::{Theme, ThemeId, ThemeMode};
pub use foundation::{metrics, style, theme};
pub use foundation::icons::{FULL_UV, Icons};
#[cfg(feature = "material-icons")]
pub use foundation::material::MaterialIcons;
```

The crate has no dependency on your `Command` type. `PaletteItem<T>`, `MenuGroup<T>`,
`TreeNode<K>`, `Keymap<C>`, and `Documents<T>` are generic over it.
