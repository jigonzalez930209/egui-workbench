//! Serve a HostSpec over WebSocket.

use std::process::ExitCode;

use egui_workbench_remote::RemoteServer;

fn main() -> ExitCode {
    let mut spec = None;
    let mut addr = "127.0.0.1:31337".to_owned();
    let mut args = std::env::args().skip(1);
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--spec" => spec = args.next(),
            "--addr" => addr = args.next().unwrap_or(addr),
            "--help" | "-h" => {
                println!("egui-workbench-remote --spec app.json [--addr 127.0.0.1:31337]");
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
    match RemoteServer::load(&spec, &addr).and_then(|server| server.serve()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
