use {
    anyhow::Result,
    crossbeam::channel,
    crate::{
        dup::Dup,
        ext,
        hash::FileHash,
    },
    fnv::FnvHashMap,
    rayon::{
        prelude::ParallelIterator,
        iter::ParallelBridge,
    },
    std::{
        fs,
        path::{Path, PathBuf},
    },
};

#[derive(Default)]
pub struct DupMap {
    pub dups: FnvHashMap<FileHash, Dup>,
    pub seen: usize,
}

impl DupMap {
    pub fn add_file(&mut self, path: &Path) -> Result<()> {
        let hash = FileHash::new(path)?;
        let e = self.dups.entry(hash).or_default();
        e.paths.push(path.to_path_buf());
        self.seen += 1;
        Ok(())
    }
    pub fn compile(&mut self) {
        self.dups.retain(|_, d| d.paths.len()>1);
    }
    pub fn len(&self) -> usize {
        self.dups.len()
    }
    pub fn build(root: PathBuf) -> Result<Self> {
        let (s_matching_files, r_matching_files) = channel::unbounded();
        let (s_hashed_files, r_hashed_files) = channel::unbounded::<(PathBuf, FileHash)>();

        let file_generator = std::thread::spawn(move||{
            let mut dirs = Vec::new();
            dirs.push(root);
            while let Some(dir) = dirs.pop() {
                if let Ok(entries) = fs::read_dir(&dir) {
                    for e in entries.flatten() {
                        if let Ok(md) = e.metadata() {
                            let path = e.path();
                            let name = match path.file_name().and_then(|s| s.to_str()) {
                                Some(s) => s,
                                None => { continue; },
                            };
                            if md.is_dir() {
                                // Until I implement gitignore, I'll just avoid
                                // my ~/dev directory
                                if name == "dev" {
                                    continue;
                                }
                                // we add the directory to the channel of dirs needing
                                // processing
                                dirs.push(path);
                                continue;
                            }
                            let ext = match path.extension().and_then(|s| s.to_str()) {
                                Some(s) => s,
                                None => { continue; },
                            };
                            if !ext::is_image(&ext) {
                                continue;
                            }
                            s_matching_files.send(path).unwrap();
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

        let mut dups: FnvHashMap<FileHash, Dup> = FnvHashMap::default();
        let mut seen = 0;
        r_hashed_files.iter()
            .for_each(|(path, hash)| {
                let e = dups.entry(hash).or_default();
                e.paths.push(path.to_path_buf());
                seen += 1;
            });


        file_generator.join().unwrap();

        dups.retain(|_, d| d.paths.len()>1);

        Ok(Self{
            dups,
            seen,
        })
    }
}
