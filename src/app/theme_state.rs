//! Theme state shared by applications.
//!
//! [`ThemeState`] keeps the requested theme, the follow-system preference, the reduced
//! motion preference, and the effective theme that is currently installed in egui. It
//! applies a theme only when something actually changes, so an application can call
//! [`ThemeState::sync_system`] every frame without rebuilding all egui styles.

use egui::{Color32, Context};

use crate::foundation::theme::{self, Theme, ThemeId, ThemeMode};

/// Requested and effective theme, follow-system, and reduced-motion preferences.
///
/// ```
/// # use egui_workbench::{ThemeId, app::ThemeState};
/// # use egui_workbench::egui;
/// let ctx = egui::Context::default();
/// let mut theme = ThemeState::new(&ctx, ThemeId::GithubDark, false);
/// assert_eq!(theme.effective(), ThemeId::GithubDark);
/// assert!(theme.is_dark());
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThemeState {
    requested: ThemeId,
    follow_system: bool,
    effective: ThemeId,
    reduced_motion: bool,
    accent: Option<Color32>,
    custom: Option<Theme>,
}

impl Default for ThemeState {
    fn default() -> Self {
        Self {
            requested: ThemeId::GithubLight,
            follow_system: false,
            effective: ThemeId::GithubLight,
            reduced_motion: false,
            accent: None,
            custom: None,
        }
    }
}

impl ThemeState {
    /// Apply `requested` (following the desktop preference when asked) and keep the state.
    pub fn new(ctx: &Context, requested: ThemeId, follow_system: bool) -> Self {
        Self::with_reduced_motion(ctx, requested, follow_system, false)
    }

    /// Like [`Self::new`], with the reduced-motion preference applied too.
    pub fn with_reduced_motion(ctx: &Context, requested: ThemeId, follow_system: bool, reduced_motion: bool) -> Self {
        let effective = theme::apply_theme(ctx, requested, follow_system);
        theme::set_reduced_motion(ctx, reduced_motion);
        Self {
            requested,
            follow_system,
            effective,
            reduced_motion,
            accent: None,
            custom: None,
        }
    }

    /// Restore a theme from stored settings. Unknown values fall back to GitHub Light.
    pub fn from_settings(ctx: &Context, requested: &str, follow_system: bool, reduced_motion: bool) -> Self {
        Self::with_reduced_motion(ctx, ThemeId::from_setting(requested), follow_system, reduced_motion)
    }

    /// The stored theme preference.
    pub fn requested(&self) -> ThemeId {
        self.requested
    }

    /// The theme currently installed in egui.
    pub fn effective(&self) -> ThemeId {
        self.effective
    }

    /// Whether the light/dark base follows the desktop preference.
    pub fn follow_system(&self) -> bool {
        self.follow_system
    }

    /// Whether animations are disabled.
    pub fn reduced_motion(&self) -> bool {
        self.reduced_motion
    }

    /// The tokens currently installed: the custom theme when one is set, otherwise the
    /// effective built-in theme with the brand accent (if any).
    pub fn tokens(&self) -> Theme {
        self.custom.unwrap_or_else(|| {
            let base = Theme::for_id(self.effective);
            self.accent.map_or(base, |accent| base.with_accent(accent))
        })
    }

    /// The brand accent applied on top of the built-in themes.
    pub fn accent(&self) -> Option<Color32> {
        self.accent
    }

    /// Replace the accent, link, focus, and selection of every built-in theme with a
    /// brand color (see [`Theme::with_accent`]; contrast stays AA), or remove it with
    /// `None`. The accent follows theme changes and the desktop light/dark preference.
    ///
    /// ```
    /// # use egui_workbench::{ThemeId, Theme, app::ThemeState};
    /// # use egui_workbench::egui::{self, Color32};
    /// let ctx = egui::Context::default();
    /// let mut theme = ThemeState::new(&ctx, ThemeId::GithubDark, false);
    /// theme.set_accent(&ctx, Some(Color32::from_rgb(0xe0, 0x6c, 0x2b)));
    /// assert_ne!(theme.tokens().accent, Theme::dark().accent);
    /// theme.set_requested(&ctx, ThemeId::GithubLight);
    /// assert!(theme.is_light(), "the accent is re-derived on the new base");
    /// ```
    pub fn set_accent(&mut self, ctx: &Context, accent: Option<Color32>) {
        if accent != self.accent {
            self.accent = accent;
            self.apply(ctx);
        }
    }

