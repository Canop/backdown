use {
    backdown::{
        dup_map::DupMap,
    },
    anyhow::Result,
};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("A path must be given as argument");
        return Ok(());
    }
    let root = &args[1];
    println!("root: {:?}", root);
    let dup_map = DupMap::build(root.into())?;
    dbg!(dup_map.len());
    for dup in dup_map.dups.values() {
        for path in &dup.paths {
            println!("{:?}", path);
        }
        println!("----------------------------------------------");
    }
    dbg!(dup_map.seen);
    Ok(())
}
