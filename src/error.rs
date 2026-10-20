//! Crate-level error types. Keep IO/protocol failures typed so upper layers
//! can decide whether to reconnect, surface UI feedback, or exit.

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("config: {0}")]
    Config(String),

    #[error("irc protocol: {0}")]
    Protocol(String),

    #[error("connection: {0}")]
    Connection(#[from] std::io::Error),

    #[error("channel closed")]
    ChannelClosed,

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
