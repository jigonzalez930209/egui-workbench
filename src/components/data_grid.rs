//! A typed, virtualized data grid with sort, filter, search, selection, and copy.

use std::collections::BTreeSet;

use egui::{Align2, RichText, ScrollArea, Sense, Ui};

use crate::components::Component;
use crate::data::{CellValue, Dataset, LinkHub, LinkMode, LinkSelection, Query};

/// Sort direction of a column.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[non_exhaustive]
pub enum SortDirection {
    /// Smallest first.
    #[default]
    Ascending,
    /// Largest first.
    Descending,
}

/// Persistent grid state: sort, filter, selection, column widths, and hidden columns.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DataGridState {
    /// Sorted column and direction.
    pub sort: Option<(usize, SortDirection)>,
    /// The query-language filter.
    pub query: String,
    /// Quick search over every text cell.
    pub search: String,
    /// Selected rows.
    pub selected_rows: BTreeSet<usize>,
    /// Selected cell.
    pub selected_cell: Option<(usize, usize)>,
    /// Widths by column; missing entries use 120 pt.
    pub widths: Vec<f32>,
    /// Hidden columns.
    pub hidden: BTreeSet<usize>,
    /// A row to scroll to.
    pub scroll_to_row: Option<usize>,
}

impl DataGridState {
    /// The width of a column.
    pub fn width(&self, column: usize) -> f32 {
        self.widths.get(column).copied().unwrap_or(120.0)
    }

    /// Toggle a column's visibility.
    pub fn toggle_column(&mut self, column: usize) {
        if !self.hidden.remove(&column) {
            self.hidden.insert(column);
        }
    }

    /// The visible row indices after the query, search, and sort.
    ///
    /// # Errors
    /// When `query` is not a valid expression.
    pub fn visible_rows(&self, data: &Dataset) -> Result<Vec<usize>, crate::data::QueryError> {
        let mut rows: Vec<usize> = if self.query.trim().is_empty() {
            (0..data.row_count()).collect()
        } else {
            Query::parse(&self.query)?.matching_rows(data)
        };
        if !self.search.is_empty() {
            let needle = self.search.to_lowercase();
            rows.retain(|row| {
                data.columns().iter().enumerate().any(|(column, _)| {
                    !self.hidden.contains(&column) && data.cell(*row, column).to_text().to_lowercase().contains(&needle)
                })
            });
        }
        if let Some((column, direction)) = self.sort {
            rows.sort_by(|left, right| {
                let ordering = compare_cells(data.cell(*left, column), data.cell(*right, column));
                if direction == SortDirection::Descending {
                    ordering.reverse()
                } else {
                    ordering
                }
            });
        }
        Ok(rows)
    }

    /// Copy the selected rows (or all visible rows) as TSV.
    pub fn copy_tsv(&self, data: &Dataset) -> String {
        let rows = self.copy_rows(data);
        let columns: Vec<usize> = (0..data.width())
            .filter(|column| !self.hidden.contains(column))
            .collect();
        let mut text = String::new();
        text.push_str(
            &columns
                .iter()
                .map(|column| data.column(*column).map_or("", |column| column.name.as_str()))
                .collect::<Vec<_>>()
                .join("\t"),
        );
        text.push('\n');
        for row in rows {
            text.push_str(
                &columns
                    .iter()
                    .map(|column| data.cell(row, *column).to_text())
                    .collect::<Vec<_>>()
                    .join("\t"),
            );
            text.push('\n');
        }
        text
    }

    /// Copy the selected rows (or all visible rows) as a Markdown table.
    pub fn copy_markdown(&self, data: &Dataset) -> String {
        let rows = self.copy_rows(data);
        let columns: Vec<usize> = (0..data.width())
            .filter(|column| !self.hidden.contains(column))
            .collect();
        let mut text = String::new();
        text.push('|');
        for column in &columns {
            text.push_str(&format!(
                " {} |",
                data.column(*column).map_or("", |column| column.name.as_str())
            ));
        }
        text.push_str("\n|");
        for _ in &columns {
            text.push_str(" --- |");
        }
        text.push('\n');
        for row in rows {
            text.push('|');
            for column in &columns {
                text.push_str(&format!(" {} |", data.cell(row, *column).to_text()));
            }
            text.push('\n');
        }
        text
    }

