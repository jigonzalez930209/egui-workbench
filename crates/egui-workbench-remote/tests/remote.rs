//! The WebSocket handshake and the 0.11 request/response path.

use std::io::{BufRead, BufReader, Cursor, Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::time::Duration;

use egui_workbench_host::{
    Host, HostSpec, Message, Patch, Request, Response, SPEC_VERSION, WidgetSpec, read_frame, write_frame,
};
use egui_workbench_remote::RemoteServer;
use serde_json::json;

fn spec(text: &str) -> HostSpec {
    HostSpec {
        version: SPEC_VERSION,
        widgets: vec![WidgetSpec::Label {
            text: text.to_owned(),
            size: 13.0,
            tone: "default".to_owned(),
            strong: false,
        }],
        ..HostSpec::default()
    }
}

fn read_frame_payload(stream: &mut impl Read) -> Vec<u8> {
    let mut header = [0u8; 2];
    stream.read_exact(&mut header).expect("frame header");
    let length = match header[1] & 0x7f {
        126 => {
            let mut bytes = [0u8; 2];
            stream.read_exact(&mut bytes).expect("length");
            u16::from_be_bytes(bytes) as usize
        }
        127 => {
            let mut bytes = [0u8; 8];
            stream.read_exact(&mut bytes).expect("length");
            u64::from_be_bytes(bytes) as usize
        }
        length => length as usize,
    };
    let mut payload = vec![0u8; length];
    stream.read_exact(&mut payload).expect("payload");
    payload
}

fn send_client_frame(stream: &mut impl Write, payload: &[u8]) {
    stream.write_all(&[0x82]).expect("opcode");
    stream.write_all(&[0x80 | 126]).expect("length marker");
    stream.write_all(&(payload.len() as u16).to_be_bytes()).expect("length");
    let mask = [7u8, 1, 9, 3];
    stream.write_all(&mask).expect("mask");
    let masked: Vec<u8> = payload
        .iter()
        .enumerate()
        .map(|(index, byte)| byte ^ mask[index % 4])
        .collect();
    stream.write_all(&masked).expect("payload");
    stream.flush().expect("flush");
}

fn message(payload: &[u8]) -> Message {
    read_frame(Cursor::new(payload.to_vec())).expect("message")
}

#[test]
fn handshake_patch_and_command_over_websocket() {
    let host = Host::new(spec("before")).expect("host");
    let server = RemoteServer::new(host, "127.0.0.1:0")
        .expect("bind")
        .on_command(|host, id| {
            if id == "go" {
                let _ = host.apply(&Patch::Set {
                    path: "/widgets/0/text".to_owned(),
                    value: json!("after"),
                });
            }
        });
    let addr = server.local_addr().expect("addr");
    std::thread::spawn(move || {
        let _ = server.serve();
    });

    let mut stream = TcpStream::connect(addr).expect("connect");
    stream.set_read_timeout(Some(Duration::from_secs(5))).expect("timeout");
    stream
        .write_all(
            b"GET / HTTP/1.1\r\nHost: localhost\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\nSec-WebSocket-Version: 13\r\n\r\n",
        )
        .expect("handshake");
    let mut reader = BufReader::new(stream.try_clone().expect("clone"));
    let mut response = String::new();
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).expect("header");
        if line == "\r\n" || line.is_empty() {
            break;
        }
        response.push_str(&line);
    }
    assert!(response.contains("101 Switching Protocols"), "{response}");
    assert!(response.contains("Sec-WebSocket-Accept:"), "{response}");

    let hello = message(&read_frame_payload(&mut reader));
    assert!(matches!(hello, Message::Hello { .. }));
    let initial = message(&read_frame_payload(&mut reader));
    assert!(matches!(
        initial,
        Message::Response {
            response: Response::Spec { .. },
            ..
        }
    ));

    let mut bytes = Vec::new();
    write_frame(
        &mut bytes,
        &Message::Request {
            id: 1,
            request: Request::ApplyPatch {
                patch: Patch::Set {
                    path: "/widgets/0/text".to_owned(),
                    value: json!("patched"),
                },
            },
        },
    )
    .expect("encode");
    send_client_frame(&mut stream, &bytes);
    let patched = message(&read_frame_payload(&mut reader));
    let Message::Response {
        id: 1,
        response: Response::Spec { spec },
    } = patched
    else {
        panic!("unexpected patch response: {patched:?}");
    };
    assert_eq!(spec["widgets"][0]["text"], "patched");

    let mut bytes = Vec::new();
    write_frame(
        &mut bytes,
        &Message::Request {
            id: 2,
            request: Request::RunCommand { id: "go".to_owned() },
        },
    )
    .expect("encode");
    send_client_frame(&mut stream, &bytes);
    let ran = message(&read_frame_payload(&mut reader));
    let Message::Response {
        id: 2,
        response: Response::Spec { spec },
    } = ran
    else {
        panic!("unexpected command response: {ran:?}");
    };
    assert_eq!(spec["widgets"][0]["text"], "after");
    let _ = Arc::new(());
}
