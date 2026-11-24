//! The command palette and quick open.
//!
//! A 560 pt modal: a 15 pt bold title with "×", a focused filter field, and up to 12
//! matching 26 pt rows in a 300 pt scroll area. The first match is highlighted with
//! the selection fill and runs on Enter; a click runs any row. Esc or a backdrop click
//! closes it. Matching is a case-insensitive substring test on label and detail.

use egui::epaint::Shadow;
use egui::{Color32, Context, Frame, Id, Key, Modal, RichText, ScrollArea, Sense};

use crate::{close_glyph, text_field};

/// One palette row.
#[derive(Clone, Debug, PartialEq)]
pub struct PaletteItem<T> {
    /// Display label.
    pub label: String,
    /// Secondary text, shown after the label (a shortcut or a path).
    pub detail: String,
    /// Value returned when the row runs.
    pub value: T,
}

/// The prefix mode of a palette query.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PaletteMode {
    /// No prefix: commands, files, and every other result.
    #[default]
    All,
    /// `>`: commands.
    Commands,
    /// `:`: go to a line.
    Lines,
    /// `@`: symbols.
    Symbols,
}

impl PaletteMode {
    /// The prefix that selects this mode, or an empty string.
    pub fn prefix(self) -> &'static str {
        match self {
            Self::All => "",
            Self::Commands => ">",
            Self::Lines => ":",
            Self::Symbols => "@",
        }
    }

    /// Parse the prefix of a query.
    pub fn parse(query: &str) -> (Self, &str) {
        if let Some(rest) = query.strip_prefix('>') {
            (Self::Commands, rest)
        } else if let Some(rest) = query.strip_prefix(':') {
            (Self::Lines, rest)
        } else if let Some(rest) = query.strip_prefix('@') {
            (Self::Symbols, rest)
        } else {
            (Self::All, query)
        }
    }
}

/// Persistent palette state.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Default)]
pub struct PaletteState {
    /// Whether the palette is visible.
    pub open: bool,
    /// The filter text, including its `>`/`:`/`@` prefix.
    pub query: String,
    /// Recently chosen labels, newest first.
    #[cfg_attr(feature = "serde", serde(default))]
    pub recent: Vec<String>,
}

impl PaletteState {
    /// Open with an empty filter.
    pub fn open(&mut self) {
        self.open = true;
        self.query.clear();
    }

    /// The active prefix mode and the text after it.
    pub fn mode(&self) -> (PaletteMode, &str) {
        PaletteMode::parse(&self.query)
    }

    /// Record a chosen row, keeping the newest twelve.
    pub fn record(&mut self, label: impl Into<String>) {
        let label = label.into();
        self.recent.retain(|recent| recent != &label);
        self.recent.insert(0, label);
        self.recent.truncate(12);
    }

    /// Whether `label` is in the recent list.
    pub fn is_recent(&self, label: &str) -> bool {
        self.recent.iter().any(|recent| recent == label)
    }
}

/// Translated strings of the palette.
#[derive(Clone, Copy, Debug)]
pub struct PaletteLabels<'a> {
    /// Title, such as "Command Palette" or "Quick Open".
    pub title: &'a str,
    /// Filter hint.
    pub hint: &'a str,
    /// Shown when nothing matches.
    pub empty: &'a str,
    /// Close tooltip.
    pub close: &'a str,
}

/// Items whose label or detail contains the query, at most 12.
pub fn filter<'a, T>(items: &'a [PaletteItem<T>], query: &str) -> Vec<&'a PaletteItem<T>> {
    let needle = query.to_lowercase();
    items
        .iter()
        .filter(|item| {
            needle.is_empty()
                || item.label.to_lowercase().contains(&needle)
                || item.detail.to_lowercase().contains(&needle)
        })
        .take(12)
        .collect()
}

/// Items ranked by a fuzzy score and capped at 12.
///
/// The `>`/`:`/`@` prefix is stripped before matching; use [`PaletteState::mode`] to
/// decide which provider supplies the rows. Label matches rank above detail matches.
/// Pass `recent` to put the commands the user just ran first.
pub fn fuzzy_filter<'a, T>(items: &'a [PaletteItem<T>], query: &str, recent: &[String]) -> Vec<&'a PaletteItem<T>> {
    let (_, needle) = PaletteMode::parse(query);
    let needle = needle.trim().to_lowercase();
    let mut ranked: Vec<(i32, usize, &'a PaletteItem<T>)> = items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| {
            if needle.is_empty() {
                let recent_rank = recent
                    .iter()
                    .position(|label| label == &item.label)
                    .map_or(i32::MIN, |position| 1000 - position as i32);
                return Some((recent_rank, index, item));
            }
            let score = score(&item.label, &needle).max(score(&item.detail, &needle) - 200);
            (score > 0).then_some((score, index, item))
        })
        .collect();
    ranked.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| left.1.cmp(&right.1)));
    ranked.into_iter().take(12).map(|(_, _, item)| item).collect()
}