    /// Copy the selected rows (or all visible rows) as a JSON array.
    ///
    /// # Errors
    /// When the selected values cannot be serialized.
    #[cfg(feature = "serde")]
    pub fn copy_json(&self, data: &Dataset) -> Result<String, String> {
        let rows = self.copy_rows(data);
        let values: Vec<serde_json::Value> = rows
            .into_iter()
            .map(|row| {
                let mut object = serde_json::Map::new();
                for (column, definition) in data.columns().iter().enumerate() {
                    if self.hidden.contains(&column) {
                        continue;
                    }
                    object.insert(definition.name.clone(), cell_json(data.cell(row, column)));
                }
                serde_json::Value::Object(object)
            })
            .collect();
        serde_json::to_string_pretty(&values).map_err(|error| error.to_string())
    }

    fn copy_rows(&self, data: &Dataset) -> Vec<usize> {
        if !self.selected_rows.is_empty() {
            self.selected_rows.iter().copied().collect()
        } else {
            self.visible_rows(data).unwrap_or_default()
        }
    }
}

/// One user action from the grid.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum DataGridEvent {
    /// The sort column or direction changed.
    Sorted,
    /// The selection changed.
    Selection,
    /// Copy text in the given format.
    Copy {
        /// The text.
        text: String,
        /// A label for a toast.
        format: &'static str,
    },
}

/// A typed grid over a [`Dataset`].
pub struct DataGrid<'a> {
    data: &'a Dataset,
    state: &'a mut DataGridState,
    height: f32,
    row_height: f32,
    linked: Option<&'a LinkSelection>,
    linked_mode: LinkMode,
}

impl<'a> DataGrid<'a> {
    /// A grid with a 320 pt viewport and 22 pt rows.
    pub fn new(data: &'a Dataset, state: &'a mut DataGridState) -> Self {
        Self {
            data,
            state,
            height: 320.0,
            row_height: 22.0,
            linked: None,
            linked_mode: LinkMode::Highlight,
        }
    }

    /// Follow a selection bus.
    pub fn link(mut self, hub: &'a LinkHub) -> Self {
        self.linked = Some(hub.selection());
        self.linked_mode = hub.mode();
        self
    }

    /// Set the viewport height.
    pub fn height(mut self, height: f32) -> Self {
        self.height = height.max(80.0);
        self
    }

    /// Set the row height.
    pub fn row_height(mut self, row_height: f32) -> Self {
        self.row_height = row_height.max(14.0);
        self
    }

