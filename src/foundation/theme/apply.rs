//! Applying theme tokens to an egui context.

use egui::style::{WidgetVisuals, Widgets};
use egui::{Color32, Stroke};

use super::{Theme, ThemeId, ThemeMode};

/// Duration of every hover, tab, and switch animation, in seconds.
pub const ANIMATION_TIME: f32 = 0.10;

/// Resolve a theme against the system preference, keeping its family.
pub fn effective_id(id: ThemeId, follow_system: bool, system_dark: bool) -> ThemeId {
    if follow_system {
        id.follow_system(system_dark)
    } else {
        id
    }
}

/// Whether the desktop currently prefers a dark theme.
pub fn system_is_dark(ctx: &egui::Context) -> bool {
    ctx.input(|input| input.raw.system_theme)
        .is_some_and(|theme| theme == egui::Theme::Dark)
}

/// Apply the GitHub light or dark theme.
pub fn apply(ctx: &egui::Context, mode: ThemeMode) {
    apply_id(
        ctx,
        match mode {
            ThemeMode::Light => ThemeId::GithubLight,
            ThemeMode::Dark => ThemeId::GithubDark,
        },
    );
}

/// Apply a theme, following the system light/dark preference when requested.
pub fn apply_theme(ctx: &egui::Context, id: ThemeId, follow_system: bool) -> ThemeId {
    let effective = effective_id(id, follow_system, system_is_dark(ctx));
    apply_id(ctx, effective);
    effective
}

/// Apply a theme identifier: visuals, widget states, spacing, scrollbars, and type scale.
pub fn apply_id(ctx: &egui::Context, id: ThemeId) {
    apply_tokens(ctx, &Theme::for_id(id));
}

/// Apply any set of tokens, built-in or your own (see [`Theme::with_accent`] and
/// [`Theme::accessibility_issues`]). The light or dark egui base follows
/// [`Theme::is_dark`]; the installed [`crate::style::WorkbenchStyle`] is kept.
pub fn apply_tokens(ctx: &egui::Context, theme: &Theme) {
    let theme = *theme;
    ctx.data_mut(|data| data.insert_temp(current_key(), theme));
    let mut visuals = if theme.is_dark() {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    visuals.panel_fill = theme.background;
    visuals.window_fill = theme.surface;
    visuals.faint_bg_color = theme.hover;
    visuals.extreme_bg_color = theme.sunken;
    visuals.hyperlink_color = theme.link;
    visuals.selection.bg_fill = theme.selection;
    visuals.selection.stroke = Stroke::new(1.0, theme.text);
    visuals.window_stroke = Stroke::new(1.0, theme.border);
    // Semantic colors come from the theme, not egui's saturated #FF0000 / #FF6400, so
    // danger and warning text meets AA on every surface.
    visuals.error_fg_color = theme.danger;
    visuals.warn_fg_color = theme.warning;

    paint_widgets(&mut visuals.widgets, &theme);
    visuals.override_text_color = Some(theme.text);

    ctx.all_styles_mut(|style| {
        style.visuals = visuals.clone();
        style.spacing.item_spacing = egui::vec2(8.0, 4.0);
        style.spacing.button_padding = egui::vec2(8.0, 3.0);
        style.spacing.interact_size = egui::vec2(26.0, 24.0);
        style.spacing.menu_margin = egui::Margin::same(4);
        style.spacing.scroll.bar_width = 6.0;
        style.spacing.scroll.floating_width = 3.0;
        style.spacing.scroll.floating_allocated_width = 4.0;
        style.spacing.scroll.bar_inner_margin = 2.0;
        style.animation_time = ANIMATION_TIME;
    });
    // Radii, border widths, and the type scale come from the installed style.
    crate::foundation::style::apply_to_egui(ctx, &crate::foundation::style::get(ctx));
}

/// The five widget states. Fills come from the tokens; every state shares a 1 px
/// focus-colored foreground stroke before the per-state overrides. Radii and border
/// widths are set afterwards from the [`crate::style::WorkbenchStyle`].
fn paint_widgets(widgets: &mut Widgets, theme: &Theme) {
    let foreground = Stroke::new(1.0, theme.focus);
    let states: [(&mut WidgetVisuals, Color32); 5] = [
        (&mut widgets.noninteractive, theme.background),
        (&mut widgets.inactive, theme.surface),
        (&mut widgets.hovered, theme.hover),
        (&mut widgets.active, theme.accent),
        (&mut widgets.open, theme.surface),
    ];
    for (state, fill) in states {
        state.bg_fill = fill;
        state.weak_bg_fill = fill;
        state.fg_stroke = foreground;
    }
    widgets.inactive.weak_bg_fill = theme.surface;
    widgets.inactive.bg_stroke = theme.border_stroke();
    widgets.hovered.bg_stroke = Stroke::new(1.0, theme.focus);
    widgets.active.fg_stroke = Stroke::new(1.0, theme.text);
    widgets.active.bg_stroke = Stroke::new(2.0, theme.focus);
    widgets.noninteractive.bg_stroke = Stroke::new(1.0, theme.border);
}

fn current_key() -> egui::Id {
    egui::Id::unique("egui_workbench::theme::current")
}

/// The tokens of the theme last applied with [`apply_id`] (or [`apply_theme`]).
///
/// Components use it for the roles egui's visuals do not carry, such as
/// `success`. Falls back to GitHub Light or Dark, following the current visuals,
/// when no theme was applied.
pub fn current(ctx: &egui::Context) -> Theme {
    ctx.data(|data| data.get_temp::<Theme>(current_key()))
        .unwrap_or_else(|| {
            if ctx.style_of(ctx.theme()).visuals.dark_mode {
                Theme::dark()
            } else {
                Theme::light()
            }
        })
}

/// Whether animations are on (reduced motion sets the animation time to zero).
pub fn motion_enabled(ctx: &egui::Context) -> bool {
    ctx.style_of(ctx.theme()).animation_time > 0.0
}

/// Turn animations off for reduced-motion preferences, or back on.
pub fn set_reduced_motion(ctx: &egui::Context, reduced: bool) {
    ctx.all_styles_mut(|style| {
        style.animation_time = if reduced { 0.0 } else { ANIMATION_TIME };
    });
}
