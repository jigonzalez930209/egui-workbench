//! Transient notifications stacked in a window corner, like `sonner` in React.

use egui::{Area, Color32, Context, Id, Order, Pos2, pos2};

use crate::Icons;
use crate::theme;

mod view;

/// The meaning of a toast, which picks its icon and accent color.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToastKind {
    /// Neutral information (accent color, `info` icon).
    #[default]
    Info,
    /// A finished action (success color, check icon).
    Success,
    /// Something needs attention (warning color, triangle icon).
    Warning,
    /// A failure (danger color, cross icon).
    Error,
    /// Work in progress; a spinner and no timeout until it is updated.
    Loading,
}

impl ToastKind {
    pub(super) fn icon(self) -> &'static str {
        match self {
            Self::Info => "lucide-info",
            Self::Success => "lucide-circle-check",
            Self::Warning => "lucide-triangle-alert",
            Self::Error => "lucide-circle-x",
            Self::Loading => "lucide-loader-circle",
        }
    }

    pub(super) fn color(self, theme: &theme::Theme) -> Color32 {
        match self {
            Self::Info => theme.link,
            Self::Success => theme.success,
            Self::Warning => theme.warning,
            Self::Error => theme.danger,
            Self::Loading => theme.text_muted,
        }
    }
}

/// The corner (or edge center) the stack grows from.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToastPosition {
    /// Top left corner.
    TopLeft,
    /// Top edge, centered.
    TopCenter,
    /// Top right corner.
    TopRight,
    /// Bottom left corner.
    BottomLeft,
    /// Bottom edge, centered.
    BottomCenter,
    /// Bottom right corner, above the status bar.
    #[default]
    BottomRight,
}

impl ToastPosition {
    fn is_top(self) -> bool {
        matches!(self, Self::TopLeft | Self::TopCenter | Self::TopRight)
    }
}

/// The identity of a toast, for [`Toasts::update`] and [`Toasts::dismiss`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ToastId(pub(super) u64);

/// What happened to a toast this frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToastEvent {
    /// The action button was clicked; the toast closes.
    Action(ToastId),
    /// The close button was clicked.
    Closed(ToastId),
    /// The timer ran out.
    Expired(ToastId),
}

/// One notification. Build it, then pass it to [`Toasts::add`].
#[derive(Clone, Debug)]
pub struct Toast {
    pub(super) kind: ToastKind,
    pub(super) title: String,
    pub(super) description: Option<String>,
    pub(super) action: Option<String>,
    pub(super) duration: Option<f32>,
    pub(super) closable: bool,
}

impl Toast {
    /// A toast with a title and the default 4 s timeout (none for `Loading`).
    pub fn new(kind: ToastKind, title: impl Into<String>) -> Self {
        Self {
            kind,
            title: title.into(),
            description: None,
            action: None,
            duration: (kind != ToastKind::Loading).then_some(DEFAULT_DURATION),
            closable: true,
        }
    }

    /// A second, muted line under the title. It wraps.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// A small button on the right that reports [`ToastEvent::Action`].
    pub fn action(mut self, label: impl Into<String>) -> Self {
        self.action = Some(label.into());
        self
    }

    /// Seconds before the toast closes on its own. Hovering any toast pauses the timer.
    pub fn duration(mut self, seconds: f32) -> Self {
        self.duration = Some(seconds.max(0.5));
        self
    }

    /// Keep the toast until it is closed or updated.
    pub fn persistent(mut self) -> Self {
        self.duration = None;
        self
    }

    /// Show the × button (on by default).
    pub fn closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }
}

const DEFAULT_DURATION: f32 = 4.0;
const ENTER: f32 = 0.18;
const EXIT: f32 = 0.14;
const GAP: f32 = 8.0;
const MARGIN: f32 = 16.0;

struct Entry {
    id: ToastId,
    toast: Toast,
    remaining: Option<f32>,
    shown_at: Option<f64>,
    closing_at: Option<f64>,
    dismissed: bool,
    height: f32,
}

/// The toast queue and its renderer.
///
/// Keep one `Toasts` in the application state, push toasts from anywhere, and call
/// [`Toasts::show`] once per frame after the panels. Nothing repaints while the stack is
/// empty; while toasts are visible, the frame only wakes up for the next timeout or
/// animation step.
///
/// ```
/// # use egui_workbench::components::{Toast, ToastKind, Toasts};
/// # let ctx = egui::Context::default();
/// # let mut icons = egui_workbench::Icons::new();
/// let mut toasts = Toasts::new();
/// toasts.success("Saved");
/// let id = toasts.loading("Exporting…");
/// toasts.update(id, Toast::new(ToastKind::Success, "Exported").description("report.pdf"));
/// # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
/// for event in toasts.show(ui.ctx(), &mut icons) {
///     let _ = event;
/// }
/// # });
/// # output.textures_delta.clear();
/// ```
pub struct Toasts {
    entries: Vec<Entry>,
    next: u64,
    position: ToastPosition,
    width: f32,
    max_visible: usize,
    offset: Pos2,
    last_tick: Option<f64>,
}