    /// Paint the grid.
    pub fn show(self, ui: &mut Ui) -> Option<DataGridEvent> {
        let mut event = None;
        self.state.widths.resize(self.data.width(), 120.0);
        ui.horizontal(|ui| {
            if ui.button("Copy TSV").clicked() {
                event = Some(DataGridEvent::Copy {
                    text: self.state.copy_tsv(self.data),
                    format: "TSV",
                });
            }
            if ui.button("Copy Markdown").clicked() {
                event = Some(DataGridEvent::Copy {
                    text: self.state.copy_markdown(self.data),
                    format: "Markdown",
                });
            }
            #[cfg(feature = "serde")]
            if ui.button("Copy JSON").clicked()
                && let Ok(text) = self.state.copy_json(self.data)
            {
                event = Some(DataGridEvent::Copy { text, format: "JSON" });
            }
            ui.add(egui::TextEdit::singleline(&mut self.state.query).hint_text("voltage > 0.3 && cycle in 2..5"));
            ui.add(egui::TextEdit::singleline(&mut self.state.search).hint_text("Search…"));
            if let Err(error) = self.state.visible_rows(self.data) {
                ui.label(RichText::new(error.to_string()).color(ui.visuals().error_fg_color));
            }
        });
        ui.add_space(4.0);
        let rows = self.state.visible_rows(self.data).unwrap_or_default();
        let columns: Vec<usize> = (0..self.data.width())
            .filter(|column| !self.state.hidden.contains(column))
            .collect();
        let total_width: f32 = columns.iter().map(|column| self.state.width(*column)).sum();
        ScrollArea::both()
            .max_height(self.height)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    for column in &columns {
                        let width = self.state.width(*column);
                        let (rect, response) = ui.allocate_exact_size(egui::vec2(width, 24.0), Sense::click());
                        response.widget_info(|| {
                            egui::WidgetInfo::labeled(
                                egui::Role::ColumnHeader,
                                true,
                                self.data
                                    .column(*column)
                                    .map_or("", |definition| definition.name.as_str()),
                            )
                        });
                        let active = self.state.sort.is_some_and(|(active, _)| active == *column);
                        if response.clicked() {
                            self.state.sort = match self.state.sort {
                                Some((active, SortDirection::Ascending)) if active == *column => {
                                    Some((*column, SortDirection::Descending))
                                }
                                _ => Some((*column, SortDirection::Ascending)),
                            };
                            event = Some(DataGridEvent::Sorted);
                        }
                        ui.painter().rect_filled(
                            rect,
                            crate::style::get(ui.ctx()).inner(),
                            if active {
                                ui.visuals().selection.bg_fill
                            } else {
                                ui.visuals().widgets.inactive.weak_bg_fill
                            },
                        );
                        ui.painter().text(
                            egui::pos2(rect.left() + 6.0, rect.center().y),
                            Align2::LEFT_CENTER,
                            self.data
                                .column(*column)
                                .map_or("", |definition| definition.name.as_str()),
                            crate::style::get(ui.ctx()).font(11.5),
                            ui.visuals().text_color(),
                        );
                        if let Some(definition) = self.data.column(*column)
                            && let Some(stats) = self.data.stats(*column)
                        {
                            response.on_hover_text(format!(
                                "{} · {} · {} rows · min {:?} · max {:?}",
                                definition.kind().label(),
                                definition.unit.as_deref().unwrap_or("no unit"),
                                stats.count,
                                stats.min,
                                stats.max
                            ));
                        }
                    }
                });
                ui.separator();
                ScrollArea::vertical()
                    .id_salt("data_grid_rows")
                    .max_height(self.height - 32.0)
                    .auto_shrink([false, false])
                    .show_rows(ui, self.row_height, rows.len(), |ui, range| {
                        for index in range {
                            let row = rows[index];
                            let selected = self.state.selected_rows.contains(&row)
                                || self.linked.is_some_and(|selection| selection.contains(row));
                            let (rect, response) = ui.allocate_exact_size(
                                egui::vec2(total_width.max(ui.available_width()), self.row_height),
                                Sense::click(),
                            );
                            response.widget_info(|| {
                                egui::WidgetInfo::selected(egui::Role::Row, true, selected, format!("Row {row}"))
                            });
                            if selected {
                                ui.painter()
                                    .rect_filled(rect, 0.0, ui.visuals().selection.bg_fill.gamma_multiply(0.5));
                            } else if response.hovered() {
                                ui.painter()
                                    .rect_filled(rect, 0.0, ui.visuals().widgets.hovered.weak_bg_fill);
                            }
                            let mut x = rect.left();
                            for column in &columns {
                                let width = self.state.width(*column);
                                let cell = self.data.cell(row, *column);
                                let selected_cell = self.state.selected_cell == Some((row, *column));
                                let cell_rect = egui::Rect::from_min_size(
                                    egui::pos2(x, rect.top()),
                                    egui::vec2(width, rect.height()),
                                );
                                if selected_cell {
                                    ui.painter().rect_stroke(
                                        cell_rect,
                                        0.0,
                                        ui.visuals().selection.stroke,
                                        egui::StrokeKind::Inside,
                                    );
                                }
                                let color = match cell {
                                    CellValue::Null => ui.visuals().weak_text_color(),
                                    CellValue::Float(_) | CellValue::Int(_) => ui.visuals().text_color(),
                                    CellValue::Bool(_) => ui.visuals().hyperlink_color,
                                    CellValue::Date(_) | CellValue::Time(_) => ui.visuals().warn_fg_color,
                                    CellValue::Text(_) | CellValue::Category(_) => ui.visuals().text_color(),
                                };
                                ui.painter().text(
                                    egui::pos2(x + 6.0, rect.center().y),
                                    Align2::LEFT_CENTER,
                                    cell.to_text(),
                                    crate::style::get(ui.ctx()).font(11.0),
                                    color,
                                );
                                x += width;
                            }
                            if response.clicked() {
                                self.state.selected_rows.clear();
                                self.state.selected_rows.insert(row);
                                self.state.selected_cell = Some((row, columns.first().copied().unwrap_or(0)));
                                event = Some(DataGridEvent::Selection);
                            }
                        }
                    });
            });
        event
    }
}

