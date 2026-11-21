//! Visual reference of the component layer in both light and dark themes.
//!
//! Refresh with `UPDATE_SNAPSHOTS=1 cargo test --test snapshots_components`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use egui_workbench::components::prelude::*;
use egui_workbench::egui;
use egui_workbench::{Icons, ThemeId};

#[derive(Default)]
struct Board {
    icons: Icons,
    text: String,
    area: String,
    toggle: bool,
    check: bool,
    mode: usize,
    value: f32,
}

fn draw(ui: &mut egui::Ui, board: &mut Board) {
    egui::CentralPanel::default().show(ui, |ui| board_contents(ui, board));
}

fn board_contents(ui: &mut egui::Ui, board: &mut Board) {
    ui.heading("Component board");
    ui.add_space(8.0);
    ui.horizontal_wrapped(|ui| {
        for (label, variant) in [
            ("Primary", ButtonVariant::Primary),
            ("Secondary", ButtonVariant::Secondary),
            ("Outline", ButtonVariant::Outline),
            ("Ghost", ButtonVariant::Ghost),
            ("Destructive", ButtonVariant::Destructive),
            ("Link", ButtonVariant::Link),
        ] {
            Button::new(label).variant(variant).show(ui);
        }
    });
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        Button::new("Disabled").enabled(false).show(ui);
        IconButton::new("settings", "Settings").show(ui, &mut board.icons);
        IconButton::new("plus", "New")
            .size(IconButtonSize::Tool)
            .show(ui, &mut board.icons);
        IconButton::new("refresh", "Refresh")
            .size(IconButtonSize::Small)
            .show(ui, &mut board.icons);
    });
    ui.add_space(10.0);
    ui.horizontal_top(|ui| {
        ui.vertical(|ui| {
            TextField::new(&mut board.text)
                .label("Name")
                .hint("Untitled")
                .width(260.0)
                .show(ui);
            ui.add_space(6.0);
            Switch::new(&mut board.toggle).label("Preview").show(ui);
            Checkbox::new(&mut board.check).label("Word wrap").show(ui);
        });
        ui.add_space(20.0);
        ui.vertical(|ui| {
            let options = [
                (0, "Automatic".to_owned()),
                (1, "Light".to_owned()),
                (2, "Dark".to_owned()),
            ];
            Select::new("Mode", &mut board.mode, &options).show(ui);
            ui.add_space(6.0);
            Stepper::new(&mut board.value, 0.0..=2.0, 0.1)
                .decimals(1)
                .suffix("×")
                .show(ui, &mut board.icons);
            ui.add_space(6.0);
            TextArea::new(&mut board.area)
                .label("Notes")
                .rows(2)
                .width(240.0)
                .show(ui);
        });
    });
    ui.add_space(10.0);
    ui.horizontal_wrapped(|ui| {
        Badge::new("Saved").show(ui);
        Badge::new("Accent").tone(Tone::Accent).show(ui);
        Badge::new("Warning").tone(Tone::Warning).show(ui);
        Badge::new("Danger").tone(Tone::Danger).show(ui);
        Badge::new("Success")
            .tone(Tone::Custom(ui.visuals().hyperlink_color))
            .dot(true)
            .show(ui);
    });
    ui.add_space(10.0);
    Card::new()
        .title("Card")
        .description("Raised content with the shared border.")
        .width(360.0)
        .show(ui, |ui| {
            Label::new("Card body").tone(Tone::Muted).show(ui);
            Button::new("Action")
                .variant(ButtonVariant::Outline)
                .size(Size::Sm)
                .show(ui);
        });
}

fn board<'a>(id: ThemeId) -> egui_kittest::Harness<'a, Board> {
    common::themed(id, [760.0, 560.0], Board::default(), draw)
}

#[test]
fn components_github_light() {
    let mut harness = board(ThemeId::GithubLight);
    harness.run();
    harness.snapshot("components-github-light");
}

#[test]
fn components_github_dark() {
    let mut harness = board(ThemeId::GithubDark);
    harness.run();
    harness.snapshot("components-github-dark");
}