    /// The custom theme, if one replaces the built-in themes.
    pub fn custom(&self) -> Option<Theme> {
        self.custom
    }

    /// Install your own tokens (for example `Theme::dark().with_accent(brand)`), or go
    /// back to the requested built-in theme with `None`. While a custom theme is set,
    /// following the system preference has no effect.
    pub fn set_custom(&mut self, ctx: &Context, custom: Option<Theme>) {
        if custom != self.custom {
            self.custom = custom;
            self.apply(ctx);
        }
    }

    /// Whether the effective theme has a light base.
    pub fn is_light(&self) -> bool {
        match self.custom {
            Some(custom) => !custom.is_dark(),
            None => self.effective.mode() == ThemeMode::Light,
        }
    }

    /// Whether the effective theme has a dark base.
    pub fn is_dark(&self) -> bool {
        !self.is_light()
    }

    /// The window clear color for `eframe::App::clear_color`.
    pub fn clear_color(&self) -> [f32; 4] {
        self.tokens().background.to_normalized_gamma_f32()
    }

    /// Re-resolve and apply the requested theme. Returns the effective theme.
    pub fn apply(&mut self, ctx: &Context) -> ThemeId {
        if self.custom.is_none() {
            self.effective = theme::effective_id(self.requested, self.follow_system, theme::system_is_dark(ctx));
        }
        theme::apply_tokens(ctx, &self.tokens());
        theme::set_reduced_motion(ctx, self.reduced_motion);
        self.effective
    }

    /// Change the requested theme and apply it.
    pub fn set_requested(&mut self, ctx: &Context, requested: ThemeId) {
        if requested != self.requested {
            self.requested = requested;
            self.apply(ctx);
        }
    }

    /// Turn follow-system on or off and re-apply the theme.
    pub fn set_follow_system(&mut self, ctx: &Context, follow_system: bool) {
        if follow_system != self.follow_system {
            self.follow_system = follow_system;
            self.apply(ctx);
        }
    }

    /// Turn reduced motion on or off. Re-applies egui's animation time only on change.
    pub fn set_reduced_motion(&mut self, ctx: &Context, reduced_motion: bool) {
        if reduced_motion != self.reduced_motion {
            self.reduced_motion = reduced_motion;
            theme::set_reduced_motion(ctx, reduced_motion);
        }
    }

    /// Re-apply the theme when the desktop light/dark preference changed.
    ///
    /// Call this from `eframe::App::logic`. Returns `true` when a new effective theme
    /// was installed.
    pub fn sync_system(&mut self, ctx: &Context) -> bool {
        if !self.follow_system || self.custom.is_some() {
            return false;
        }
        let effective = theme::effective_id(self.requested, true, theme::system_is_dark(ctx));
        if effective == self.effective {
            return false;
        }
        self.effective = effective;
        theme::apply_tokens(ctx, &self.tokens());
        theme::set_reduced_motion(ctx, self.reduced_motion);
        true
    }

    /// The stored theme identifier.
    pub fn theme_setting(&self) -> &'static str {
        self.requested.as_setting()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_round_trip() {
        let ctx = Context::default();
        let state = ThemeState::from_settings(&ctx, "dracula-soft", false, true);
        assert_eq!(state.requested(), ThemeId::DraculaSoft);
        assert_eq!(state.theme_setting(), "dracula-soft");
        assert!(!state.follow_system());
        assert!(state.reduced_motion());
        assert!(state.is_dark());
        assert_eq!(state.clear_color().len(), 4);
    }

