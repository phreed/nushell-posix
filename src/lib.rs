pub mod plugin;

// Re-export commonly used types and functions
pub use plugin::{
    parse_posix_script, AndOrData, AndOrOperator, Assignment, CompoundCommandData,
    CompoundCommandKind, ListData, PipelineData, PosixCommand, PosixPlugin, PosixScript,
    PosixToNuConverter, Redirection, SimpleCommandData,
};
