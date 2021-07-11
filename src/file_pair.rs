use {
    crate::*,
};

#[derive(Debug, Clone, Copy)]
pub struct FilePair {
    pub dup_set_idx: usize,
    pub left_file_idx: usize,
    pub right_file_idx: usize,
}

impl FilePair {
    pub fn left_ref(self) -> DupFileRef {
        DupFileRef {
            dup_set_idx: self.dup_set_idx,
            dup_file_idx: self.left_file_idx,
        }
    }
    pub fn right_ref(self) -> DupFileRef {
        DupFileRef {
            dup_set_idx: self.dup_set_idx,
            dup_file_idx: self.right_file_idx,
        }
    }
}