fn score(text: &str, needle: &str) -> i32 {
    let text = text.to_lowercase();
    if text == needle {
        return 1000;
    }
    if text.starts_with(needle) {
        return 850 - text.len().min(100) as i32;
    }
    if let Some(position) = text.find(needle) {
        return 700 - position.min(200) as i32;
    }
    // Subsequence: every needle character in order, with a penalty for each gap.
    let mut position = 0;
    let mut gap = 0;
    for character in needle.chars() {
        let Some(found) = text[position..].find(character) else {
            return 0;
        };
        gap += found;
        position += found + character.len_utf8();
    }
    (400 - gap.min(200) as i32).max(1)
}

/// Results arriving from a background provider without blocking the frame.
///
/// The application owns the sender and requests work when the query changes; call
/// [`Self::poll`] once per frame and pass [`Self::items`] to the palette.
pub struct PaletteFeed<T> {
    receiver: Option<std::sync::mpsc::Receiver<Vec<PaletteItem<T>>>>,
    items: Vec<PaletteItem<T>>,
    pending: bool,
}

impl<T> Default for PaletteFeed<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> PaletteFeed<T> {
    /// An empty feed.
    pub fn new() -> Self {
        Self {
            receiver: None,
            items: Vec::new(),
            pending: false,
        }
    }

    /// Attach a channel. The previous results stay visible until the new ones arrive.
    pub fn submit(&mut self, receiver: std::sync::mpsc::Receiver<Vec<PaletteItem<T>>>) {
        self.receiver = Some(receiver);
        self.pending = true;
    }

    /// Replace the results immediately (for a synchronous provider).
    pub fn set_items(&mut self, items: Vec<PaletteItem<T>>) {
        self.items = items;
        self.pending = false;
        self.receiver = None;
    }

    /// Take a finished result set without blocking. Returns whether the rows changed.
    pub fn poll(&mut self) -> bool {
        let Some(receiver) = &self.receiver else {
            return false;
        };
        match receiver.try_recv() {
            Ok(items) => {
                self.items = items;
                self.pending = false;
                self.receiver = None;
                true
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => false,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.pending = false;
                self.receiver = None;
                false
            }
        }
    }

    /// The current rows.
    pub fn items(&self) -> &[PaletteItem<T>] {
        &self.items
    }

    /// Whether a background request is still running.
    pub fn is_pending(&self) -> bool {
        self.pending
    }
}

