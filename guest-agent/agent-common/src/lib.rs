//! KVM Manager Guest Agent - Common Protocol Types
//!
//! This crate defines the JSON-RPC 2.0 protocol types shared between
//! the guest agent and the host backend.

pub mod protocol;
pub mod error;

pub use protocol::*;
pub use error::*;
