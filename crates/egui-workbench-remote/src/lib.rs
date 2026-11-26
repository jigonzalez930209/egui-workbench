//! The remote UI server.
//!
//! A [`Host`] runs on the server; a browser renders the spec with the wasm host. Messages
//! are the versioned [`egui_workbench_host::Message`] values inside binary WebSocket
//! frames, each prefixed with a four-byte big-endian length. The default bind address is
//! `127.0.0.1`; put authentication and TLS in front of it when exposing it.

use std::io::{BufRead, BufReader, Cursor, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use egui_workbench_host::{
    FrameError, Host, HostError, Message, PROTOCOL_VERSION, Request, Response, read_frame, write_frame,
};
use sha1::{Digest, Sha1};

/// A command callback: the server holds the host lock while it runs.
pub type CommandHandler = Arc<dyn Fn(&mut Host, &str) + Send + Sync>;

/// A remote server with a bound listener.
pub struct RemoteServer {
    listener: TcpListener,
    host: Arc<Mutex<Host>>,
    handler: CommandHandler,
    allowed_origins: Vec<String>,
}

/// Most connections served at once. Further sockets are refused instead of spawning an
/// unbounded number of threads.
const MAX_CONNECTIONS: usize = 32;

/// Most bytes and lines read before the WebSocket upgrade is complete.
const MAX_HANDSHAKE_BYTES: u64 = 8 * 1024;
const MAX_HEADER_LINES: usize = 64;

/// Time a client has to finish the WebSocket upgrade.
const HANDSHAKE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);
/// Time a write may block on a client that stopped reading.
const WRITE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);
/// Largest WebSocket frame payload accepted.
const MAX_FRAME_BYTES: u64 = 64 * 1024 * 1024;

impl RemoteServer {
    /// Bind `addr` and serve `host`. Use port `0` in tests to pick a free port.
    ///
    /// # Errors
    /// When the address cannot be bound.
    pub fn new(host: Host, addr: impl ToSocketAddrs) -> Result<Self, RemoteError> {
        let listener = TcpListener::bind(addr).map_err(|error| RemoteError::Io(error.to_string()))?;
        Ok(Self {
            listener,
            host: Arc::new(Mutex::new(host)),
            handler: Arc::new(|_, _| {}),
            allowed_origins: Vec::new(),
        })
    }

    /// Load a spec file and bind `addr`.
    ///
    /// # Errors
    /// When the file cannot be read or the address cannot be bound.
    pub fn load(path: impl AsRef<std::path::Path>, addr: impl ToSocketAddrs) -> Result<Self, RemoteError> {
        let host = Host::load(path).map_err(RemoteError::Host)?;
        Self::new(host, addr)
    }

    /// Install the command handler. It runs on the connection thread with the host lock.
    pub fn on_command(mut self, handler: impl Fn(&mut Host, &str) + Send + Sync + 'static) -> Self {
        self.handler = Arc::new(handler);
        self
    }

    /// Accept a browser `Origin` value in the WebSocket handshake.
    ///
    /// Native clients send no `Origin` and are always accepted. A request that carries an
    /// `Origin` not added here is refused with `403`, which blocks cross-site WebSocket
    /// hijacking from a page the user has open.
    pub fn allow_origin(mut self, origin: impl Into<String>) -> Self {
        self.allowed_origins.push(origin.into());
        self
    }

    /// The address the listener is bound to.
    ///
    /// # Errors
    /// When the local address cannot be read.
    pub fn local_addr(&self) -> Result<SocketAddr, RemoteError> {
        self.listener
            .local_addr()
            .map_err(|error| RemoteError::Io(error.to_string()))
    }

