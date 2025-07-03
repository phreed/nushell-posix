pub mod converter;
pub mod parser;
pub mod core;

// Re-export main types used by the plugin
pub use converter::PosixToNuConverter;
pub use parser::{parse_posix_script, PosixScript};
pub use core::PosixPlugin;
