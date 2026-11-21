//! Themes, the undecorated window, edge resizing, and window commands.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use egui_workbench::egui::{self, ResizeDirection, ViewportCommand, pos2};
use egui_workbench::{Theme, ThemeId, ThemeMode, WindowAction, theme};

#[test]
fn every_theme_sets_both_egui_styles() {
    for id in ThemeId::ALL {
        let ctx = egui::Context::default();
        theme::apply_id(&ctx, id);
        let tokens = Theme::for_id(id);
        for mode in [egui::Theme::Light, egui::Theme::Dark] {
            let style = ctx.style_of(mode);
            let visuals = &style.visuals;
            assert_eq!(visuals.panel_fill, tokens.background, "{id:?}");
            assert_eq!(visuals.window_fill, tokens.surface, "{id:?}");
            assert_eq!(visuals.extreme_bg_color, tokens.sunken, "{id:?}");
            assert_eq!(visuals.faint_bg_color, tokens.hover, "{id:?}");
            assert_eq!(visuals.hyperlink_color, tokens.link, "{id:?}");
            assert_eq!(visuals.selection.bg_fill, tokens.selection, "{id:?}");
            assert_eq!(visuals.override_text_color, Some(tokens.text), "{id:?}");
            assert_eq!(visuals.widgets.hovered.weak_bg_fill, tokens.hover, "{id:?}");
            assert_eq!(visuals.widgets.inactive.weak_bg_fill, tokens.surface, "{id:?}");
            assert_eq!(visuals.widgets.active.bg_fill, tokens.accent, "{id:?}");
            assert_eq!(visuals.widgets.inactive.bg_stroke.color, tokens.border, "{id:?}");
            assert_eq!(visuals.widgets.hovered.bg_stroke.color, tokens.focus, "{id:?}");
            assert_eq!(visuals.widgets.active.bg_stroke.width, 2.0, "{id:?}");
            assert_eq!(visuals.window_corner_radius, egui::CornerRadius::same(8));
            assert_eq!(visuals.dark_mode, id.mode() == ThemeMode::Dark, "{id:?}");
            assert_eq!(style.spacing.item_spacing, egui::vec2(8.0, 4.0));
            assert_eq!(style.spacing.button_padding, egui::vec2(8.0, 3.0));
            assert_eq!(style.spacing.interact_size, egui::vec2(26.0, 24.0));
            assert_eq!(style.spacing.scroll.bar_width, 6.0);
            assert_eq!(style.animation_time, theme::ANIMATION_TIME);
            let size = |text_style: egui::TextStyle| style.text_styles[&text_style].size;
            assert_eq!(
                [
                    size(egui::TextStyle::Body),
                    size(egui::TextStyle::Button),
                    size(egui::TextStyle::Small),
                    size(egui::TextStyle::Monospace),
                    size(egui::TextStyle::Heading),
                ],
                [14.0, 12.5, 11.0, 13.0, 20.0]
            );
        }
    }
}

#[test]
fn reduced_motion_stops_and_restores_animation() {
    let ctx = egui::Context::default();
    theme::apply_id(&ctx, ThemeId::Dracula);
    theme::set_reduced_motion(&ctx, true);
    assert_eq!(ctx.style_of(egui::Theme::Dark).animation_time, 0.0);
    theme::set_reduced_motion(&ctx, false);
    assert_eq!(ctx.style_of(egui::Theme::Dark).animation_time, theme::ANIMATION_TIME);
}

#[test]
fn following_the_system_keeps_the_family() {
    for (id, dark, expected) in [
        (ThemeId::GithubLight, true, ThemeId::GithubDark),
        (ThemeId::GithubDarkDimmed, false, ThemeId::GithubLight),
        (ThemeId::Alucard, true, ThemeId::Dracula),
        (ThemeId::DraculaSoft, false, ThemeId::Alucard),
        (ThemeId::OneDarkProLight, true, ThemeId::OneDarkPro),
        (ThemeId::OneDarkProDarker, false, ThemeId::OneDarkProLight),
    ] {
        assert_eq!(theme::effective_id(id, true, dark), expected, "{id:?}");
        assert_eq!(theme::effective_id(id, false, dark), id, "{id:?}");
    }
}

