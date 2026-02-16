use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("XDG_RUNTIME_DIR is not set")]
    XdgRuntimeDirNotSet,
    #[error("Protocol error: {0}")]
    ProtocolDecodeError(#[from] prost::DecodeError),
    #[error("Protocol error: {0}")]
    ProtocolEncodeError(#[from] prost::EncodeError),
    #[error("Received response with no payload")]
    NoPayload,
    #[error("{message}")]
    ErrorResponse { message: String },
    #[error("Unexpected response type: expected {expected}, got {actual}")]
    UnexpectedResponseType {
        expected: &'static str,
        actual: String,
    },
    #[error("Invalid connection type")]
    InvalidConnectionType,
    #[error("Invalid update type")]
    InvalidUpdateType,
}
