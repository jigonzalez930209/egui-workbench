//! Repository tasks: scaffold an application, run the checks, refresh snapshots.
//!
//! ```bash
//! cargo xtask new ../my-app
//! cargo xtask new ../my-app --name my-app --title "My App"
//! cargo xtask check
//! cargo xtask style-lint
//! cargo xtask snapshots
//! ```

use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

mod style_lint;

fn main() -> Result<(), Box<dyn Error>> {
    let mut arguments = env::args().skip(1);
    let Some(task) = arguments.next() else {
        usage();
        return Ok(());
    };
    match task.as_str() {
        "new" => new(arguments),
        "check" => {
            run(&["fmt", "--all", "--check"], &[])?;
            style_lint()?;
            run(&["clippy", "--workspace", "--all-targets", "--", "-D", "warnings"], &[])?;
            run(&["doc", "--workspace", "--no-deps"], &[("RUSTDOCFLAGS", "-D warnings")])?;
            run(&["test", "--workspace"], &[])
        }
        "snapshots" => run(&["test", "--test", "snapshots"], &[("UPDATE_SNAPSHOTS", "1")]),
        "fmt" => run(&["fmt", "--all"], &[]),
        "style-lint" => style_lint(),
        "template-check" => check_template(),
        "-h" | "--help" | "help" => {
            usage();
            Ok(())
        }
        other => Err(format!("unknown task `{other}`; use --help").into()),
    }
}

fn style_lint() -> Result<(), Box<dyn Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let found = style_lint::violations(&root)?;
    if found.is_empty() {
        println!("style-lint: no literal radii or font sizes");
        return Ok(());
    }
    for violation in &found {
        eprintln!("{violation}");
    }
    Err(format!(
        "style-lint: {} literal value(s); read them from `egui_workbench::style`",
        found.len()
    )
    .into())
}

fn usage() {
    println!(
        "cargo xtask <task>\n\
         \n\
         new <path> [--name <crate>] [--title <title>]  create an application from templates/app\n\
         check                                          fmt --check, style-lint, clippy -D warnings, doc, test\n\
         style-lint                                     reject literal radii and font sizes in src/\n\
         snapshots                                      update the PNG references\n\
         template-check                                 generate the starter app and check it\n\
         fmt                                            format the workspace"
    );
}

fn new(mut arguments: impl Iterator<Item = String>) -> Result<(), Box<dyn Error>> {
    let Some(destination) = arguments.next() else {
        return Err("new needs a destination directory".into());
    };
    let mut name = None;
    let mut title = None;
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--name" => name = Some(arguments.next().ok_or("--name needs a value")?),
            "--title" => title = Some(arguments.next().ok_or("--title needs a value")?),
            other => return Err(format!("unknown option `{other}`").into()),
        }
    }

    let destination = PathBuf::from(destination);
    if destination.exists() {
        return Err(format!("{} already exists", destination.display()).into());
    }
    let crate_name = name.unwrap_or_else(|| default_name(&destination));
    validate_name(&crate_name)?;
    let app_title = title.unwrap_or_else(|| default_title(&crate_name));

    let template = Path::new(env!("CARGO_MANIFEST_DIR")).join("../templates/app");
    if !template.is_dir() {
        return Err(format!("template not found at {}", template.display()).into());
    }
    let workbench = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").canonicalize()?;
    copy_tree(&template, &destination)?;
    // The template manifest configures cargo-generate; a generated app does not use it.
    let template_manifest = destination.join("cargo-generate.toml");
    if template_manifest.is_file() {
        fs::remove_file(template_manifest)?;
    }
    for license in ["LICENSE-APACHE", "LICENSE-MIT"] {
        let source = workbench.join(license);
        if source.is_file() {
            fs::copy(&source, destination.join(license))?;
        }
    }
    let workbench_path = workbench.to_string_lossy().replace('\\', "/");
    let dependency = format!("path = \"{workbench_path}\",");
    replace_tree(
        &destination,
        &[
            ("__APP_NAME__", &crate_name),
            ("__APP_TITLE__", &app_title),
            ("__WORKBENCH_PATH__", &workbench_path),
            ("{{project-name}}", &crate_name),
            ("{{app_title}}", &app_title),
            (
                "{% if workbench_path != \"\" %}path = \"{{workbench_path}}\", {% else %}version = \"0.3\", {% endif %}",
                &dependency,
            ),
            ("{{workbench_path}}", &workbench_path),
        ],
    )?;

    println!("Created {} ({app_title})", destination.display());
    println!("Next:");
    println!("  cd {}", destination.display());
    println!("  cargo run");
    Ok(())
}

fn default_name(destination: &Path) -> String {
    destination.file_name().map_or_else(
        || "my-app".to_owned(),
        |name| {
            let sanitized = sanitize(&name.to_string_lossy());
            if sanitized.is_empty() {
                "my-app".to_owned()
            } else {
                sanitized
            }
        },
    )
}

fn sanitize(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for character in name.chars() {
        if character.is_ascii_alphanumeric() {
            out.push(character.to_ascii_lowercase());
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_matches('-').to_owned()
}

fn validate_name(name: &str) -> Result<(), Box<dyn Error>> {
    let valid = !name.is_empty()
        && name.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-' || character == '_'
        })
        && name.starts_with(|character: char| character.is_ascii_alphanumeric());
    if valid {
        Ok(())
    } else {
        Err(format!("`{name}` is not a valid crate name").into())
    }
}

fn default_title(name: &str) -> String {
    name.split(['-', '_'])
        .filter(|word| !word.is_empty())
        .map(|word| {
            let mut characters = word.chars();
            characters
                .next()
                .map_or_else(String::new, |first| first.to_uppercase().chain(characters).collect())
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn copy_tree(source: &Path, destination: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let name = entry.file_name();
        let from = entry.path();
        let to = destination.join(&name);
        if entry.file_type()?.is_dir() {
            copy_tree(&from, &to)?;
        } else {
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

fn replace_tree(directory: &Path, replacements: &[(&str, &str)]) -> Result<(), Box<dyn Error>> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            replace_tree(&path, replacements)?;
            continue;
        }
        let bytes = fs::read(&path)?;
        let Ok(text) = String::from_utf8(bytes) else {
            continue;
        };
        let mut replaced = text;
        for (from, to) in replacements {
            replaced = replaced.replace(from, to);
        }
        fs::write(&path, replaced)?;
    }
    Ok(())
}

fn check_template() -> Result<(), Box<dyn Error>> {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../target/template-check");
    if directory.exists() {
        fs::remove_dir_all(&directory)?;
    }
    new(vec![
        directory.display().to_string(),
        "--name".to_owned(),
        "template-check".to_owned(),
    ]
    .into_iter())?;
    run_in(&directory, &["fmt", "--check"], &[])?;
    run_in(&directory, &["clippy", "--all-targets", "--", "-D", "warnings"], &[])?;
    run_in(&directory, &["test"], &[])
}

fn run(arguments: &[&str], environment: &[(&str, &str)]) -> Result<(), Box<dyn Error>> {
    run_in(Path::new("."), arguments, environment)
}

fn run_in(directory: &Path, arguments: &[&str], environment: &[(&str, &str)]) -> Result<(), Box<dyn Error>> {
    let status = Command::new("cargo")
        .current_dir(directory)
        .args(arguments)
        .envs(environment.iter().copied())
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("cargo {} failed", arguments.join(" ")).into())
    }
}