#[test]
fn apply_theme_reads_the_desktop_preference() {
    let mut harness = common::harness([200.0, 100.0], None, |ui, applied: &mut Option<ThemeId>| {
        *applied = Some(theme::apply_theme(ui.ctx(), ThemeId::Alucard, true));
    });
    harness.input_mut().system_theme = Some(egui::Theme::Dark);
    harness.run_steps(2);
    assert_eq!(*harness.state(), Some(ThemeId::Dracula));
    harness.input_mut().system_theme = Some(egui::Theme::Light);
    harness.run_steps(2);
    assert_eq!(*harness.state(), Some(ThemeId::Alucard));
}

#[test]
fn theme_ids_round_trip_through_settings() {
    for id in ThemeId::ALL {
        assert_eq!(ThemeId::from_setting(id.as_setting()), id);
        assert!(!id.label().is_empty() && !id.family().is_empty());
    }
    assert_eq!(ThemeId::from_setting("dark"), ThemeId::GithubDark);
    assert_eq!(ThemeId::from_setting("nonsense"), ThemeId::GithubLight);
}

#[test]
fn native_options_describe_an_undecorated_window() {
    let options = egui_workbench::native_options("Demo");
    let viewport = &options.viewport;
    assert_eq!(viewport.decorations, Some(false));
    assert_eq!(viewport.inner_size, Some(egui::vec2(980.0, 720.0)));
    assert_eq!(viewport.min_inner_size, Some(egui::vec2(640.0, 420.0)));
    assert_eq!(viewport.title.as_deref(), Some("Demo"));
    assert!(matches!(options.renderer, egui_workbench::eframe::Renderer::Glow));
    let sized = egui_workbench::native_options_sized("Demo", [800.0, 600.0], [500.0, 300.0]);
    assert_eq!(sized.viewport.inner_size, Some(egui::vec2(800.0, 600.0)));
}

#[test]
fn pressing_a_window_edge_starts_a_resize() {
    let mut harness = common::harness([600.0, 400.0], (), |ui, ()| {
        egui_workbench::handle_window_edges(ui.ctx());
    });
    harness.run();
    for (pos, expected) in [
        (pos2(1.0, 200.0), Some(ResizeDirection::West)),
        (pos2(598.0, 200.0), Some(ResizeDirection::East)),
        (pos2(300.0, 398.0), Some(ResizeDirection::South)),
        (pos2(1.0, 398.0), Some(ResizeDirection::SouthWest)),
        (pos2(598.0, 398.0), Some(ResizeDirection::SouthEast)),
        (pos2(300.0, 1.0), None),
        (pos2(300.0, 200.0), None),
    ] {
        common::press_at(&harness, pos);
        harness.step();
        let resize = common::commands(&harness)
            .into_iter()
            .find_map(|command| match command {
                ViewportCommand::BeginResize(direction) => Some(direction),
                _ => None,
            });
        assert_eq!(resize, expected, "{pos:?}");
        common::release_at(&harness, pos);
        harness.step();
    }
}

#[test]
fn window_actions_send_viewport_commands() {
    for (action, expected) in [
        (WindowAction::Minimize, ViewportCommand::Minimized(true)),
        (WindowAction::ToggleMaximize, ViewportCommand::Maximized(true)),
        (WindowAction::Close, ViewportCommand::Close),
    ] {
        // The harness runs a first frame when it is built; queue the action after that.
        let mut harness = common::harness([200.0, 100.0], None, |ui, pending: &mut Option<WindowAction>| {
            if let Some(action) = pending.take() {
                egui_workbench::apply_window_action(ui.ctx(), action);
            }
        });
        *harness.state_mut() = Some(action);
        harness.step();
        assert!(common::commands(&harness).contains(&expected), "{action:?}");
    }
}