/// Show the palette and return the value of the row that ran.
pub fn show_palette<T: Clone>(
    ctx: &Context,
    state: &mut PaletteState,
    labels: PaletteLabels<'_>,
    items: &[PaletteItem<T>],
) -> Option<T> {
    if !state.open {
        return None;
    }
    let mut chosen = None;
    let mut keep_open = true;
    let visuals = ctx.style_of(ctx.theme()).visuals.clone();
    let frame = Frame::new()
        .fill(visuals.window_fill)
        .stroke(crate::style::get(ctx).border(visuals.window_stroke.color))
        .corner_radius(crate::style::get(ctx).dialog())
        .inner_margin(egui::Margin::same(14))
        .shadow(Shadow {
            offset: [0, 8],
            blur: 24,
            spread: 0,
            color: Color32::from_black_alpha(60),
        });
    let dialog = Modal::new(Id::unique("workbench_palette"))
        .frame(frame)
        .show(ctx, |ui| {
            ui.set_width(560.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(labels.title)
                        .size(crate::style::get(ui.ctx()).font_size(15.0))
                        .strong()
                        .color(ui.visuals().text_color()),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if close_glyph(ui, 14.0, labels.close) {
                        keep_open = false;
                    }
                });
            });
            ui.add_space(8.0);
            // The field spans the same width as the result rows below it.
            text_field(ui, &mut state.query, labels.hint, ui.available_width()).request_focus();
            ui.add_space(8.0);
            let matches = fuzzy_filter(items, &state.query, &state.recent);
            ScrollArea::vertical()
                .max_height(300.0)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    if matches.is_empty() {
                        ui.label(RichText::new(labels.empty).weak());
                    }
                    for (index, item) in matches.iter().enumerate() {
                        let (rect, response) = ui.allocate_exact_size(
                            egui::vec2(ui.available_width(), crate::style::get(ui.ctx()).list_row_height()),
                            Sense::click(),
                        );
                        response.widget_info(|| {
                            egui::WidgetInfo::selected(egui::Role::ListBoxOption, true, index == 0, &item.label)
                        });
                        if index == 0 {
                            ui.painter().rect_filled(
                                rect,
                                crate::style::get(ui.ctx()).item(),
                                ui.visuals().selection.bg_fill,
                            );
                        } else if response.hovered() {
                            ui.painter().rect_filled(
                                rect,
                                crate::style::get(ui.ctx()).item(),
                                ui.visuals().widgets.hovered.weak_bg_fill,
                            );
                        }
                        // Label on the left, detail (usually the shortcut) right-aligned and
                        // muted, like the menus.
                        ui.painter().text(
                            egui::pos2(rect.left() + 10.0, rect.center().y),
                            egui::Align2::LEFT_CENTER,
                            &item.label,
                            crate::style::get(ui.ctx()).font(12.5),
                            ui.visuals().text_color(),
                        );
                        if !item.detail.is_empty() {
                            ui.painter().text(
                                egui::pos2(rect.right() - 10.0, rect.center().y),
                                egui::Align2::RIGHT_CENTER,
                                &item.detail,
                                crate::style::get(ui.ctx()).font(11.5),
                                ui.visuals().weak_text_color(),
                            );
                        }
                        if response.clicked() {
                            state.record(item.label.clone());
                            chosen = Some(item.value.clone());
                            keep_open = false;
                        }
                    }
                });
            if ctx.input(|input| input.key_pressed(Key::Enter))
                && let Some(item) = matches.first()
            {
                state.record(item.label.clone());
                chosen = Some(item.value.clone());
                keep_open = false;
            }
        });
    if dialog.backdrop_response.clicked() || ctx.input(|input| input.key_pressed(Key::Escape)) {
        keep_open = false;
    }
    state.open = keep_open;
    chosen
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuzzy_ranking_prefers_exact_and_prefix_matches() {
        let item = |label: &str, detail: &str| PaletteItem {
            label: label.to_owned(),
            detail: detail.to_owned(),
            value: label.to_owned(),
        };
        let items = vec![
            item("Save As…", "Ctrl+Shift+S"),
            item("Save", "Ctrl+S"),
            item("Search", "Ctrl+F"),
            item("Recent file", "save.rs"),
        ];
        let matches = fuzzy_filter(&items, "save", &[]);
        assert_eq!(matches[0].label, "Save");
        assert_eq!(matches[1].label, "Save As…");
        assert!(matches.iter().any(|item| item.label == "Recent file"));
    }

    #[test]
    fn recent_labels_come_first_and_modes_strip_the_prefix() {
        let items: Vec<PaletteItem<usize>> = ["Open", "Save", "Close"]
            .into_iter()
            .enumerate()
            .map(|(index, label)| PaletteItem {
                label: label.to_owned(),
                detail: String::new(),
                value: index,
            })
            .collect();
        let matches = fuzzy_filter(&items, "", &["Close".to_owned()]);
        assert_eq!(matches[0].label, "Close");
        let matches = fuzzy_filter(&items, ">save", &[]);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].label, "Save");
        let (mode, query) = PaletteMode::parse("@main");
        assert_eq!(mode, PaletteMode::Symbols);
        assert_eq!(query, "main");
    }

    #[test]
    fn a_background_feed_never_blocks_the_frame() {
        let (sender, receiver) = std::sync::mpsc::channel();
        let mut feed = PaletteFeed::new();
        assert!(!feed.poll());
        feed.submit(receiver);
        assert!(feed.is_pending());
        assert!(!feed.poll());
        sender
            .send(vec![PaletteItem {
                label: "Save".to_owned(),
                detail: "Ctrl+S".to_owned(),
                value: 1,
            }])
            .expect("send");
        assert!(feed.poll());
        assert_eq!(feed.items().len(), 1);
        assert!(!feed.is_pending());
    }

    #[test]
    fn filtering_matches_label_and_detail_case_insensitively() {
        let item = |label: &str, detail: &str| PaletteItem {
            label: label.to_owned(),
            detail: detail.to_owned(),
            value: label.to_owned(),
        };
        let items = vec![
            item("Save", "Ctrl+S"),
            item("Open Folder…", "Ctrl+Shift+O"),
            item("Quit", ""),
        ];
        let labels = |query: &str| {
            filter(&items, query)
                .iter()
                .map(|item| item.label.clone())
                .collect::<Vec<_>>()
        };
        assert_eq!(labels("").len(), 3);
        assert_eq!(labels("folder"), ["Open Folder…"]);
        assert_eq!(labels("ctrl+s"), ["Save", "Open Folder…"]);
    }
}
