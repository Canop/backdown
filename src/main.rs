use {
    backdown::*,
    anyhow::Result,
    crossterm::style::{Attribute::*, Color::*},
    minimad::*,
    termimad::*,
};

fn main() -> Result<()> {
    let args: Args = argh::from_env();
    if args.version {
        println!("backdown {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    let root = args.path
        .unwrap_or_else(|| std::env::current_dir().unwrap());
    let skin = make_skin();
    skin.print_text("\n# Phase 1) Analysis");
    mad_print_inline!(skin, "Analyzing directory *$0*...\n", root.to_string_lossy());
    let dup_report = DupReport::build(root, args.only_images)?;
    dup_report.print_summary(&skin);
    if dup_report.is_empty() {
        println!("There's nothing to remove");
        return Ok(());
    }
    let dirs_report = DirsReport::compute(&dup_report.dups)?;
    skin.print_text("\n# Phase 2) Staging: choose files to remove");
    let rr = ask_on_dirs(&dirs_report, &dup_report.dups, &skin)?;
    if rr.is_empty() || rr.quit {
        return Ok(());
    }
    skin.print_text("\n# Phase 3) Review and confirm removals");
    loop {
        ask!(&skin, "What do you want to do now?", {
            ('s', "Review touched **s**ets of identical files") => {
                rr.list_dup_sets(&dup_report.dups, &skin);
            }
            ('f', "Review all **f**iles staged for removal") => {
                rr.list_staged_removals(&dup_report.dups, &skin);
            }
            ('r', "Do the **r**emovals now") => {
                rr.do_the_removal(&dup_report.dups, &skin)?;
                break;
            }
            ('q', "**Q**uit *backdown*, removing nothing") => {
                break;
            }
        });
    }
    Ok(())
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
