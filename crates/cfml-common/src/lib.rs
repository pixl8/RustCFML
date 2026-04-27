//! Common utilities for RustCFML

/// RustCFML workspace version (cfml-common inherits `version.workspace = true`).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod dynamic;
pub mod encodings;
pub mod introspection;
pub mod position;
pub mod vfs;
pub mod vm;