    /// Accept connections until the process stops.
    ///
    /// # Errors
    /// When accepting a connection fails.
    pub fn serve(self) -> Result<(), RemoteError> {
        let connections = Arc::new(AtomicUsize::new(0));
        let allowed_origins = Arc::new(self.allowed_origins);
        for stream in self.listener.incoming() {
            let stream = stream.map_err(|error| RemoteError::Io(error.to_string()))?;
            if connections.load(Ordering::SeqCst) >= MAX_CONNECTIONS {
                let mut stream = stream;
                let _ = stream
                    .write_all(b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
                continue;
            }
            connections.fetch_add(1, Ordering::SeqCst);
            let host = Arc::clone(&self.host);
            let handler = Arc::clone(&self.handler);
            let origins = Arc::clone(&allowed_origins);
            let guard = ConnectionGuard(Arc::clone(&connections));
            std::thread::spawn(move || {
                let _guard = guard;
                if let Err(error) = serve_connection(stream, host, handler, &origins) {
                    eprintln!("remote connection ended: {error}");
                }
            });
        }
        Ok(())
    }
}

struct ConnectionGuard(Arc<AtomicUsize>);

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}

/// Why a remote server or connection failed.
#[derive(Debug)]
#[non_exhaustive]
pub enum RemoteError {
    /// A host operation failed.
    Host(HostError),
    /// The socket or handshake failed.
    Io(String),
    /// A WebSocket frame was malformed.
    WebSocket(String),
    /// A protocol message was malformed.
    Protocol(String),
}

impl std::fmt::Display for RemoteError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Host(error) => write!(formatter, "{error}"),
            Self::Io(error) => formatter.write_str(error),
            Self::WebSocket(error) => write!(formatter, "websocket: {error}"),
            Self::Protocol(error) => write!(formatter, "protocol: {error}"),
        }
    }
}

impl std::error::Error for RemoteError {}

impl From<HostError> for RemoteError {
    fn from(error: HostError) -> Self {
        Self::Host(error)
    }
}

fn serve_connection(
    stream: TcpStream,
    host: Arc<Mutex<Host>>,
    handler: CommandHandler,
    allowed_origins: &[String],
) -> Result<(), RemoteError> {
    // A client that opens a socket and goes quiet must not hold a connection slot.
    stream.set_read_timeout(Some(HANDSHAKE_TIMEOUT)).map_err(io)?;
    stream.set_write_timeout(Some(WRITE_TIMEOUT)).map_err(io)?;
    let mut reader = BufReader::new(stream.try_clone().map_err(io)?);
    let mut writer = stream;
    handshake(&mut reader, &mut writer, allowed_origins)?;
    writer.set_read_timeout(None).map_err(io)?;

    write_message(
        &mut writer,
        &Message::Hello {
            version: PROTOCOL_VERSION,
            capabilities: vec!["patches".to_owned(), "data".to_owned(), "schema".to_owned()],
        },
    )?;
    write_spec(&mut writer, &host)?;

    while let Some(frame) = read_websocket(&mut reader)? {
        match frame.opcode {
            0x1 | 0x2 => {
                let message = decode_message(&frame.payload)?;
                handle_message(message, &host, &handler, &mut writer)?;
            }
            0x8 => break,
            0x9 => write_websocket(&mut writer, 0xA, &frame.payload)?,
            0xA => {}
            other => return Err(RemoteError::WebSocket(format!("unsupported opcode {other}"))),
        }
    }
    Ok(())
}

