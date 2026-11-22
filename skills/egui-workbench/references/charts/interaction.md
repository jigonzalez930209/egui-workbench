# Interaction

> Pan, wheel zoom, box zoom, area selection, hover popups, legend toggles, tools, and annotations. Live page: <https://jigonzalez930209.github.io/egui-workbench/charts/interaction.html>

## Pointer gestures

| Gesture | Action |
|---|---|
| Drag with the pan tool | Move the view |
| Wheel | Zoom around the cursor; Shift = x only, Alt = y only |
| Right-drag | Zoom to a box |
| Drag with the zoom tool | Zoom to a box; a thin box zooms one axis |
| Drag with the select tool | Select an area |
| Shift-drag | Select an area regardless of the active tool |
| Double-click | Fit the data |
| Click | Report the nearest point within 12 pt as `clicked` and clear the selection |
| Hover | Ring, dashed guides, and a popup with series/x/y |

The wheel is consumed by the plot, so a surrounding scroll area stays put while the
cursor is over the frame.

## Tools

`PlotState::tool` is `PlotTool::{Pan, Zoom, Select}`. The toolbar in the top-right corner
switches tools, zooms in/out, and fits the data. `Plot::toolbar(false)` hides it.

```rust
let mut state = PlotState::default();
state.tool = PlotTool::Select;
```

## Selection

```rust
let response = plot.show(ui, &mut icons, &mut state);
if response.selection_changed {
    if let Some(bounds) = state.selection {
        let inside = points_in(&data, &bounds);
        self.selected = inside;
    }
}
```

`PlotState::selection` is in data coordinates. A click clears it and sets
`selection_changed`. The selection is painted with the selection color from the theme or
`PlotConfig::colors.selection`.

## Hover

`PlotResponse::hovered` holds the nearest point within 12 pt:

```rust
pub struct PlotHover {
    pub series: String,
    pub index: usize,
    pub point: [f64; 2],
}
```

The popup and the dashed guides can be disabled with `PlotConfig::hover_popup` and
`hover_guides`. `PlotResponse::pointer` is the cursor in data coordinates while it is
inside the frame.

## Response

```rust
pub struct PlotResponse {
    pub response: egui::Response,
    pub pointer: Option<[f64; 2]>,
    pub hovered: Option<PlotHover>,
    pub clicked: Option<PlotHover>,
    pub selection_changed: bool,
    pub view: Bounds,
    pub frame: egui::Rect,
    pub settings_requested: bool,
    pub export_requested: Option<ExportFormat>,
    // private transform
}
```

`screen_pos(point)` and `data_pos(pos)` convert for overlays. `view` is the visible range
in data coordinates.

## Annotations

Draw on top of the plot after it returns, using the response transform:

```rust
let response = plot.show(ui, &mut icons, &mut state);
if let Some(peak) = data.iter().max_by(|a, b| a[1].total_cmp(&b[1])) {
    let pos = response.screen_pos(*peak);
    ui.painter().circle_stroke(pos, 5.0, Stroke::new(1.5, theme.focus));
    ui.painter().text(
        pos + egui::vec2(8.0, -8.0),
        egui::Align2::LEFT_BOTTOM,
        "peak",
        egui_workbench::style::get(ui.ctx()).font(11.0), // follows the text scale
        ui.visuals().text_color(),
    );
}
```

This works because the plot does not repaint the whole UI; the annotation is painted in
the same frame, in the same paint layer, after the plot's own shapes.

## Legend

Clicking a legend row toggles that series. `PlotState::is_hidden(name)` and
`set_hidden(name, hidden)` read and write the state; the application can persist it.
`PlotConfig::legend` chooses the corner or hides the legend; `legend_frame` controls its
background.

## Focus and keyboard

The plot is a pointer-first component. The toolbar buttons are real `IconButton`s and
therefore keyboard accessible. If your screen needs keyboard shortcuts for zoom or
selection, bind them in the application keymap and mutate `PlotState` before `show`.

## Repaint behavior

The plot requests repaint only while a drag, an animation, or live-data streaming is
active. A still pointer on a still plot costs no frames.
