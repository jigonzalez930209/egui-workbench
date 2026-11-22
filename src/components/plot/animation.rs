//! Hover animations for legend rows and the direction arrow of streaming series.
//!
//! Every effect is a pure function of the time since the pointer entered a legend row, so
//! the screen and an export (which never animates) share the same geometry. Reduced
//! motion turns every effect off. Effects are selected per plot in [`PlotConfig`], and
//! each chart family interprets them where it makes sense (line motion, dash motion, pie
//! slices, radar vertices, gauge needles).
//!
//! [`PlotConfig`]: super::PlotConfig

use egui::Color32;

use super::PlotConfig;
use super::{PlotState, scene::Dash};

/// The animation played while the pointer rests on a legend row.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LegendAnimation {
    /// Smoothly blends the series color into the highlight color and back.
    #[default]
    Blink,
    /// Pulses the series opacity.
    Fade,
    /// Pulses the stroke width.
    Throb,
    /// Scrolls the dashes along the line (marching ants).
    DashMarch,
    /// Grows the dashes in and out of a solid line.
    DashToggle,
    /// Draws a soft halo around the line.
    Glow,
    /// A bright segment travels along the line.
    Sweep,
    /// The line wiggles perpendicular to its direction.
    Wiggle,
    /// The line draws itself, repeating.
    Reveal,
    /// Markers bounce up and down.
    MarkerBounce,
    /// Markers grow and shrink.
    MarkerPulse,
    /// Small sparkles appear along the line.
    Twinkle,
    /// The line's color washes from left to right.
    Wave,
}

impl LegendAnimation {
    /// Every animation, in menu order.
    pub const ALL: [Self; 13] = [
        Self::Blink,
        Self::Fade,
        Self::Throb,
        Self::DashMarch,
        Self::DashToggle,
        Self::Glow,
        Self::Sweep,
        Self::Wiggle,
        Self::Reveal,
        Self::MarkerBounce,
        Self::MarkerPulse,
        Self::Twinkle,
        Self::Wave,
    ];

    /// A short name for settings menus.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Blink => "Blink",
            Self::Fade => "Fade",
            Self::Throb => "Throb",
            Self::DashMarch => "Marching dashes",
            Self::DashToggle => "Dash toggle",
            Self::Glow => "Glow",
            Self::Sweep => "Sweep",
            Self::Wiggle => "Wiggle",
            Self::Reveal => "Reveal",
            Self::MarkerBounce => "Marker bounce",
            Self::MarkerPulse => "Marker pulse",
            Self::Twinkle => "Twinkle",
            Self::Wave => "Color wave",
        }
    }
}

/// How the animation's clock is shaped within one cycle.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AnimationEasing {
    /// Constant speed.
    #[default]
    Linear,
    /// Starts slow.
    EaseIn,
    /// Ends slow.
    EaseOut,
    /// Slow at both ends.
    EaseInOut,
    /// Holds, then jumps at the midpoint.
    Step,
    /// A smooth sine oscillator: back and forth with no abrupt turn.
    Sine,
}

impl AnimationEasing {
    /// Every easing, in menu order.
    pub const ALL: [Self; 6] = [
        Self::Linear,
        Self::EaseIn,
        Self::EaseOut,
        Self::EaseInOut,
        Self::Step,
        Self::Sine,
    ];

    /// A short name for settings menus.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Linear => "Linear",
            Self::EaseIn => "Ease in",
            Self::EaseOut => "Ease out",
            Self::EaseInOut => "Ease in-out",
            Self::Step => "Step",
            Self::Sine => "Sine",
        }
    }

    /// Shape a cycle position in `0..=1`.
    pub fn apply(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Self::Linear => t,
            Self::EaseIn => t * t,
            Self::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            Self::EaseInOut => t * t * (3.0 - 2.0 * t),
            Self::Step => {
                if t < 0.5 {
                    0.0
                } else {
                    1.0
                }
            }
            Self::Sine => 0.5 - 0.5 * (std::f32::consts::TAU * t).cos(),
        }
    }
}

/// Settings for the legend hover animation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegendHoverConfig {
    /// Play an effect while the pointer rests on a legend row.
    pub enabled: bool,
    /// Which effect.
    pub animation: LegendAnimation,
    /// How the cycle is shaped.
    pub easing: AnimationEasing,
    /// Seconds for one full cycle.
    pub period: f32,
    /// The second color; [`crate::Theme::warning`](crate::theme::Theme) when `None`.
    pub highlight: Option<Color32>,
}

impl Default for LegendHoverConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            animation: LegendAnimation::Blink,
            easing: AnimationEasing::Linear,
            period: 1.2,
            highlight: None,
        }
    }
}