    #[test]
    fn changing_the_theme_updates_the_effective_tokens() {
        let ctx = Context::default();
        let mut state = ThemeState::new(&ctx, ThemeId::GithubLight, false);
        state.set_requested(&ctx, ThemeId::OneDarkPro);
        assert_eq!(state.effective(), ThemeId::OneDarkPro);
        assert!(state.is_dark());
        assert_eq!(
            ctx.style_of(egui::Theme::Dark).visuals.panel_fill,
            state.tokens().background
        );
    }

    #[test]
    fn a_custom_theme_replaces_and_restores_the_built_in_one() {
        let ctx = Context::default();
        let mut state = ThemeState::new(&ctx, ThemeId::GithubLight, false);
        let brand = Theme::dark().with_accent(egui::Color32::from_rgb(0xe0, 0x6c, 0x2b));
        state.set_custom(&ctx, Some(brand));
        assert!(state.is_dark());
        assert_eq!(state.tokens(), brand);
        assert_eq!(theme::current(&ctx), brand);
        assert_eq!(ctx.style_of(egui::Theme::Dark).visuals.panel_fill, brand.background);
        state.set_custom(&ctx, None);
        assert!(state.is_light());
        assert_eq!(theme::current(&ctx), Theme::light());
    }

    #[test]
    fn the_accent_follows_theme_changes_and_the_desktop() {
        let ctx = Context::default();
        let brand = egui::Color32::from_rgb(0xe0, 0x6c, 0x2b);
        let mut state = ThemeState::new(&ctx, ThemeId::Alucard, true);
        state.set_accent(&ctx, Some(brand));
        assert_eq!(theme::current(&ctx), Theme::for_id(ThemeId::Alucard).with_accent(brand));
        let input = egui::RawInput {
            system_theme: Some(egui::Theme::Dark),
            ..egui::RawInput::default()
        };
        let mut output = ctx.run_ui(input, |ui| {
            assert!(state.sync_system(ui.ctx()));
        });
        output.textures_delta.clear();
        assert_eq!(state.effective(), ThemeId::Dracula);
        assert_eq!(theme::current(&ctx), Theme::for_id(ThemeId::Dracula).with_accent(brand));
        state.set_accent(&ctx, None);
        assert_eq!(theme::current(&ctx), Theme::for_id(ThemeId::Dracula));
    }

    #[test]
    fn following_the_system_keeps_the_family() {
        let ctx = Context::default();
        let state = ThemeState::with_reduced_motion(&ctx, ThemeId::Alucard, true, false);
        assert_eq!(state.effective(), ThemeId::Alucard);
        assert!(!state.is_dark());
    }

    #[test]
    fn reduced_motion_is_applied_once_and_survives_theme_changes() {
        let ctx = Context::default();
        let mut state = ThemeState::new(&ctx, ThemeId::GithubLight, false);
        state.set_reduced_motion(&ctx, true);
        assert_eq!(ctx.style_of(egui::Theme::Light).animation_time, 0.0);
        state.set_requested(&ctx, ThemeId::GithubDark);
        assert_eq!(ctx.style_of(egui::Theme::Dark).animation_time, 0.0);
    }

    #[test]
    fn syncs_the_desktop_preference_only_when_it_changes() {
        let ctx = Context::default();
        let mut state = ThemeState::with_reduced_motion(&ctx, ThemeId::Alucard, true, true);
        let mut changed = None;
        let mut input = egui::RawInput {
            system_theme: Some(egui::Theme::Dark),
            ..egui::RawInput::default()
        };
        let mut output = ctx.run_ui(input, |ui| {
            changed = Some(state.sync_system(ui.ctx()));
        });
        output.textures_delta.clear();
        assert_eq!(changed, Some(true));
        assert_eq!(state.effective(), ThemeId::Dracula);
        assert_eq!(ctx.style_of(egui::Theme::Dark).animation_time, 0.0);

        input = egui::RawInput {
            system_theme: Some(egui::Theme::Dark),
            ..egui::RawInput::default()
        };
        let mut output = ctx.run_ui(input, |ui| {
            changed = Some(state.sync_system(ui.ctx()));
        });
        output.textures_delta.clear();
        assert_eq!(changed, Some(false));
    }
}
