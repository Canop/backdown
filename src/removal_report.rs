use {
    crate::*,
    minimad::*,
    std::{
        collections::HashSet,
        fs,
        path::Path,
    },
    termimad::*,
};

#[derive(Debug, Clone, Default)]
pub struct RemovalReport<'d> {
    pub dup_sets_with_staged: HashSet<usize>,
    pub staged_removals: HashSet<DupFileRef>,
    pub staged_dir_removals: Vec<&'d Path>,
    pub len_to_remove: u64,
    pub broken: bool,
    pub quit: bool,
}


impl<'d> RemovalReport<'d> {
    pub fn stage_file(&mut self, dup_file_ref: DupFileRef, dups: &[DupSet]) {
        self.len_to_remove += dups[dup_file_ref.dup_set_idx].file_len;
        self.dup_sets_with_staged.insert(dup_file_ref.dup_set_idx);
        self.staged_removals.insert(dup_file_ref);
        //println!("staged {:?}", &dups[dup_file_ref.dup_set_idx].files[dup_file_ref.dup_file_idx].path);
    }

    pub fn is_empty(&self) -> bool {
        self.staged_removals.is_empty()
    }

    pub fn list_staged_removals(
        &self,
        dups: &[DupSet],
        skin: &MadSkin,
    ) {
        mad_print_inline!(
            &skin,
            "**$0** files planned for removal for a total size of **$1**:\n",
            self.staged_removals.len(),
            file_size::fit_4(self.len_to_remove),
        );
        for (idx, file_ref) in self.staged_removals.iter().enumerate() {
            let path = file_ref.path(dups);
            let size = dups[file_ref.dup_set_idx].file_len;
            mad_print_inline!(
                &skin,
                "#$0 : *$1* (**$2**)\n",
                idx + 1,
                path.to_string_lossy(),
                file_size::fit_4(size),
            );
        }
    }

    pub fn list_dup_sets(
        &self,
        dups: &[DupSet],
        skin: &MadSkin,
    ) {
        static MD: &str = r#"
        |:-|:-|
        |Set #*${set_num}* : each file is **${file_len}**|action|
        |:-|:-:|
        ${files
        |${path}|**${action}**|
        }
        |-
        "#;
        let template = TextTemplate::from(MD);
        for (dup_set_idx, dup_set) in dups.iter().enumerate() {
            if !self.dup_sets_with_staged.contains(&dup_set_idx) {
                continue;
            }
            let mut expander = OwningTemplateExpander::new();
            expander
                .set("set_num", dup_set_idx + 1)
                .set("file_len", file_size::fit_4(dup_set.file_len));
            for (dup_file_idx, file) in dup_set.files.iter().enumerate() {
                let file_ref = DupFileRef { dup_set_idx, dup_file_idx };
                expander.sub("files")
                    .set("path", file.path.to_string_lossy())
                    .set_md(
                        "action",
                        if self.staged_removals.contains(&file_ref) {
                            "*remove*"
                        } else {
                            "keep"
                        }
                    );
            }
            skin.print_owning_expander(&expander, &template);
        }
    }

    /// "Normally" the algorithms of backdown never remove all files
    /// in a set of identical files. But if I change those algorithms
    /// and make them more complex, I may make an error. So this
    /// function will check there's at least one kept file in each
    /// touched set, and will raise an error if a set is totally
    /// emptied.
    /// This *must* be called just before starting the real removals.
    pub fn check_no_emptied_set(
        &self,
        dups: &[DupSet],
    ) -> anyhow::Result<()> {
        for (dup_set_idx, dup_set) in dups.iter().enumerate() {
            let mut staged_count = 0;
            for dup_file_idx in 0..dup_set.files.len() {
                if self.staged_removals.contains(&DupFileRef{ dup_set_idx, dup_file_idx }) {
                    staged_count += 1;
                }
            }
            if staged_count >= dup_set.files.len() {
                anyhow::bail!("We staged all files in set for removal! Abort!");
            }
        }
        Ok(())
    }


    pub fn do_the_removal(
        &self,
        dups: &[DupSet],
        skin: &MadSkin,
    ) -> anyhow::Result<()> {
        self.check_no_emptied_set(dups)?;
        skin.print_text("\n# Phase 4) Removal");
        println!("Removing...");
        let mut removed_len = 0;
        let mut removed_count = 0;
        // file removals
        for dup_file_ref in &self.staged_removals {
            let path = dup_file_ref.path(dups);
            match fs::remove_file(path) {
                Ok(()) => {
                    removed_count += 1;
                    removed_len += dups[dup_file_ref.dup_set_idx].file_len;
                }
                Err(e) => {
                    mad_print_inline!(
                        skin,
                        " Failed to remove *$1* : $2\n",
                        path.to_string_lossy(),
                        e,
                    );
                }
            }
        }
        // directory removals
        for path in &self.staged_dir_removals {
            debug!("removing {:?}", path);
            if let Err(e) = fs::remove_dir(path) {
                mad_print_inline!(
                    skin,
                    " Failed to remove directory *$1* : $2\n",
                    path.to_string_lossy(),
                    e,
                );
            }
        }
        mad_print_inline!(
            skin,
            "Removed *$0* files with a total size of **$1**\n",
            removed_count,
            file_size::fit_4(removed_len),
        );
        Ok(())
    }
}