/// An effect running on one series this frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Effect {
    pub kind: LegendAnimation,
    /// Eased cycle position in `0..1`.
    pub phase: f32,
    pub highlight: Color32,
    /// The extreme `Blink` color: white on dark themes, black on light themes (or the
    /// user's custom highlight).
    pub blink: Color32,
}

impl Effect {
    /// The effect of the hovered legend row at `now`, if motion is on.
    pub(super) fn for_hover(
        config: &PlotConfig,
        state: &PlotState,
        series: usize,
        now: f64,
        theme: &crate::theme::Theme,
        motion: bool,
    ) -> Option<Self> {
        let hover = config.legend_hover;
        if !hover.enabled || !motion || state.legend_hover() != Some(series) {
            return None;
        }
        let period = f64::from(hover.period.max(0.2));
        let elapsed = (now - state.legend_hover_start()).max(0.0);
        let t = (elapsed / period).rem_euclid(1.0) as f32;
        Some(Self {
            kind: hover.animation,
            phase: hover.easing.apply(t),
            highlight: hover.highlight.unwrap_or(theme.warning),
            blink: hover.highlight.unwrap_or(if theme.is_dark() {
                Color32::WHITE
            } else {
                Color32::BLACK
            }),
        })
    }

    /// The smooth 0 → 1 → 0 oscillator of one cycle.
    pub(super) fn wave(self) -> f32 {
        0.5 - 0.5 * (std::f32::consts::TAU * self.phase).cos()
    }

    /// The series color blended toward the highlight.
    pub(super) fn color(self, base: Color32) -> Color32 {
        match self.kind {
            // A strong two-color blink: the series goes from white (on dark themes) or
            // black (on light themes) to its own color and back.
            LegendAnimation::Blink => mix(self.blink, base, self.wave()),
            LegendAnimation::Fade => base.gamma_multiply(1.0 - 0.65 * self.wave()),
            LegendAnimation::Throb => mix(base, self.highlight, 0.3 * self.wave()),
            LegendAnimation::Wave => mix(base, self.highlight, 0.55 * self.phase),
            LegendAnimation::Glow | LegendAnimation::Sweep | LegendAnimation::Reveal => {
                mix(base, self.highlight, 0.35 * self.wave())
            }
            _ => mix(base, self.highlight, 0.18 * self.wave()),
        }
    }

    /// A halo color for [`LegendAnimation::Glow`], or `None`.
    pub(super) fn halo(self) -> Option<Color32> {
        (self.kind == LegendAnimation::Glow).then(|| self.highlight.gamma_multiply(0.18 + 0.25 * self.wave()))
    }

    /// The stroke width, after width effects.
    pub(super) fn width(self, base: f32) -> f32 {
        match self.kind {
            LegendAnimation::Throb => base * (1.0 + 0.9 * self.wave()),
            LegendAnimation::Glow => base * 1.15,
            _ => base,
        }
    }

    /// The dash pattern, after motion effects.
    pub(super) fn dash(self, base: Dash) -> Dash {
        match self.kind {
            LegendAnimation::DashMarch => {
                let (dash, gap) = dash_gap(base);
                Dash::Marching {
                    dash,
                    gap,
                    offset: self.phase * (dash + gap) * 2.0,
                }
            }
            LegendAnimation::DashToggle => {
                let (dash, gap) = dash_gap(base);
                let grow = self.wave();
                if grow < 0.08 {
                    Dash::Solid
                } else {
                    Dash::Dashed(dash * grow, gap * grow)
                }
            }
            _ => base,
        }
    }

    /// A screen-space offset for one point of a line or polygon.
    pub(super) fn point_offset(self, index: usize, tangent: egui::Vec2) -> egui::Vec2 {
        if self.kind != LegendAnimation::Wiggle {
            return egui::Vec2::ZERO;
        }
        let length = tangent.length();
        if length < 1e-3 {
            return egui::Vec2::ZERO;
        }
        let normal = egui::vec2(-tangent.y, tangent.x) / length;
        normal * ((index as f32 * 0.55 + self.phase * std::f32::consts::TAU).sin() * 3.0)
    }

    /// A screen-space offset for one marker.
    pub(super) fn marker_offset(self, index: usize) -> egui::Vec2 {
        match self.kind {
            LegendAnimation::MarkerBounce => {
                let t = index as f32 * 0.8 + self.phase * std::f32::consts::TAU;
                egui::vec2(0.0, -((t.sin() + 1.0) * 2.5))
            }
            _ => egui::Vec2::ZERO,
        }
    }

    /// A marker radius, after pulse effects.
    pub(super) fn marker_radius(self, base: f32, index: usize) -> f32 {
        match self.kind {
            LegendAnimation::MarkerPulse => base * (1.0 + 0.7 * self.wave()),
            LegendAnimation::Twinkle => base * (0.7 + 0.8 * (index as f32 * 0.9 + self.phase * 6.0).sin().abs()),
            _ => base,
        }
    }

