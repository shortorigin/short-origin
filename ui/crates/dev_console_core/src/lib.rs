//! Headless data and action orchestration for the developer DX console.

#![warn(missing_docs, rustdoc::broken_intra_doc_links)]

pub mod model;
pub mod providers;
pub mod service;
pub mod shell;

pub use model::*;
pub use providers::*;
pub use service::*;
pub use shell::ShellContext;
