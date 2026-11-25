//! MCP stdio server for a host spec.

use std::process::ExitCode;

use egui_workbench_agent::{McpServer, Permissions};
use egui_workbench_host::Host;

fn main() -> ExitCode {
    let mut spec = None;
    let mut read_only = true;
    let mut allow = Vec::new();
    let mut args = std::env::args().skip(1);
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--spec" => spec = args.next(),
            "--read-only" => read_only = true,
            "--read-write" => read_only = false,
            "--allow" => allow.push(args.next().unwrap_or_default()),
            "--help" | "-h" => {
                println!(
                    "egui-workbench-mcp --spec app.json [--read-only|--read-write] [--allow id]...\n\
                     \nRead-only is the default. The server speaks MCP JSON-RPC on stdio.\n\
                     Destructive commands need the EGUI_WORKBENCH_CONFIRM environment variable;\n\
                     without it they are refused."
                );
                return ExitCode::SUCCESS;
            }
            other => {
                eprintln!("unknown argument `{other}`");
                return ExitCode::FAILURE;
            }
        }
    }
    let Some(spec) = spec else {
        eprintln!("--spec <path> is required");
        return ExitCode::FAILURE;
    };
    let host = match Host::load(&spec) {
        Ok(host) => host,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };
    let mut permissions = Permissions {
        read_only,
        ..Permissions::default()
    };
    if !allow.is_empty() {
        permissions = permissions.allow_commands(allow);
    }
    if !read_only {
        let sensitive = host
            .spec()
            .commands
            .iter()
            .filter(|command| command.category.eq_ignore_ascii_case("destructive"))
            .map(|command| command.id.clone())
            .collect::<Vec<_>>();
        permissions = match std::env::var("EGUI_WORKBENCH_CONFIRM") {
            Ok(token) => permissions.confirm(sensitive, token),
            Err(_) => permissions.require_confirmation(sensitive),
        };
    }
    match McpServer::new(host).with_permissions(permissions).serve_stdio() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
