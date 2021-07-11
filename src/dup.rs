use {
    std::path::{Path, PathBuf},
};


// TODO virer et utiliser PathBuf directement ?
#[derive(Debug)]
pub struct DupFile {
    pub path: PathBuf,
    // pub staged_for_removal: bool,
}

/// the list of files having a hash
/// TODO rename DupSet ?
#[derive(Debug, Default)]
pub struct DupSet {
    pub files: Vec<DupFile>, // identical files
    pub file_len: u64,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq,)]
pub struct DupFileRef {
    pub dup_set_idx: usize,
    pub dup_file_idx: usize,
}

impl DupFile {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            //staged_for_removal: false,
        }
    }
}

impl DupFileRef {
    pub fn path(self, dups: &[DupSet]) -> &Path {
        &dups[self.dup_set_idx].files[self.dup_file_idx].path
    }
    pub fn file_name(self, dups:&[DupSet]) -> String {
        self.path(dups)
            .file_name()
            .map_or_else(
                || "".to_string(),
                |n| n.to_string_lossy().to_string()
            )
    }
}
