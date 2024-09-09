use {
    crate::*,
    fnv::FnvHashMap,
    minimad::*,
    termimad::*,
};

const MAX_LISTED_FILES: usize = 5;

pub fn ask_on_dirs<'d>(
    dirs_report: &'d DirsReport,
    dups: &'d [DupSet],
    skin: &MadSkin,
) -> anyhow::Result<RemovalReport<'d>> {
    let mut rr = RemovalReport::default();
    let mut question_idx = 0;
    let mut questions = dirs_report.dup_dirs.len() + dirs_report.brotherhoods.len() + dirs_report.dir_pairs.len();
    let ask_about_autosolve = dirs_report.auto_solvable_brotherhoods_count > 1;
    if ask_about_autosolve {
        questions += 1;
    }

    static MD: &str = r#"
    I'll now ask you up to *${questions}* questions to determine what files should be removed.\
    No file will be removed until you have the possibility to review them after the staging step.\
    You don't have to answer all questions:\
    you may end the staging phase at any time and then either do the removals or quit.
    "#;
    let mut expander = OwningTemplateExpander::new();
    expander.set("questions", questions);
    skin.print_owning_expander(&expander, &TextTemplate::from(MD));

    // return true if break
    let check = |rr: &RemovalReport| {
        if rr.quit {
            return true;
        }
        mad_print_inline!(
            skin,
            " -> currently staged: **$0** duplicate files for a removed size of **$1**\n",
            // two following lines used for some screenshots so that I don't redo the staging
            // 1042,
            // "5.5G",
            rr.staged_removals.len(),
            file_size::fit_4(rr.len_to_remove),
        );
        rr.broken
    };

    let skip_auto_solvable_brotherhoods = ask_about_autosolve && {
        let solved = ask_auto_solve(
            question_idx,
            questions,
            dirs_report,
            dups,
            skin,
            &mut rr,
        )?;
        if check(&rr) {
            return Ok(rr);
        }
        question_idx += 1;
        solved
    };

    for dup_dir in &dirs_report.dup_dirs {
        ask_on_dup_dir(
            question_idx,
            questions,
            dup_dir,
            dups,
            skin,
            &mut rr,
        )?;
        if check(&rr) {
            break;
        }
        question_idx += 1;
    }
    if rr.broken || rr.quit {
        return Ok(rr);
    }

    for brotherhood in &dirs_report.brotherhoods {
        if skip_auto_solvable_brotherhoods && brotherhood.is_auto_solvable {
            mad_print_inline!(skin, "skipping question *$0*\n", question_idx);
        } else {
            ask_on_brotherhood(
                question_idx,
                questions,
                brotherhood,
                dups,
                skin,
                &mut rr,
            )?;
            if check(&rr) {
                break;
            }
        }
        question_idx += 1;
    }
    if rr.broken || rr.quit {
        return Ok(rr);
    }

    for dir_pair in &dirs_report.dir_pairs {
        ask_on_dir_pair(
            question_idx,
            questions,
            dir_pair,
            dups,
            skin,
            &mut rr,
        )?;
        if check(&rr) {
            break;
        }
        question_idx += 1;
    }

    Ok(rr)
}

static MD_AUTO_SOLVE: &str = r#"

## Staging Question **${num}**/${questions}
You have several duplicates with "copy" names in the same directory than their identical "source" (for example *${example_1}* and *${example_2}*).
I can automatically stage those **${file_count}** duplicates, which would let you gain **${size}**.
If you accept, you'll skip *${skippable_questions}* questions.
"#;

/// return whether auto solvable brotherhoods are solved (we'll skip their questions then)
fn ask_auto_solve<'d>(
    question_idx: usize,
    questions: usize,
    dirs_report: &'d DirsReport,
    dups: &'d [DupSet],
    skin: &MadSkin,
    rr: &mut RemovalReport<'d>,
) -> anyhow::Result<bool> {
    debug_assert!(question_idx == 0);
    let mut removable_count = 0;
    let mut removable_len = 0;
    let mut skippable_questions = 0;
    let mut example_names = Vec::new();
    for brotherhood in dirs_report.brotherhoods.iter().filter(|b| b.is_auto_solvable) {
        removable_count += brotherhood.files.len() - 1;
        removable_len += (brotherhood.files.len() - 1) as u64 * dups[brotherhood.dup_set_idx].file_len;
        skippable_questions += 1;
        if example_names.len() < 2 {
            example_names.push(
                brotherhood.files.iter()
                    .map(|&dup_file_idx| DupFileRef {
                        dup_set_idx: brotherhood.dup_set_idx,
                        dup_file_idx,
                    })
                    .filter_map(|dup_file_ref| dup_file_ref.copy_name(dups))
                    .next()
                    .unwrap() // SAFETY: it's not auto solvable if there's no copy named file
            );
        }
    }
    let mut expander = OwningTemplateExpander::new();
    expander
        .set("num", question_idx + 1)
        .set("questions", questions)
        .set("example_1", example_names[0])
        .set("example_2", example_names[1])
        .set("skippable_questions", skippable_questions)
        .set("file_count", removable_count)
        .set("size", file_size::fit_4(removable_len));
    skin.print_owning_expander(&expander, &TextTemplate::from(MD_AUTO_SOLVE));
    Ok(ask!(skin, "Do you want me to automatically stage those copies ?", ('y') {
        ('y', "**Y**es") => {
            for brotherhood in dirs_report.brotherhoods.iter().filter(|b| b.is_auto_solvable) {
                let dup_file_refs = brotherhood.files.iter()
                    .map(|&dup_file_idx| DupFileRef {
                        dup_set_idx: brotherhood.dup_set_idx,
                        dup_file_idx,
                    })
                    .filter(|dup_file_ref| dup_file_ref.is_copy_named(dups));
                for dup_file_ref in dup_file_refs {
                    rr.stage_file(dup_file_ref, dups);
                }
            }
            true
        }
        ('n', "**N**o") => {
            false
        }
        ('e', "**E**nd staging and quit") => {
            rr.quit = true;
            false
        }
    }))
}

