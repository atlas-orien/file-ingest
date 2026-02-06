use file_ingest::{NormalizationOptions, normalize_file};
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() < 2 {
        eprintln!("Usage: normalize_md <raw.md> <canonical.md>");
        std::process::exit(1);
    }

    let options = NormalizationOptions::default();
    normalize_file(&args[0], &args[1], &options)?;
    println!("{}", args[1]);
    Ok(())
}
