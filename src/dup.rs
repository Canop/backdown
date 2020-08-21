
use {
    std::path::PathBuf,
};

/// the list of files having a hash
#[derive(Debug, Default)]
pub struct Dup {
    pub paths: Vec<PathBuf>,
}
