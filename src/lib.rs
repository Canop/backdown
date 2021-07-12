pub mod args;
pub mod ask;
pub mod dirs;
pub mod dup;
pub mod dup_report;
pub mod file_pair;
pub mod ext;
pub mod hash;
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
    removal_report::*,
};
