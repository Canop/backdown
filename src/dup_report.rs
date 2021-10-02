use {
    anyhow::Result,
    crossbeam::channel,
    crate::*,
    fnv::FnvHashMap,
    minimad::*,
    rayon::{
        prelude::ParallelIterator,
        iter::ParallelBridge,
    },
    std::{
        cmp::Reverse,
        fs,
        path::PathBuf,
    },
    termimad::*,
};

#[derive(Default)]
pub struct DupReport {
    pub dups: Vec<DupSet>,
    pub seen: usize,
    /// number of files which could be removed
    /// when keeping one of each set
    pub duplicate_count: usize,
    pub duplicate_len_sum: u64,
}

impl DupReport {
    pub fn len(&self) -> usize {
        self.dups.len()
    }
    pub fn build(
        root: PathBuf,
        only_images: bool,
    ) -> Result<Self> {
        let (s_matching_files, r_matching_files) = channel::unbounded();
        let (s_hashed_files, r_hashed_files) = channel::unbounded::<(PathBuf, FileHash)>();
        let file_generator = std::thread::spawn(move||{
            let mut dirs = Vec::new();
            dirs.push(root);
            while let Some(dir) = dirs.pop() {
                if let Ok(entries) = fs::read_dir(&dir) {
                    for e in entries.flatten() {
                        let path = e.path();
                        let name = match path.file_name().and_then(|s| s.to_str()) {
                            Some(s) => s,
                            None => { continue; },
                        };
                        if name.starts_with('.') {
                            continue;
                        }
                        if let Ok(md) = path.symlink_metadata() {
                            if md.is_dir() {
                                // we add the directory to the channel of dirs needing processing
                                dirs.push(path);
                                continue;
                            }
                            if md.is_file() {
                                if only_images {
                                    let ext = match path.extension().and_then(|s| s.to_str()) {
                                        Some(s) => s,
                                        None => { continue; },
                                    };
                                    if !ext::is_image(ext) {
                                        continue;
                                    }
                                }
                                s_matching_files.send(path).unwrap();
                            }
                        }
                    }
                }
            }
        });

        // parallel computation of the hashes
        r_matching_files.into_iter().par_bridge()
            .for_each_with(s_hashed_files, |s, path| {
                if let Ok(hash) = FileHash::new(&path) {
                    s.send((path, hash)).unwrap();
                }
            });

        let mut map: FnvHashMap<FileHash, Vec<DupFile>> = FnvHashMap::default();
        let mut seen = 0;
        r_hashed_files.iter()
            .for_each(|(path, hash)| {
                let e = map.entry(hash).or_default();
                e.push(DupFile::new(path));
                seen += 1;
            });

        file_generator.join().unwrap();

        let mut dups = Vec::new();
        let mut duplicate_count = 0;
        let mut duplicate_len_sum = 0;
        for (_hash, files) in map.drain() {
            if files.len() < 2 {
                continue;
            }
            if let Ok(md) = fs::metadata(&files[0].path) {
                duplicate_count += files.len() - 1;
                let file_len = md.len();
                if file_len > 0 {
                    duplicate_len_sum += (files.len() - 1) as u64 * file_len;
                    dups.push(DupSet {
                        files,
                        file_len,
                    });
                }
            }
        }

        dups.sort_by_key(|dup| Reverse(dup.files.len()));

        Ok(Self{
            dups,
            seen,
            duplicate_count,
            duplicate_len_sum,
        })
    }

    pub fn print_summary(
        &self,
        skin: &MadSkin,
    ) {
        static MD: &str = r#"
        I've hashed *${seen}* files and found *${set_count}* sets of duplicates.\
        *${removable_count}* files can be removed to gain **${gain}**.\
        "#;
        let mut expander = OwningTemplateExpander::new();
        expander
                .set("seen", self.seen)
                .set("set_count", self.dups.len())
                .set("removable_count", self.duplicate_count)
                .set("gain", file_size::fit_4(self.duplicate_len_sum));
        skin.print_owning_expander(&expander, &TextTemplate::from(MD));
    }
    pub fn is_empty(&self) -> bool {
        self.dups.is_empty()
    }
}