impl Component for DataGrid<'_> {
    type Output = Option<DataGridEvent>;

    fn show(self, ui: &mut Ui) -> Self::Output {
        self.show(ui)
    }
}

fn compare_cells(left: CellValue<'_>, right: CellValue<'_>) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    match (left, right) {
        (CellValue::Null, CellValue::Null) => Ordering::Equal,
        (CellValue::Null, _) => Ordering::Greater,
        (_, CellValue::Null) => Ordering::Less,
        (CellValue::Float(left), CellValue::Float(right)) => left.partial_cmp(&right).unwrap_or(Ordering::Equal),
        (CellValue::Int(left), CellValue::Int(right)) => left.cmp(&right),
        (CellValue::Bool(left), CellValue::Bool(right)) => left.cmp(&right),
        (CellValue::Text(left), CellValue::Text(right)) | (CellValue::Category(left), CellValue::Category(right)) => {
            left.cmp(right)
        }
        (CellValue::Date(left), CellValue::Date(right)) => left.cmp(&right),
        (CellValue::Time(left), CellValue::Time(right)) => left.cmp(&right),
        (left, right) => left.to_text().cmp(&right.to_text()),
    }
}

#[cfg(feature = "serde")]
fn cell_json(cell: CellValue<'_>) -> serde_json::Value {
    match cell {
        CellValue::Null => serde_json::Value::Null,
        CellValue::Float(value) => serde_json::Value::from(value),
        CellValue::Int(value) => serde_json::Value::from(value),
        CellValue::Bool(value) => serde_json::Value::from(value),
        CellValue::Text(value) | CellValue::Category(value) => serde_json::Value::from(value),
        CellValue::Date(value) => serde_json::Value::from(value),
        CellValue::Time(value) => serde_json::Value::from(value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::Column;

    fn data() -> Dataset {
        Dataset::from_columns([
            Column::floats("v", vec![0.4, 0.1, 0.3]),
            Column::text("name", ["b", "a", "c"]),
        ])
    }

    #[test]
    fn sorts_filters_and_copies() {
        let data = data();
        let mut state = DataGridState {
            sort: Some((0, SortDirection::Ascending)),
            ..DataGridState::default()
        };
        assert_eq!(state.visible_rows(&data).expect("rows"), [1, 2, 0]);
        state.query = "v > 0.2".to_owned();
        assert_eq!(state.visible_rows(&data).expect("rows"), [2, 0]);
        state.search = "c".to_owned();
        assert_eq!(state.visible_rows(&data).expect("rows"), [2]);
        state.search.clear();
        let tsv = state.copy_tsv(&data);
        assert!(tsv.starts_with("v\tname\n"));
        assert!(tsv.contains("0.3\tc"));
    }

    #[test]
    fn hidden_columns_are_not_copied() {
        let data = data();
        let mut state = DataGridState::default();
        state.hidden.insert(1);
        let markdown = state.copy_markdown(&data);
        assert!(!markdown.contains("name"));
        assert!(markdown.contains("0.4"));
    }

    #[test]
    fn a_bar_selection_copies_only_selected_rows() {
        let data = data();
        let mut state = DataGridState::default();
        state.selected_rows.insert(1);
        assert!(state.copy_tsv(&data).contains("0.1\ta"));
        assert!(!state.copy_tsv(&data).contains("0.4\tb"));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn json_copy_is_an_array_of_objects() {
        let data = data();
        let state = DataGridState::default();
        let json = state.copy_json(&data).expect("json");
        assert!(json.contains("\"v\": 0.4"));
        assert!(json.contains("\"name\": \"b\""));
    }
}
