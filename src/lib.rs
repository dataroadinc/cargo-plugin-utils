//! Shared utilities for cargo plugins.

pub mod common;
pub mod logger;
pub mod progress_logger;
pub mod scrolling;
pub mod tty;

pub use common::{
    detect_repo,
    find_package,
    get_metadata,
    get_owner_repo,
    get_package_version_from_manifest,
    get_workspace_packages,
};
pub use logger::{
    Logger,
    SubprocessOutput,
};
pub use progress_logger::ProgressLogger;
pub use tty::should_show_progress;