    /// A portion of `points` to draw in the highlight color, for sweep and reveal.
    pub(super) fn highlighted(self, points: &[egui::Pos2]) -> Option<Vec<egui::Pos2>> {
        let count = points.len();
        if count < 2 {
            return None;
        }
        match self.kind {
            LegendAnimation::Sweep => {
                let span = (count / 4).max(2);
                let start = ((count - span) as f32 * self.phase) as usize;
                Some(points[start..(start + span).min(count)].to_vec())
            }
            LegendAnimation::Reveal => {
                let end = (count as f32 * self.phase).ceil() as usize;
                (end >= 2).then(|| points[..end.min(count)].to_vec())
            }
            _ => None,
        }
    }

    /// Apply a pie slice offset: the hovered slice lifts out of the ring.
    pub(super) fn explode(self) -> f32 {
        match self.kind {
            LegendAnimation::Wiggle | LegendAnimation::MarkerBounce => 5.0 + 2.0 * self.wave(),
            _ => 3.0 + 2.0 * self.wave(),
        }
    }

    /// Apply a radar value scale, for vertex motion.
    pub(super) fn radial_scale(self) -> f32 {
        match self.kind {
            LegendAnimation::Wiggle => 1.0 + 0.06 * self.wave(),
            LegendAnimation::Throb => 1.0 + 0.03 * self.wave(),
            _ => 1.0,
        }
    }
}

fn dash_gap(dash: Dash) -> (f32, f32) {
    match dash {
        Dash::Solid => (7.0, 4.0),
        Dash::Dashed(dash, gap) => (dash, gap),
        Dash::Dotted(spacing) => (spacing * 0.5, spacing),
        Dash::Marching { dash, gap, .. } => (dash, gap),
    }
}

/// Linear interpolation in sRGB with unmultiplied alpha.
pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let [ar, ag, ab, aa] = a.to_srgba_unmultiplied();
    let [br, bg, bb, ba] = b.to_srgba_unmultiplied();
    let channel = |a: u8, b: u8| (f32::from(a) + (f32::from(b) - f32::from(a)) * t).round() as u8;
    Color32::from_rgba_unmultiplied(channel(ar, br), channel(ag, bg), channel(ab, bb), channel(aa, ba))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn easing_stays_in_range_and_shapes_the_cycle() {
        for easing in AnimationEasing::ALL {
            for step in 0..=20 {
                let t = easing.apply(step as f32 / 20.0);
                assert!((0.0..=1.0).contains(&t), "{easing:?} at {step}: {t}");
            }
        }
        assert_eq!(AnimationEasing::Linear.apply(0.25), 0.25);
        assert!(AnimationEasing::EaseIn.apply(0.25) < 0.25);
        assert!(AnimationEasing::EaseOut.apply(0.25) > 0.25);
    }

    #[test]
    fn every_animation_has_a_label() {
        for animation in LegendAnimation::ALL {
            assert!(!animation.label().is_empty());
        }
    }

    #[test]
    fn mix_blends_both_ends() {
        let a = Color32::from_rgb(0, 0, 0);
        let b = Color32::from_rgb(200, 100, 50);
        assert_eq!(mix(a, b, 0.0), a);
        assert_eq!(mix(a, b, 1.0), b);
        assert_eq!(mix(a, b, 0.5), Color32::from_rgb(100, 50, 25));
    }

    #[test]
    fn the_hover_effect_follows_the_pointer_and_reduced_motion() {
        let config = PlotConfig::default();
        let mut state = PlotState::default();
        let theme = crate::theme::Theme::dark();
        assert!(Effect::for_hover(&config, &state, 0, 0.0, &theme, true).is_none());
        state.set_legend_hover(Some(0), 1.0);
        let effect = Effect::for_hover(&config, &state, 0, 1.6, &theme, true).expect("hovered");
        assert_eq!(effect.kind, LegendAnimation::Blink);
        assert!(effect.wave() > 0.0);
        assert!(Effect::for_hover(&config, &state, 1, 1.6, &theme, true).is_none());
        assert!(Effect::for_hover(&config, &state, 0, 1.6, &theme, false).is_none());
    }

    #[test]
    fn blink_goes_from_white_on_dark_and_black_on_light() {
        let base = Color32::from_rgb(30, 120, 220);
        let dark = Effect {
            kind: LegendAnimation::Blink,
            phase: 0.0,
            highlight: Color32::YELLOW,
            blink: Color32::WHITE,
        };
        let light = Effect {
            blink: Color32::BLACK,
            ..dark
        };
        assert_eq!(dark.color(base), Color32::WHITE);
        assert_eq!(light.color(base), Color32::BLACK);
        let mid = Effect { phase: 0.5, ..dark };
        assert_eq!(mid.color(base), base);
    }
}
