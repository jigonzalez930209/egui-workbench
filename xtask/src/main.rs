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
use std::process::{Command, Stdio};

mod style_lint;

fn main() -> Result<(), Box<dyn Error>> {
    let mut arguments = env::args().skip(1);
    let Some(task) = arguments.next() else {
        usage();
        return Ok(());
    };
    match task.as_str() {
        "new" => new(arguments),
        "check" => check(),
        "snapshots" => run(&["test", "--test", "snapshots"], &[("UPDATE_SNAPSHOTS", "1")]),
        "budgets" => run(
            &[
                "test",
                "--release",
                "--test",
                "data_budget",
                "--test",
                "stream_budget",
                "--",
                "--ignored",
            ],
            &[],
        ),
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

fn check() -> Result<(), Box<dyn Error>> {
    run(&["fmt", "--all", "--check"], &[])?;
    style_lint()?;
    run(&["clippy", "--workspace", "--all-targets", "--", "-D", "warnings"], &[])?;
    run(&["doc", "--workspace", "--no-deps"], &[("RUSTDOCFLAGS", "-D warnings")])?;
    run(&["test", "--workspace"], &[])?;
    check_template()?;

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let examples = root.join("docs-site/scripts/check-examples.sh");
    run_step("docs examples", "bash", &[&examples.to_string_lossy()], &[])?;
    let c_abi = root.join("crates/egui-workbench-c/check.sh");
    run_step("C ABI", "bash", &[&c_abi.to_string_lossy()], &[])?;

    if have("python3", &["-c", "import egui_workbench"]) {
        run_step("Python", "pytest", &["crates/egui-workbench-python/tests"], &[])?;
    } else {
        skip("Python", "the egui_workbench module (build it with maturin)");
    }
    if have("go", &["version"]) {
        let go = root.join("clients/go/check.sh");
        run_step("Go client", "bash", &[&go.to_string_lossy()], &[])?;
    } else {
        skip("Go client", "go");
    }
    if have("cargo", &["deny", "--version"]) {
        run_step("cargo deny", "cargo", &["deny", "check"], &[])?;
    } else {
        skip("cargo deny", "cargo-deny");
    }
    if have("cargo", &["semver-checks", "--version"]) {
        run_step("semver-checks", "cargo", &["semver-checks", "check-release"], &[])?;
    } else {
        skip("semver-checks", "cargo-semver-checks");
    }
    check_msrv()?;

    let wasm = root.join("crates/egui-workbench-wasm");
    if have("wasm-pack", &["--version"]) && have("chromedriver", &["--version"]) {
        run_step_in(
            &wasm,
            "wasm tests",
            "wasm-pack",
            &["test", "--headless", "--chrome"],
            &[],
        )?;
    } else {
        skip("wasm tests", "wasm-pack + chromedriver");
    }
    Ok(())
}

fn check_msrv() -> Result<(), Box<dyn Error>> {
    let installed = Command::new("rustup")
        .args(["toolchain", "list"])
        .output()
        .ok()
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .any(|line| line.starts_with("1.95"))
        })
        .unwrap_or(false);
    if !installed {
        skip("MSRV 1.95", "rustup toolchain 1.95");
        return Ok(());
    }
    run_step(
        "MSRV 1.95",
        "cargo",
        &["+1.95", "check", "--lib", "--all-features", "--locked"],
        &[],
    )
}

fn have(program: &str, arguments: &[&str]) -> bool {
    Command::new(program)
        .args(arguments)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn skip(step: &str, tool: &str) {
    println!("xtask check: skipping {step} ({tool} not found)");
}

fn run_step(step: &str, program: &str, arguments: &[&str], environment: &[(&str, &str)]) -> Result<(), Box<dyn Error>> {
    run_step_in(Path::new("."), step, program, arguments, environment)
}

fn run_step_in(
    directory: &Path,
    step: &str,
    program: &str,
    arguments: &[&str],
    environment: &[(&str, &str)],
) -> Result<(), Box<dyn Error>> {
    println!("xtask check: {step}");
    let status = Command::new(program)
        .current_dir(directory)
        .args(arguments)
        .envs(environment.iter().copied())
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{step} failed ({program} {})", arguments.join(" ")).into())
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
         check                                          fmt, style-lint, clippy, doc, tests, template, docs examples, C ABI, clients, deny, MSRV\n\
         budgets                                        run the ignored performance budgets in release mode\n\
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
    let dependency = format!("path = \"{workbench_path}\", ");
    replace_tree(
        &destination,
        &[
            ("__APP_NAME__", &crate_name),
            ("__APP_TITLE__", &app_title),
            ("__WORKBENCH_PATH__", &workbench_path),
            ("{{project-name}}", &crate_name),
            ("{{app_title}}", &app_title),
            (
                "{% if workbench_path != \"\" %}path = \"{{workbench_path}}\", {% else %}version = \"1\", {% endif %}",
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
