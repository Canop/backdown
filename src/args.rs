use {
    argh::FromArgs,
    std::path::PathBuf,
};

#[derive(FromArgs)]
/// Help you remove duplicate files from your disks
///
///
/// Source and doc at https://github.com/Canop/backdown
pub struct Args {
    /// print the version
    #[argh(switch, short = 'v')]
    pub version: bool,

    /// whether to only handle image files
    #[argh(switch, short = 'i')]
    pub only_images: bool,

    #[argh(positional)]
    /// where to look for duplicates (will use . if no directory is provided)
    pub path: Option<PathBuf>,
}