fn handle_message(
    message: Message,
    host: &Arc<Mutex<Host>>,
    handler: &CommandHandler,
    writer: &mut TcpStream,
) -> Result<(), RemoteError> {
    let (id, request) = match message {
        Message::Request { id, request } => (id, request),
        Message::Patch { patch } => {
            let mut guard = host.lock().unwrap_or_else(|error| error.into_inner());
            let applied = guard.apply(&patch);
            drop(guard);
            return match applied {
                Ok(()) => write_spec(writer, host),
                Err(error) => write_message(
                    writer,
                    &Message::Error {
                        message: error.to_string(),
                    },
                ),
            };
        }
        Message::Hello { .. } => {
            return write_message(
                writer,
                &Message::Response {
                    id: 0,
                    response: Response::Ok,
                },
            );
        }
        _ => return Ok(()),
    };
    let response = match request {
        Request::GetSpec => Response::Spec {
            spec: host.lock().unwrap_or_else(|error| error.into_inner()).json().clone(),
        },
        Request::ApplyPatch { patch } => {
            let mut host = host.lock().unwrap_or_else(|error| error.into_inner());
            match host.apply(&patch) {
                Ok(()) => Response::Spec {
                    spec: host.json().clone(),
                },
                Err(error) => Response::Error {
                    message: error.to_string(),
                },
            }
        }
        Request::RunCommand { id: command } => {
            let mut host = host.lock().unwrap_or_else(|error| error.into_inner());
            handler(&mut host, &command);
            let _ = host.apply_actions();
            let events = host.poll_events();
            let response = Response::Spec {
                spec: host.json().clone(),
            };
            drop(host);
            write_message(writer, &Message::Response { id, response })?;
            for event in events {
                write_message(writer, &Message::Event { event })?;
            }
            return Ok(());
        }
        Request::SetData { name, columns } => {
            let mut host = host.lock().unwrap_or_else(|error| error.into_inner());
            let dataset = egui_workbench_host::data::dataset_from_columns(columns);
            host.set_dataset(name, dataset);
            Response::Spec {
                spec: host.json().clone(),
            }
        }
        Request::Reload => {
            let mut host = host.lock().unwrap_or_else(|error| error.into_inner());
            if host.reload_if_changed() {
                Response::Spec {
                    spec: host.json().clone(),
                }
            } else {
                Response::Ok
            }
        }
        _ => Response::Error {
            message: "unsupported request".to_owned(),
        },
    };
    write_message(writer, &Message::Response { id, response })
}

fn write_spec(writer: &mut TcpStream, host: &Arc<Mutex<Host>>) -> Result<(), RemoteError> {
    let spec = host.lock().unwrap_or_else(|error| error.into_inner()).json().clone();
    write_message(
        writer,
        &Message::Response {
            id: 0,
            response: Response::Spec { spec },
        },
    )
}

fn decode_message(payload: &[u8]) -> Result<Message, RemoteError> {
    read_frame(Cursor::new(payload.to_vec())).map_err(|error| match error {
        FrameError::Protocol(error) => RemoteError::Protocol(error.to_string()),
        FrameError::Io(error) => RemoteError::Io(error),
        FrameError::Closed => RemoteError::Io("the payload ended early".to_owned()),
        _ => RemoteError::Protocol("unknown frame error".to_owned()),
    })
}

fn write_message(writer: &mut TcpStream, message: &Message) -> Result<(), RemoteError> {
    let mut payload = Vec::new();
    write_frame(&mut payload, message).map_err(|error| RemoteError::Protocol(error.to_string()))?;
    write_websocket(writer, 0x2, &payload)
}

fn handshake(
    reader: &mut impl BufRead,
    writer: &mut impl Write,
    allowed_origins: &[String],
) -> Result<(), RemoteError> {
    let mut key = None;
    let mut origin = None;
    let mut used = 0u64;
    let mut lines = 0usize;
    loop {
        if lines >= MAX_HEADER_LINES {
            return reject(
                writer,
                "431 Request Header Fields Too Large",
                "too many handshake headers",
            );
        }
        let mut line = String::new();
        let remaining = MAX_HANDSHAKE_BYTES.saturating_sub(used);
        let read = reader.by_ref().take(remaining).read_line(&mut line).map_err(io)?;
        if read == 0 {
            return Err(RemoteError::WebSocket("the handshake ended early".to_owned()));
        }
        used += read as u64;
        if !line.ends_with('\n') {
            return reject(
                writer,
                "431 Request Header Fields Too Large",
                "handshake headers are too large",
            );
        }
        lines += 1;
        let line = line.trim_end_matches(['\r', '\n']);
        if line.is_empty() {
            break;
        }
        if lines == 1 {
            // The request line is `GET <path> HTTP/1.1`; only headers carry a colon.
            continue;
        }
        let Some((name, value)) = line.split_once(':') else {
            return reject(writer, "400 Bad Request", "malformed handshake header");
        };
        let value = value.trim().to_owned();
        if name.eq_ignore_ascii_case("Sec-WebSocket-Key") {
            key = Some(value);
        } else if name.eq_ignore_ascii_case("Origin") {
            origin = Some(value);
        }
    }
    if let Some(origin) = &origin
        && !allowed_origins
            .iter()
            .any(|allowed| allowed.eq_ignore_ascii_case(origin))
    {
        return reject(writer, "403 Forbidden", "origin is not allowed");
    }
    let key = key.ok_or_else(|| RemoteError::WebSocket("missing Sec-WebSocket-Key".to_owned()))?;
    let mut hasher = Sha1::new();
    hasher.update(key.as_bytes());
    hasher.update(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
    let accept = STANDARD.encode(hasher.finalize());
    write!(
        writer,
        "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {accept}\r\n\r\n"
    )
    .and_then(|()| writer.flush())
    .map_err(io)
}

fn reject(writer: &mut impl Write, status: &str, reason: &str) -> Result<(), RemoteError> {
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{reason}",
        reason.len()
    );
    writer
        .write_all(response.as_bytes())
        .and_then(|()| writer.flush())
        .map_err(io)?;
    Err(RemoteError::WebSocket(reason.to_owned()))
}

