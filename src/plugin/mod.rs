pub mod converter;
pub mod core;
pub mod parser_heuristic;
pub mod parser_posix;

// Re-export main types used by the plugin
pub use converter::PosixToNuConverter;
pub use core::PosixPlugin;
pub use parser_posix::{parse_posix_script, PosixScript};

// Re-export parser types for integration tests
pub use parser_posix::*;
