
use {
    anyhow::Result,
    std::{
        fs::File,
        io,
        path::Path,
    },
};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct FileHash {
    hash: blake3::Hash,
}

impl FileHash {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = File::open(path)?;
        let mut hasher = blake3::Hasher::new();
        io::copy(&mut file, &mut hasher)?;
        let hash = hasher.finalize();
        Ok(Self {
            hash,
        })
    }
}
