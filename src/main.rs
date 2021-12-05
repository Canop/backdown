#[macro_use] extern crate cli_log;

use {
    backdown::*,
    anyhow::Result,
    crossterm::style::{Attribute::*, Color::*},
    termimad::*,
};

fn run_app() -> Result<()> {
    let args: Args = argh::from_env();
    if args.version {
        println!("backdown {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    let root = args.path
        .unwrap_or_else(|| std::env::current_dir().unwrap());
    let skin = make_skin();
    info!("root: {:?}", &root);
    skin.print_text("\n# Phase 1) Analysis");
    mad_print_inline!(skin, "Analyzing directory *$0*...\n", root.to_string_lossy());
    let dup_report = time!(
        Info,
        "computing dup sets",
        DupReport::build(root, args.only_images)?,
    );
    dup_report.print_summary(&skin);
    if dup_report.is_empty() {
        println!("There's nothing to remove");
        return Ok(());
    }
    let dirs_report = time!(
        Info,
        "computing dirs report",
        DirsReport::compute(&dup_report.dups)?,
    );
    skin.print_text("\n# Phase 2) Staging: choose files to remove");
    let rr = ask_on_dirs(&dirs_report, &dup_report.dups, &skin)?;
    if rr.is_empty() || rr.quit {
        return Ok(());
    }
    skin.print_text("\n# Phase 3) Review and confirm removals");
    let mut exported = false;
    loop {
        let mut question = Question::new("What do you want to do now?");
        question.add_answer('s', "Review touched **s**ets of identical files");
        if !exported {
            question.add_answer(
                'j',
                "Export touched sets of identical files in a **J**SON file",
            );
        }
        question.add_answer('f', "Review all **f**iles staged for removal");
        question.add_answer('r', "Do the **r**emovals now");
        question.add_answer('q', "**Q**uit *backdown*, removing nothing");
        match question.ask(&skin)?.as_ref() {
            "s" => {
                rr.list_dup_sets(&dup_report.dups, &skin);
            }
            "j" => {
                let value = rr.dup_sets_as_json(&dup_report.dups);
                let path = write_in_file("backdown-report", &value)?;
                mad_print_inline!(skin, "Wrote *$0*\n", path.to_string_lossy());
                exported = true;
            }
            "f" => {
                rr.list_staged_removals(&dup_report.dups, &skin);
            }
            "r" => {
                rr.do_the_removal(&dup_report.dups, &skin)?;
                break;
            }
            "q" => {
                break;
            }
            _ => {} // should not happen
        }
    }
    Ok(())
}

fn main() {
    init_cli_log!();
    if let Err(e) = run_app() {
        eprintln!("{}", e);
    }
    info!("bye");
}

fn make_skin() -> MadSkin {
    let mut skin = MadSkin::default();
    skin.table.align = Alignment::Left;
    skin.headers[0].align = Alignment::Left;
    skin.set_headers_fg(AnsiValue(178));
    skin.bold.set_fg(Yellow);
    skin.italic.set_fg(AnsiValue(204));
    skin.italic.remove_attr(Italic);
    skin.scrollbar.thumb.set_fg(AnsiValue(178));
    skin.code_block.align = Alignment::Center;
    skin
}
