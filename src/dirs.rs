use {
    crate::*,
    fnv::FnvHashMap,
    std::{
        cmp::{Ord, Ordering, Reverse},
        path::Path,
    },
};

#[derive(Debug)]
pub struct DirsReport<'d> {
    pub dup_dirs: Vec<DupDir<'d>>,
    pub brotherhoods: Vec<Brotherhood<'d>>,
    pub auto_solvable_brotherhoods_count: usize,
    pub dir_pairs: Vec<DirPair<'d>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DirPairKey<'d> {
    pub left_dir: &'d Path,
    pub right_dir: &'d Path,
}

#[derive(Debug)]
pub struct DirPair<'d> {
    pub key: DirPairKey<'d>,
    pub file_pairs: Vec<FilePair>,
}

/// a brotherhood gather duplicates having the same parent
#[derive(Debug)]
pub struct Brotherhood<'d> {

    pub parent: &'d Path,

    pub dup_set_idx: usize,

    /// file indexes
    pub files: Vec<usize>,

    /// when all files have names like "thing (copy).png", "thing (another copy).png", etc.
    /// except one file, we can propose an automated resolution.
    /// Note that we don't check the start of filenames are identical because we
    /// don't, in fact, care.
    pub is_auto_solvable: bool,
}

/// a directory which contains only duplicates
#[derive(Debug)]
pub struct DupDir<'d> {
    pub path: &'d Path,
    pub files: Vec<DupFileRef>,
}

impl<'d> Brotherhood<'d> {
    fn maybe_add_files(&mut self, a_idx: usize, b_idx: usize) {
        if !self.files.contains(&a_idx) {
            self.files.push(a_idx);
        }
        if !self.files.contains(&b_idx) {
            self.files.push(b_idx);
        }
    }
    pub fn file_ref(&self, i: usize) -> DupFileRef {
        DupFileRef {
            dup_set_idx: self.dup_set_idx,
            dup_file_idx: self.files[i],
        }
    }
    pub fn gain(&self, dups: &[DupSet]) -> u64 {
        (self.files.len() - 1) as u64 * dups[self.dup_set_idx].file_len
    }
}

impl<'d> DirPairKey<'d> {
    pub fn new(a: &'d Path, b: &'d Path) -> (Self, bool) {
        if a.cmp(b) == Ordering::Less {
            (
                DirPairKey {
                    left_dir: a,
                    right_dir: b,
                },
                false
            )
        } else {
            (
                DirPairKey {
                    left_dir: b,
                    right_dir: a,
                },
                true
            )
        }
    }
}

impl<'d> DirPair<'d> {
    pub fn new(
        key: DirPairKey<'d>,
        file_pairs: Vec<FilePair>,
    ) -> Self {
        Self { key, file_pairs }
    }
}

impl<'d> DirsReport<'d> {
    pub fn compute(dups: &'d[DupSet]) -> anyhow::Result<Self> {
        let mut brotherhoods = Vec::new();
        let mut auto_solvable_brotherhoods_count = 0;
        let mut dp_map: FnvHashMap<DirPairKey, Vec<FilePair>> = FnvHashMap::default();
        let mut dir_map: FnvHashMap<&Path, Vec<DupFileRef>> = FnvHashMap::default();
        let mut brotherhood_per_parent: FnvHashMap<&Path, Brotherhood<'d>> = FnvHashMap::default();
        for (dup_set_idx, dup) in dups.iter().enumerate() {
            brotherhood_per_parent.clear();
            for (a_file_idx, a) in dup.files.iter().enumerate() {
                let a_parent = a.path.parent().unwrap();
                // adding to the dir_map
                dir_map.entry(a_parent)
                    .or_default()
                    .push(DupFileRef { dup_set_idx, dup_file_idx: a_file_idx });

                // building dir pair
                for b_file_idx in a_file_idx+1..dup.files.len() {
                    let b = &dup.files[b_file_idx];
                    let b_parent = b.path.parent().unwrap();
                    let (dpk, inverted) = DirPairKey::new(
                        a_parent,
                        b_parent,
                    );
                    let (left_file_idx, right_file_idx) = if inverted {
                        (b_file_idx, a_file_idx)
                    } else {
                        (a_file_idx, b_file_idx)
                    };
                    if a_parent == b_parent {
                        // brotherhood
                        brotherhood_per_parent
                            .entry(a_parent)
                            .or_insert_with(|| Brotherhood {
                                parent: a_parent,
                                dup_set_idx,
                                files: Vec::new(),
                                is_auto_solvable: false,
                            })
                            .maybe_add_files(left_file_idx, right_file_idx);
                    } else {
                        // dir_pair
                        dp_map.entry(dpk)
                            .or_default()
                            .push(FilePair {
                                dup_set_idx,
                                left_file_idx,
                                right_file_idx,
                            });
                    }
                }
            }
            for (_, mut brotherhood) in brotherhood_per_parent.drain() {
                let copy_count = brotherhood.files
                    .iter()
                    .map(|&dup_file_idx| DupFileRef {
                        dup_set_idx: brotherhood.dup_set_idx,
                        dup_file_idx,
                    })
                    .filter(|dup_file_ref| dup_file_ref.is_copy_named(dups))
                    .count();
                if copy_count + 1 == brotherhood.files.len() {
                    brotherhood.is_auto_solvable = true;
                    auto_solvable_brotherhoods_count += 1;
                }
                brotherhoods.push(brotherhood);
            }
        }

        // we remove the parent of brotherhoods from dir_map
        // because we don't want them in dup_dirs
        for brotherhood in &brotherhoods {
            dir_map.remove(brotherhood.parent);
        }

        let mut dup_dirs = Vec::new();
        for (path, files) in dir_map.drain() {
            if files.len() < 3 {
                // small directories aren't interesting, we'll handle
                // the dups by comparing dup dirs
                continue;
            }
            let total_child_count = path.read_dir()?.count();
            if total_child_count == files.len() {
                dup_dirs.push(DupDir { path, files });
            }
        }

        // ordering
        dup_dirs.sort_by_key(|dd| Reverse(dd.files.len()));
        brotherhoods.sort_by_key(|b| Reverse(b.gain(dups)));
        let mut dir_pairs: Vec<_> = dp_map
            .drain()
            .map(|(key, file_pairs)| DirPair::new(key, file_pairs))
            .collect();
        dir_pairs.sort_by_key(|dp| Reverse(dp.file_pairs.len()));

        Ok(Self {
            dup_dirs,
            brotherhoods,
            auto_solvable_brotherhoods_count,
            dir_pairs,
        })
    }
}