impl Default for Toasts {
    fn default() -> Self {
        Self::new()
    }
}

impl Toasts {
    /// An empty stack in the bottom-right corner, 356 pt wide, at most 4 visible.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            next: 0,
            position: ToastPosition::default(),
            width: 356.0,
            max_visible: 4,
            offset: Pos2::ZERO,
            last_tick: None,
        }
    }

    /// Where the stack grows from.
    pub fn position(mut self, position: ToastPosition) -> Self {
        self.position = position;
        self
    }

    /// Change the corner at run time.
    pub fn set_position(&mut self, position: ToastPosition) {
        self.position = position;
    }

    /// Toast width in points; it shrinks on narrow windows.
    pub fn width(mut self, width: f32) -> Self {
        self.width = width.max(200.0);
        self
    }

    /// How many toasts are visible at once; older ones wait their turn.
    pub fn max_visible(mut self, count: usize) -> Self {
        self.max_visible = count.max(1);
        self
    }

    /// Extra inset from the anchor edges, such as the title or status bar height.
    pub fn offset(mut self, x: f32, y: f32) -> Self {
        self.offset = pos2(x, y);
        self
    }

    /// Queue a toast and return its id.
    pub fn add(&mut self, toast: Toast) -> ToastId {
        self.next += 1;
        let id = ToastId(self.next);
        self.entries.push(Entry {
            id,
            remaining: toast.duration,
            toast,
            shown_at: None,
            closing_at: None,
            dismissed: false,
            height: 64.0,
        });
        id
    }

    /// Queue an informational toast.
    pub fn info(&mut self, title: impl Into<String>) -> ToastId {
        self.add(Toast::new(ToastKind::Info, title))
    }

    /// Queue a success toast.
    pub fn success(&mut self, title: impl Into<String>) -> ToastId {
        self.add(Toast::new(ToastKind::Success, title))
    }

    /// Queue a warning toast.
    pub fn warning(&mut self, title: impl Into<String>) -> ToastId {
        self.add(Toast::new(ToastKind::Warning, title))
    }

    /// Queue an error toast.
    pub fn error(&mut self, title: impl Into<String>) -> ToastId {
        self.add(Toast::new(ToastKind::Error, title))
    }

    /// Queue a persistent loading toast; replace it later with [`Self::update`].
    pub fn loading(&mut self, title: impl Into<String>) -> ToastId {
        self.add(Toast::new(ToastKind::Loading, title))
    }

    /// Replace a toast in place (for example, loading → success) and restart its timer.
    /// Returns `false` when the toast is already gone.
    pub fn update(&mut self, id: ToastId, toast: Toast) -> bool {
        let Some(entry) = self.entries.iter_mut().find(|entry| entry.id == id) else {
            return false;
        };
        entry.remaining = toast.duration;
        entry.toast = toast;
        entry.closing_at = None;
        entry.dismissed = false;
        true
    }

    /// Close a toast with its exit animation.
    pub fn dismiss(&mut self, id: ToastId) {
        if let Some(entry) = self.entries.iter_mut().find(|entry| entry.id == id) {
            entry.dismissed = true;
        }
    }

    /// Close every toast.
    pub fn dismiss_all(&mut self) {
        for entry in &mut self.entries {
            entry.dismissed = true;
        }
    }

    /// Whether no toast is queued or visible.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Queued and visible toasts.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Paint the stack above every panel and return what happened this frame.
    pub fn show(&mut self, ctx: &Context, icons: &mut Icons) -> Vec<ToastEvent> {
        let mut events = Vec::new();
        if self.entries.is_empty() {
            self.last_tick = None;
            return events;
        }
        let tokens = theme::current(ctx);
        let motion = theme::motion_enabled(ctx);
        let now = ctx.input(|input| input.time);
        let dt = self.last_tick.map_or(0.0, |last| (now - last).max(0.0) as f32);
        self.last_tick = Some(now);
        let screen = ctx.content_rect();
        let width = self.width.min(screen.width() - 2.0 * MARGIN).max(160.0);
        let paused = self.hovered(ctx);

        let visible = self.entries.len().min(self.max_visible);
        let mut cursor = 0.0;
        let mut wake: Option<f32> = None;
        // Newest closest to the anchor edge.
        for slot in 0..visible {
            let index = self.entries.len() - visible + (visible - 1 - slot);
            let entry = &mut self.entries[index];
            let shown_at = *entry.shown_at.get_or_insert(now);

            if entry.closing_at.is_none() {
                if entry.dismissed {
                    entry.closing_at = Some(now);
                } else if let Some(remaining) = entry.remaining.as_mut() {
                    // A toast that just appeared starts its timer this frame.
                    if !paused && shown_at < now {
                        *remaining -= dt;
                    }
                    if *remaining <= 0.0 {
                        entry.closing_at = Some(now);
                        events.push(ToastEvent::Expired(entry.id));
                    } else if !paused {
                        wake = Some(wake.map_or(*remaining, |wake| wake.min(*remaining)));
                    }
                }
            }

            let enter = if motion {
                ((now - shown_at) as f32 / ENTER).clamp(0.0, 1.0)
            } else {
                1.0
            };
            let exit = match entry.closing_at {
                Some(at) if motion => 1.0 - ((now - at) as f32 / EXIT).clamp(0.0, 1.0),
                Some(_) => 0.0,
                None => 1.0,
            };
            if enter < 1.0 || entry.closing_at.is_some() {
                ctx.request_repaint();
            }
            let visibility = view::ease_out(enter) * exit;
            let slide = (1.0 - view::ease_out(enter)) * 16.0;

            let x = match self.position {
                ToastPosition::TopLeft | ToastPosition::BottomLeft => screen.left() + MARGIN + self.offset.x,
                ToastPosition::TopCenter | ToastPosition::BottomCenter => screen.center().x - width * 0.5,
                ToastPosition::TopRight | ToastPosition::BottomRight => screen.right() - MARGIN - width - self.offset.x,
            };
            let target_y = if self.position.is_top() {
                screen.top() + MARGIN + self.offset.y + cursor - slide
            } else {
                screen.bottom() - MARGIN - self.offset.y - cursor - entry.height + slide
            };
            let area_id = Id::unique(("workbench_toast", entry.id.0));
            let y = if motion {
                ctx.animate_value_with_time(area_id.with("y"), target_y, 0.2)
            } else {
                target_y
            };
            cursor += (entry.height + GAP) * exit;

            let id = entry.id;
            let toast = &entry.toast;
            let area = Area::new(area_id)
                .order(Order::Foreground)
                .fixed_pos(pos2(x, y))
                .interactable(true)
                .show(ctx, |ui| {
                    ui.multiply_opacity(visibility);
                    view::paint(ui, icons, toast, &tokens, width)
                });
            entry.height = area.response.rect.height();
            match area.inner {
                Some(ToastEvent::Action(_)) => {
                    events.push(ToastEvent::Action(id));
                    entry.closing_at.get_or_insert(now);
                }
                Some(ToastEvent::Closed(_)) => {
                    events.push(ToastEvent::Closed(id));
                    entry.closing_at.get_or_insert(now);
                }
                _ => {}
            }
        }

        let before = self.entries.len();
        self.entries.retain(|entry| match entry.closing_at {
            Some(at) => motion && ((now - at) as f32) < EXIT,
            None => true,
        });
        let removed = before > self.entries.len();
        if let Some(wake) = wake {
            ctx.request_repaint_after(std::time::Duration::from_secs_f32(wake.max(0.016)));
        }
        if removed && !self.entries.is_empty() {
            // A queued toast takes the freed slot, and the stack closes the gap.
            ctx.request_repaint();
        }
        if self.entries.iter().any(|entry| entry.toast.kind == ToastKind::Loading) {
            ctx.request_repaint_after(std::time::Duration::from_millis(33));
        }
        events
    }

    fn hovered(&self, ctx: &Context) -> bool {
        let Some(pointer) = ctx.pointer_hover_pos() else {
            return false;
        };
        self.entries.iter().any(|entry| {
            ctx.memory(|memory| {
                memory
                    .area_rect(Id::unique(("workbench_toast", entry.id.0)))
                    .is_some_and(|rect| rect.contains(pointer))
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loading_toasts_do_not_expire_and_update_in_place() {
        let mut toasts = Toasts::new();
        let id = toasts.loading("Working");
        assert_eq!(toasts.entries[0].remaining, None);
        assert!(toasts.update(id, Toast::new(ToastKind::Success, "Done")));
        assert_eq!(toasts.entries[0].remaining, Some(DEFAULT_DURATION));
        assert_eq!(toasts.len(), 1);
    }

    #[test]
    fn dismiss_starts_the_exit() {
        let mut toasts = Toasts::new();
        let id = toasts.info("Hello");
        toasts.dismiss(id);
        assert!(toasts.entries[0].dismissed);
        assert!(!toasts.update(ToastId(99), Toast::new(ToastKind::Info, "x")));
    }
}
