//! The live documentation scenes (`examples/scenes`): every scene renders at phone and
//! desktop widths without panicking, every `<LiveDemo>` in the docs names a real scene,
//! and the first frame of every embedded demo matches its reference image.
//!
//! The references live in `tests/snapshots/doc-scenes/`. Refresh them with
//! `UPDATE_SNAPSHOTS=1 cargo test --test doc_scenes` after an intended visual change.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

#[path = "../examples/scenes/scenes.rs"]
mod scenes;

use std::path::{Path, PathBuf};

use egui_kittest::{Harness, SnapshotOptions, SnapshotResults};
use egui_workbench::{ThemeId, egui};
use scenes::{Params, SCENES, STYLES, SceneApp};

/// The reference width: about the docs column.
const POSTER_WIDTH: f32 = 720.0;

/// One `<LiveDemo …/>` tag.
#[derive(Clone, Debug, PartialEq)]
struct Usage {
    page: PathBuf,
    scene: String,
    theme: Option<String>,
    style: Option<String>,
    accent: Option<String>,
    height: f32,
}

impl Usage {
    /// The reference image stem.
    fn key(&self) -> String {
        let mut key = self.scene.clone();
        for part in [&self.style, &self.theme, &self.accent].into_iter().flatten() {
            if part != "default" {
                key.push('-');
                key.push_str(part.trim_start_matches('#'));
            }
        }
        key
    }

    /// `(file stem, theme)` of every reference this tag needs.
    fn posters(&self) -> Vec<(String, ThemeId)> {
        match &self.theme {
            Some(theme) => vec![(self.key(), ThemeId::from_setting(theme))],
            None => vec![
                (format!("{}-light", self.key()), ThemeId::GithubLight),
                (format!("{}-dark", self.key()), ThemeId::GithubDark),
            ],
        }
    }

    fn params(&self, theme: ThemeId) -> Params {
        Params {
            scene: self.scene.clone(),
            theme,
            style: self.style.clone().unwrap_or_else(|| "default".to_owned()),
            accent: self.accent.as_deref().and_then(scenes::parse_hex),
            embedded: false,
        }
    }
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn attribute(tag: &str, name: &str) -> Option<String> {
    let needle = format!(" {name}=\"");
    let start = tag.find(&needle)? + needle.len();
    let end = tag[start..].find('"')?;
    Some(tag[start..start + end].to_owned())
}

fn collect(directory: &Path, pages: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy();
        if path.is_dir() && !name.starts_with('.') && name != "node_modules" && name != "public" {
            collect(&path, pages);
        } else if name.ends_with(".md") && name != "README.md" {
            pages.push(path);
        }
    }
}

/// Every `<LiveDemo>` tag in the docs site.
fn usages() -> Vec<Usage> {
    let mut pages = Vec::new();
    collect(&root().join("docs-site"), &mut pages);
    pages.sort();
    let mut found = Vec::new();
    for page in pages {
        let text = std::fs::read_to_string(&page).unwrap();
        let mut rest = text.as_str();
        while let Some(start) = rest.find("<LiveDemo") {
            // Only real tags (`<LiveDemo` + whitespace + attributes), not prose like
            // `` `<LiveDemo>` ``.
            if !rest[start + "<LiveDemo".len()..].starts_with(char::is_whitespace) {
                rest = &rest[start + 1..];
                continue;
            }
            let end = rest[start..].find("/>").map_or(rest.len(), |end| start + end);
            let tag = rest[start..end].replace(['\n', '\t'], " ");
            found.push(Usage {
                page: page.clone(),
                scene: attribute(&tag, "scene").unwrap_or_default(),
                theme: attribute(&tag, "theme"),
                style: attribute(&tag, "look"),
                accent: attribute(&tag, "accent"),
                height: attribute(&tag, "height")
                    .and_then(|value| value.parse().ok())
                    .unwrap_or(420.0),
            });
            rest = &rest[end..];
        }
    }
    found
}

fn harness<'a>(params: Params, size: egui::Vec2) -> Harness<'a, SceneApp> {
    Harness::builder()
        .with_size(size)
        .with_step_dt(1.0 / 60.0)
        .with_max_steps(400)
        .build_ui_state(|ui, app: &mut SceneApp| app.frame(ui), SceneApp::new(params))
}

#[test]
fn every_scene_renders_at_phone_and_desktop_widths() {
    for scene in SCENES {
        for (size, theme) in [
            (egui::vec2(360.0, 560.0), ThemeId::GithubLight),
            (egui::vec2(1100.0, 640.0), ThemeId::Dracula),
        ] {
            let params = Params {
                scene: scene.to_owned(),
                theme,
                ..Params::default()
            };
            let mut harness = harness(params, size);
            harness.run_steps(12);
        }
    }
}

#[test]
fn every_style_preset_renders() {
    for style in STYLES {
        let params = Params {
            scene: "components".to_owned(),
            style: style.to_owned(),
            ..Params::default()
        };
        harness(params, egui::vec2(720.0, 560.0)).run_steps(4);
    }
}

#[test]
fn query_strings_parse_and_fall_back() {
    let params = Params::parse("?scene=plot-live&theme=dracula&style=compact-sharp&accent=e06c2b");
    assert_eq!(params.scene, "plot-live");
    assert_eq!(params.theme, ThemeId::Dracula);
    assert_eq!(params.style, "compact-sharp");
    assert_eq!(params.accent, Some(egui::Color32::from_rgb(0xe0, 0x6c, 0x2b)));
    let fallback = Params::parse("scene=nope&style=huge&accent=xyz");
    assert_eq!(fallback, Params::default());
}

#[test]
fn every_live_demo_names_a_known_scene() {
    let usages = usages();
    assert!(
        usages.len() >= 20,
        "expected the docs to embed live demos, found {}",
        usages.len()
    );
    for usage in &usages {
        assert!(
            SCENES.contains(&usage.scene.as_str()),
            "{}: unknown scene {:?}",
            usage.page.display(),
            usage.scene
        );
        if let Some(style) = &usage.style {
            assert!(
                STYLES.contains(&style.as_str()),
                "{}: unknown look {style:?}",
                usage.page.display()
            );
        }
        if let Some(theme) = &usage.theme {
            assert_eq!(
                ThemeId::from_setting(theme).as_setting(),
                theme,
                "{}: unknown theme",
                usage.page.display()
            );
        }
    }
}

#[test]
fn live_demo_first_frames_match_their_references() {
    let output = root().join("tests/snapshots/doc-scenes");
    let options = SnapshotOptions::new().output_path(&output);
    let mut seen = Vec::new();
    let mut results = SnapshotResults::new();
    for usage in usages() {
        for (stem, theme) in usage.posters() {
            if seen.contains(&stem) {
                continue;
            }
            seen.push(stem.clone());
            let mut harness = harness(usage.params(theme), egui::vec2(POSTER_WIDTH, usage.height));
            // Live data streams for two seconds; static scenes settle in a few frames.
            harness.run_steps(if matches!(usage.scene.as_str(), "plot-live" | "charts") {
                150
            } else {
                8
            });
            harness.snapshot_options(&stem, &options);
            results.extend_harness(&mut harness);
        }
    }
    drop(results);
}
