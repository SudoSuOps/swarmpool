//! CLI command implementations
//!
//! Each command follows fail-closed design:
//! - If validation fails → nothing is published
//! - If signing fails → nothing is published
//! - If IPFS fails → nothing is published

pub mod claim;
pub mod epochs;
pub mod init;
pub mod prove;
pub mod seal;
pub mod status;
pub mod submit;
pub mod validate;
pub mod watch;
pub mod withdraw;
