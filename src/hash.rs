
use {
    anyhow::Result,
    sha2::{Sha256, Digest},
    std::{
        convert::TryInto,
        fs::File,
        io,
        path::Path,
    },
};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct FileHash {
    bytes: [u8; 32],
}

impl FileHash {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = File::open(path)?;
        let mut sha256 = Sha256::new();
        io::copy(&mut file, &mut sha256)?;
        let bytes = sha256.finalize()
            .as_slice()
            .try_into().expect("unexpected failure");
        Ok(Self {
            bytes,
        })
    }
}