static MD_DUP_DIR: &str = r#"

## Staging Question **${num}**/${questions}
The *${directory}* directory contains **${file_count}** files which are all present elsewhere.\
You can remove the whole directory without losing anything.\
This would let you gain **${size}**.\
"#;

/// ask for a dir which contains only duplicates
fn ask_on_dup_dir<'d>(
    question_idx: usize,
    questions: usize,
    dup_dir: &'d DupDir,
    dups: &'d [DupSet],
    skin: &MadSkin,
    rr: &mut RemovalReport<'d>,
) -> anyhow::Result<()> {
    // first we must make sure the dir doesn't contain the last file(s) of a dupset
    let mut file_idxs_per_dupset: FnvHashMap<usize, Vec<usize>> = FnvHashMap::default();
    for file_ref in &dup_dir.files {
        file_idxs_per_dupset.entry(file_ref.dup_set_idx)
            .or_default()
            .push(file_ref.dup_file_idx);
    }
    for (&dup_set_idx, file_idxs) in &file_idxs_per_dupset {
        let dup_set = &dups[dup_set_idx];
        let not_here_or_staged_count = (0..dup_set.files.len())
            .filter(|&dup_file_idx| {
                !rr.staged_removals.contains(&DupFileRef { dup_set_idx, dup_file_idx })
                &&
                !file_idxs.contains(&dup_file_idx)
            })
            .count();
        if not_here_or_staged_count == 0 {
            // dup_set would be removed -> skipping
            return Ok(());
        }
    }
    // now we know we can stage the whole directory
    let removable_len = dup_dir.files.iter()
        .map(|dup_file_ref| dups[dup_file_ref.dup_set_idx].file_len)
        .sum();
    let mut expander = OwningTemplateExpander::new();
    expander
        .set("num", question_idx + 1)
        .set("questions", questions)
        .set("directory", dup_dir.path.to_string_lossy())
        .set("file_count", dup_dir.files.len())
        .set("size", file_size::fit_4(removable_len));
    skin.print_owning_expander(&expander, &TextTemplate::from(MD_DUP_DIR));
    ask!(skin, "What do you want to do with this directory?", ('s') {
        ('r', "Stage the whole directory for **r**emoval") => {
            for &file_ref in &dup_dir.files {
                rr.stage_file(file_ref, dups);
            }
            rr.staged_dir_removals.push(dup_dir.path);
        }
        ('s', "**S**kip and go to next question") => {}
        ('e', "**E**nd staging phase") => { rr.broken = true; }
    });
    Ok(())
}

static MD_BROTHERHOOD: &str = r#"

## Staging Question **${num}**/${questions}
The *${parent}* directory contains **${file_count}** identical files, each one of size **${size}**.
"#;

