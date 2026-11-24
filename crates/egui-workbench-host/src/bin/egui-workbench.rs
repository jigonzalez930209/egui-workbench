//! The egui-workbench sidecar: run a spec, or speak the framed host protocol on stdio.

use std::io::{self, Write};

use egui_workbench_host::{Host, HostSpec, Message, PROTOCOL_VERSION, SPEC_VERSION, read_frame, write_frame};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut spec_path = None;
    let mut stdio = false;
    let mut schema = false;
    let mut args = std::env::args().skip(1);
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--spec" => spec_path = Some(args.next().ok_or("--spec needs a path")?),
            "--stdio" | "--headless" => stdio = true,
            "--schema" => schema = true,
            "--version" => {
                println!("egui-workbench {PROTOCOL_VERSION} spec {SPEC_VERSION}");
                return Ok(());
            }
            "--help" | "-h" => {
                println!(
                    "egui-workbench --spec app.json [--stdio]\n\
                     \n\
                     --spec <path>   load a HostSpec JSON file (hot reloaded while running)\n\
                     --stdio         read framed protocol messages from stdin, write to stdout\n\
                     --schema        print the HostSpec JSON Schema and exit\n\
                     --version       print the protocol and spec versions"
                );
                return Ok(());
            }
            other => return Err(format!("unknown argument `{other}`").into()),
        }
    }

    if schema {
        println!("{}", serde_json::to_string_pretty(&HostSpec::schema())?);
        return Ok(());
    }

    let host = match spec_path {
        Some(path) => Host::load(path)?,
        None => Host::new(HostSpec::default())?,
    };

    if stdio {
        serve_stdio(host)?;
    } else {
        host.run()?;
    }
    Ok(())
}

fn serve_stdio(mut host: Host) -> Result<(), Box<dyn std::error::Error>> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let hello = Message::Hello {
        version: PROTOCOL_VERSION,
        capabilities: vec!["patches".to_owned(), "data".to_owned(), "schema".to_owned()],
    };
    write_frame(&mut stdout, &hello)?;
    stdout.flush()?;
    loop {
        let message = match read_frame(stdin.lock()) {
            Ok(message) => message,
            Err(egui_workbench_host::FrameError::Io(error)) if error.contains("unexpected end") => break,
            Err(error) => return Err(error.into()),
        };
        let response = host.handle(message);
        write_frame(&mut stdout, &response)?;
        stdout.flush()?;
        for event in host.poll_events() {
            write_frame(&mut stdout, &Message::Event { event })?;
            stdout.flush()?;
        }
    }
    Ok(())
}
