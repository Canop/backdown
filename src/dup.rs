use {
    lazy_regex::*,
    std::{
        collections::HashSet,
        path::{Path, PathBuf},
    },
};


// TODO virer et utiliser PathBuf directement ?
#[derive(Debug)]
pub struct DupFile {
    pub path: PathBuf,
    // pub staged_for_removal: bool,
}

/// the list of files having a hash
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

pub fn reference_file<'a, 'b>(
    dup_set_idx: usize,
    dup_set: &'a DupSet,
    staged_removals: &'b HashSet<DupFileRef>,
) -> Option<&'a Path> {
    let mut best: Option<&Path> = None;
    for (dup_file_idx, file) in dup_set.files.iter().enumerate() {
        let path = &file.path;
        let dup_file_ref = DupFileRef { dup_set_idx, dup_file_idx };
        if staged_removals.contains(&dup_file_ref) {
            continue;
        }
        if let Some(previous) = best {
            if previous.to_string_lossy().len() > path.to_string_lossy().len() {
                best = Some(path);
            }
        } else {
            best = Some(path);
        }
    }
    best
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
    /// get the file name when the file has a name like "thing (3).jpg"
    /// or "thing (3rd copy).png"
    pub fn copy_name(self, dups:&[DupSet]) -> Option<&str> {
        copy_name(self.path(dups))
    }
    /// tells whether the file has a name like "thing (3).jpg"
    /// or "thing (3rd copy).png"
    pub fn is_copy_named(self, dups:&[DupSet]) -> bool {
        self.copy_name(dups).is_some()
    }
}

/// get the name if this path is of a "copy" file, that is an usual name for a copy
pub fn copy_name(path: &Path) -> Option<&str> {
    path
        .file_name()
        .and_then(std::ffi::OsStr::to_str)
        .filter(|n| regex_is_match!(r#"(?x)
            .+
            \((
                \d+
            |
                [^)]*
                copy
            )\)
            (\.\w+)?
            $
        "#, n))
}

#[test]
fn test_is_copy_named() {
    use std::path::PathBuf;
    let copies = &[
        "/some/path/to/bla (3).jpg",
        "bla (3455).jpg",
        "uuuuu (copy).rs",
        "/home/dys/Images/pink hexapodes (another copy).jpeg",
        "~/uuuuu (copy)",
        "uuuuu (3rd copy)",
    ];
    for s in copies {
        assert!(copy_name(&PathBuf::from(s)).is_some());
    }
    let not_copies = &[
        "copy",
        "copy.txt",
        "bla.png",
        "/home/dys/not a copy",
        "(don't copy)",
    ];
    for s in not_copies {
        assert!(copy_name(&PathBuf::from(s)).is_none());
    }

}
