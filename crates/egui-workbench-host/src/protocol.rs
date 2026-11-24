//! The transport-agnostic host protocol.
//!
//! Messages are JSON documents. On a byte stream they are length-prefixed with a
//! four-byte big-endian length, so the same frames travel over stdio, a socket, or a
//! WebSocket binary message. The protocol is versioned and rejects a newer version.

use std::io::{Read, Write};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::data::BufferColumn;
use crate::patch::Patch;
use crate::runtime::HostEvent;

/// The protocol version understood by this host.
pub const PROTOCOL_VERSION: u32 = 1;

/// One protocol message.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[non_exhaustive]
pub enum Message {
    /// First message on a connection.
    Hello {
        /// Protocol version.
        version: u32,
        /// Capabilities, such as `data` or `patches`.
        capabilities: Vec<String>,
    },
    /// A request with a correlation id.
    Request {
        /// Correlation id.
        id: u64,
        /// The request.
        request: Request,
    },
    /// A response to a request.
    Response {
        /// Correlation id.
        id: u64,
        /// The response.
        response: Response,
    },
    /// An event emitted by the host.
    Event {
        /// The event.
        event: HostEvent,
    },
    /// A patch without a request id.
    Patch {
        /// The patch.
        patch: Patch,
    },
    /// A data buffer (large columns; a binary transport may carry it out of band).
    Data {
        /// Dataset name.
        name: String,
        /// Columns.
        columns: Vec<BufferColumn>,
    },
    /// An error frame.
    Error {
        /// Human-readable message.
        message: String,
    },
}

/// A host request.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
#[non_exhaustive]
pub enum Request {
    /// Apply a patch.
    ApplyPatch {
        /// The patch.
        patch: Patch,
    },
    /// Read the current spec.
    GetSpec,
    /// Run a command by id.
    RunCommand {
        /// Command id.
        id: String,
    },
    /// Replace a named dataset.
    SetData {
        /// Dataset name.
        name: String,
        /// Columns.
        columns: Vec<BufferColumn>,
    },
    /// Reload the spec from its path.
    Reload,
}

/// A host response.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "result", rename_all = "snake_case")]
#[non_exhaustive]
pub enum Response {
    /// The current spec.
    Spec {
        /// Spec JSON.
        spec: Value,
    },
    /// Events produced by the operation.
    Events {
        /// Events.
        events: Vec<HostEvent>,
    },
    /// Success with no payload.
    Ok,
    /// An error.
    Error {
        /// Human-readable message.
        message: String,
    },
    /// The generated JSON Schema.
    Schema {
        /// Schema JSON.
        schema: Value,
    },
}

/// Why a protocol message could not be decoded or accepted.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ProtocolError {
    /// Malformed JSON.
    Json(String),
    /// A newer protocol version.
    UnsupportedVersion(u32),
    /// A frame is too large.
    FrameTooLarge(usize),
}

impl std::fmt::Display for ProtocolError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json(error) => write!(formatter, "invalid protocol JSON: {error}"),
            Self::UnsupportedVersion(version) => write!(formatter, "unsupported protocol version {version}"),
            Self::FrameTooLarge(size) => write!(formatter, "frame of {size} bytes is too large"),
        }
    }
}

impl std::error::Error for ProtocolError {}

impl Message {
    /// Encode as compact JSON.
    ///
    /// # Errors
    /// When serialization fails (it does not for these types).
    pub fn encode(&self) -> Result<String, ProtocolError> {
        serde_json::to_string(self).map_err(|error| ProtocolError::Json(error.to_string()))
    }

    /// Decode JSON and validate the protocol version.
    ///
    /// # Errors
    /// When the JSON is malformed or the message claims a newer version.
    pub fn decode(json: &str) -> Result<Self, ProtocolError> {
        let message: Self = serde_json::from_str(json).map_err(|error| ProtocolError::Json(error.to_string()))?;
        if let Self::Hello { version, .. } = message
            && version > PROTOCOL_VERSION
        {
            return Err(ProtocolError::UnsupportedVersion(version));
        }
        Ok(message)
    }
}

/// A framed read or write error.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum FrameError {
    /// The underlying stream failed.
    Io(String),
    /// The payload was not valid JSON or protocol.
    Protocol(ProtocolError),
}

impl std::fmt::Display for FrameError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => formatter.write_str(error),
            Self::Protocol(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for FrameError {}

/// Write one length-prefixed frame.
///
/// # Errors
/// When encoding or writing fails.
pub fn write_frame(mut writer: impl Write, message: &Message) -> Result<(), FrameError> {
    let json = message.encode().map_err(FrameError::Protocol)?;
    let length =
        u32::try_from(json.len()).map_err(|_| FrameError::Protocol(ProtocolError::FrameTooLarge(json.len())))?;
    writer
        .write_all(&length.to_be_bytes())
        .and_then(|()| writer.write_all(json.as_bytes()))
        .and_then(|()| writer.flush())
        .map_err(|error| FrameError::Io(error.to_string()))
}

/// Read one length-prefixed frame.
///
/// # Errors
/// When reading, decoding, or validating fails.
pub fn read_frame(mut reader: impl Read) -> Result<Message, FrameError> {
    let mut length = [0u8; 4];
    reader
        .read_exact(&mut length)
        .map_err(|error| FrameError::Io(error.to_string()))?;
    let length = u32::from_be_bytes(length) as usize;
    if length > 64 * 1024 * 1024 {
        return Err(FrameError::Protocol(ProtocolError::FrameTooLarge(length)));
    }
    let mut bytes = vec![0u8; length];
    reader
        .read_exact(&mut bytes)
        .map_err(|error| FrameError::Io(error.to_string()))?;
    let json =
        String::from_utf8(bytes).map_err(|error| FrameError::Protocol(ProtocolError::Json(error.to_string())))?;
    Message::decode(&json).map_err(FrameError::Protocol)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_round_trip_and_versions_are_checked() {
        let message = Message::Hello {
            version: PROTOCOL_VERSION,
            capabilities: vec!["patches".to_owned()],
        };
        let json = message.encode().expect("encode");
        let back = Message::decode(&json).expect("decode");
        assert_eq!(back.encode().expect("encode"), json);
        let newer = r#"{"type":"hello","version":99,"capabilities":[]}"#;
        assert!(matches!(
            Message::decode(newer),
            Err(ProtocolError::UnsupportedVersion(99))
        ));
    }

    #[test]
    fn frames_round_trip() {
        let message = Message::Response {
            id: 7,
            response: Response::Ok,
        };
        let mut bytes = Vec::new();
        write_frame(&mut bytes, &message).expect("write");
        let back = read_frame(std::io::Cursor::new(bytes)).expect("read");
        assert_eq!(back.encode().expect("encode"), message.encode().expect("encode"));
    }
}
