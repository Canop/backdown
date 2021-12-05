#[macro_use] extern crate cli_log;

pub mod args;
pub mod ask;
pub mod dirs;
pub mod dup;
pub mod dup_report;
pub mod ext;
pub mod file_pair;
pub mod hash;
mod json;
pub mod removal_report;

pub use {
    args::*,
    ask::*,
    dirs::*,
    dup::*,
    dup_report::*,
    file_pair::*,
    ext::*,
    hash::*,
    json::*,
    removal_report::*,
};