struct Frame {
    opcode: u8,
    payload: Vec<u8>,
}

fn read_websocket(reader: &mut impl Read) -> Result<Option<Frame>, RemoteError> {
    let mut header = [0u8; 2];
    if reader.read_exact(&mut header).is_err() {
        return Ok(None);
    }
    let opcode = header[0] & 0x0F;
    let masked = header[1] & 0x80 != 0;
    // RFC 6455 section 5.1: a server closes the connection on an unmasked client frame.
    if !masked {
        return Err(RemoteError::WebSocket("client frame is not masked".to_owned()));
    }
    let length = match header[1] & 0x7F {
        126 => {
            let mut extended = [0u8; 2];
            reader.read_exact(&mut extended).map_err(io)?;
            u64::from(u16::from_be_bytes(extended))
        }
        127 => {
            let mut extended = [0u8; 8];
            reader.read_exact(&mut extended).map_err(io)?;
            u64::from_be_bytes(extended)
        }
        length => u64::from(length),
    };
    if length > MAX_FRAME_BYTES {
        return Err(RemoteError::WebSocket(format!("frame of {length} bytes is too large")));
    }
    // Control frames (close, ping, pong) carry at most 125 bytes.
    if opcode & 0x8 != 0 && length > 125 {
        return Err(RemoteError::WebSocket("control frame is too large".to_owned()));
    }
    let mut mask = [0u8; 4];
    reader.read_exact(&mut mask).map_err(io)?;
    // Grow with the bytes that actually arrive instead of trusting the declared length:
    // 32 connections each announcing 64 MiB must not reserve 2 GiB up front.
    let mut payload = Vec::new();
    let received = reader.by_ref().take(length).read_to_end(&mut payload).map_err(io)?;
    if received as u64 != length {
        return Err(RemoteError::WebSocket("the frame ended early".to_owned()));
    }
    for (index, byte) in payload.iter_mut().enumerate() {
        *byte ^= mask[index % 4];
    }
    Ok(Some(Frame { opcode, payload }))
}

fn write_websocket(writer: &mut impl Write, opcode: u8, payload: &[u8]) -> Result<(), RemoteError> {
    writer.write_all(&[0x80 | opcode]).map_err(io)?;
    match payload.len() {
        length @ 0..=125 => writer.write_all(&[length as u8]).map_err(io)?,
        length @ 126..=65_535 => {
            writer.write_all(&[126]).map_err(io)?;
            writer.write_all(&(length as u16).to_be_bytes()).map_err(io)?;
        }
        length => {
            writer.write_all(&[127]).map_err(io)?;
            writer.write_all(&(length as u64).to_be_bytes()).map_err(io)?;
        }
    }
    writer.write_all(payload).and_then(|()| writer.flush()).map_err(io)
}

fn io(error: std::io::Error) -> RemoteError {
    RemoteError::Io(error.to_string())
}