fn styled<'a>(
    theme: egui_workbench::Theme,
    style: egui_workbench::style::WorkbenchStyle,
) -> egui_kittest::Harness<'a, Board> {
    egui_kittest::Harness::builder()
        .with_size(egui::vec2(760.0, 600.0))
        .with_max_steps(64)
        .with_context_setup(move |ctx| {
            egui_workbench::style::set(ctx, style);
            egui_workbench::theme::apply_tokens(ctx, &theme);
        })
        .build_ui_state(draw, Board::default())
}

#[test]
fn components_compact_sharp() {
    use egui_workbench::style::{Density, WorkbenchStyle};
    let mut harness = styled(
        egui_workbench::Theme::light(),
        WorkbenchStyle::sharp().with_density(Density::Compact),
    );
    harness.run();
    harness.snapshot("components-compact-sharp");
}

#[test]
fn components_comfortable_rounded() {
    use egui_workbench::style::{Density, WorkbenchStyle};
    let mut harness = styled(
        egui_workbench::Theme::dark(),
        WorkbenchStyle::rounded().with_density(Density::Comfortable),
    );
    harness.run();
    harness.snapshot("components-comfortable-rounded");
}

#[test]
fn components_large_text_borderless() {
    use egui_workbench::style::WorkbenchStyle;
    let mut harness = styled(
        egui_workbench::Theme::light(),
        WorkbenchStyle {
            text_scale: 1.25,
            border_width: 0.0,
            ..WorkbenchStyle::default()
        },
    );
    harness.run();
    harness.snapshot("components-large-text-borderless");
}

#[test]
fn components_custom_accent() {
    let brand = egui_workbench::Theme::dark().with_accent(egui::Color32::from_rgb(0xe0, 0x6c, 0x2b));
    let mut harness = styled(brand, egui_workbench::style::WorkbenchStyle::default());
    harness.run();
    harness.snapshot("components-custom-accent");
}

#[test]
fn toast_with_action() {
    // Locks the label of an action button centered inside a top-aligned toast row.
    let mut toasts = Toasts::new();
    toasts.add(
        Toast::new(ToastKind::Error, "Could not save the file")
            .description("Permission denied")
            .action("Retry")
            .persistent(),
    );
    let mut harness = common::themed(
        ThemeId::GithubLight,
        [420.0, 120.0],
        (Icons::new(), toasts),
        |ui, (icons, toasts)| {
            egui::CentralPanel::default().show(ui, |_| {});
            toasts.show(ui.ctx(), icons);
        },
    );
    harness.run_steps(20);
    harness.snapshot("toast-with-action");
}

struct IconBrowser {
    icons: Icons,
    names: Vec<String>,
    selected: Option<String>,
}

fn draw_icon_browser(ui: &mut egui::Ui, state: &mut IconBrowser) {
    egui::CentralPanel::default().show(ui, |ui| {
        ui.heading("Icon browser");
        ui.label("Lucide matches for “file”, virtualized and scaled with the window.");
        ui.add_space(8.0);
        let IconBrowser { icons, names, selected } = state;
        if let Some(index) = IconGrid::new("docs-icon-browser", names)
            .prefix("lucide-")
            .size(IconSize::Auto)
            .selected(selected.as_deref())
            .show(ui, icons)
        {
            *selected = Some(names[index].clone());
        }
    });
}

fn icon_browser<'a>(id: ThemeId) -> egui_kittest::Harness<'a, IconBrowser> {
    egui_kittest::Harness::builder()
        .with_size(egui::vec2(720.0, 480.0))
        .with_max_steps(64)
        .with_context_setup(move |ctx| egui_workbench::theme::apply_id(ctx, id))
        .build_ui_state(
            draw_icon_browser,
            IconBrowser {
                icons: Icons::new(),
                names: Icons::lucide_search("file").into_iter().take(24).collect(),
                selected: None,
            },
        )
}

#[test]
fn icon_grid_github_light() {
    let mut harness = icon_browser(ThemeId::GithubLight);
    harness.run();
    harness.snapshot("icon-grid-github-light");
}

#[test]
fn icon_grid_github_dark() {
    let mut harness = icon_browser(ThemeId::GithubDark);
    harness.run();
    harness.snapshot("icon-grid-github-dark");
}