// ask for a set of identical files in the same directory
fn ask_on_brotherhood(
    question_idx: usize,
    questions: usize,
    brotherhood: &Brotherhood,
    dups: &[DupSet],
    skin: &MadSkin,
    rr: &mut RemovalReport,
) -> anyhow::Result<()> {
    // we check nothing because questions for brotherhoods come before the other ones
    // FIXME we must check it's not autosolved!
    let dup_set = &dups[brotherhood.dup_set_idx];
    let mut expander = OwningTemplateExpander::new();
    expander
        .set("num", question_idx + 1)
        .set("questions", questions)
        .set("parent", brotherhood.parent.to_string_lossy())
        .set("file_count", brotherhood.files.len())
        .set("size", file_size::fit_4(dup_set.file_len));
    skin.print_owning_expander(&expander, &TextTemplate::from(MD_BROTHERHOOD));
    let mut q = Question::new("What do you want to do with these duplicates?");

    struct F<'f> { idx: usize, name: &'f str }
    let mut candidates: Vec<F> = brotherhood.files.iter()
        .map(|&idx| F{ idx, name: dup_set.files[idx].path.file_name().unwrap().to_str().unwrap() })
        .collect();
    candidates.sort_by(|a, b| a.name.cmp(b.name));
    for (i, f) in candidates.iter().enumerate() {
        q.add_answer(
            i + 1,
            format!("keep *{}* and stage other one(s) for removal", f.name),
        );
    }
    q.add_answer('s', "**S**kip and go to next question");
    q.add_answer('e', "**E**nd staging phase");
    q.set_default("s");
    match q.ask(skin)?.as_str() {
        "s" => {}
        "e" => { rr.broken = true; }
        a => {
            if let Ok(a) = a.parse::<usize>() {
                if a == 0 {
                    println!("Options start at 1 - skipping");
                } else {
                    let chosen = &candidates[a - 1];
                    for i in 0..brotherhood.files.len() {
                        if i != chosen.idx {
                            rr.stage_file(brotherhood.file_ref(i), dups);
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

static MD_DIR_PAIR: &str = r#"

## Staging Question **${num}**/${questions}
Left and right directories have **${file_count}** common files for a total duplicate size of **${removable_len}**.
|-:|:-:|:-:|
| |left|right|
|-:|:-:|:-:|
|directory|*${left_path}*|*${right_path}*|
${common_files
|common files|${file_count}|${file_count}|
}
${removable_files
|removable file #${removable_file_idx}|**${left_file_name}**|**${right_file_name}**|
}
|already staged for removal|${removed_left_count}|${removed_right_count}|
|other files|${left_other_count}|${right_other_count}|
|-:
"#;

/// asking the question when left dir and right dir are different
fn ask_on_dir_pair(
    question_idx: usize,
    questions: usize,
    dir_pair: &DirPair,
    dups: &[DupSet],
    skin: &MadSkin,
    rr: &mut RemovalReport,
) -> anyhow::Result<()> {
    // we must recount now because files may have been already
    // staged for removals
    let (mut removed_left_count, mut removed_right_count) = (0, 0);
    let (mut removable_left_count, mut removable_right_count) = (0, 0);
    let mut removable_pairs: Vec<FilePair> = Vec::new();
    let mut removable_len: u64 = 0;
    for file_pair in &dir_pair.file_pairs {
        let removed_left = rr.staged_removals.contains(&file_pair.left_ref());
        let removed_right = rr.staged_removals.contains(&file_pair.right_ref());
        if removed_left {
            removed_left_count += 1;
        } else {
            removable_left_count += 1;
        }
        if removed_right {
            removed_right_count += 1;
        } else {
            removable_right_count += 1;
        }
        if !removed_left && !removed_right {
            removable_pairs.push(*file_pair);
            removable_len += dups[file_pair.dup_set_idx].file_len;
        }
    }
    if removable_pairs.is_empty() {
        mad_print_inline!(skin, "*skipping question because of previously staged removals*\n");
        return Ok(());
    }
    let left_dir_count = dir_pair.key.left_dir.read_dir()?.count();
    if left_dir_count < removed_left_count + removable_left_count {
        println!("skipping question because some files were removed on disk");
        return Ok(());
    }
    let left_other_count = left_dir_count  - removed_left_count - removable_left_count;
    let right_dir_count = dir_pair.key.right_dir.read_dir()?.count();
    if right_dir_count < removed_right_count + removable_right_count {
        println!("skipping question because some files were removed on disk");
        return Ok(());
    }
    let right_other_count = right_dir_count  - removed_right_count - removable_right_count;
    let mut expander = OwningTemplateExpander::new();
    expander
        .set("num", question_idx + 1)
        .set("questions", questions)
        .set("file_count", removable_pairs.len())
        .set("removable_len", file_size::fit_4(removable_len))
        .set("left_path", dir_pair.key.left_dir.to_string_lossy())
        .set("right_path", dir_pair.key.right_dir.to_string_lossy())
        .set("removed_left_count",  removed_left_count)
        .set("removed_right_count", removed_right_count)
        .set("left_other_count", left_other_count)
        .set("right_other_count", right_other_count);
    if removable_pairs.len() <= MAX_LISTED_FILES {
        for (removable_file_idx, file_pair) in removable_pairs.iter().enumerate() {
            expander.sub("removable_files")
                .set("removable_file_idx", removable_file_idx + 1)
                .set("left_file_name",  file_pair.left_ref().file_name(dups))
                .set("right_file_name", file_pair.right_ref().file_name(dups));
            }
    } else {
        expander.sub("common_files");
    }
    skin.print_owning_expander(&expander, &TextTemplate::from(MD_DIR_PAIR));
    ask!(skin, "What do you want to do here?", ('s') {
        ('l', "Stage **l**eft files for removal") => {
            for file_pair in removable_pairs {
                rr.stage_file(file_pair.left_ref(), dups);
            }
        }
        ('r', "Stage **r**ight files for removal") => {
            for file_pair in removable_pairs {
                rr.stage_file(file_pair.right_ref(), dups);
            }
        }
        ('s', "**S**kip and go to next question") => {
            println!("skipped");
        }
        ('e', "**E**nd staging phase") => {
            rr.broken = true;
        }
    });
    Ok(())
}

