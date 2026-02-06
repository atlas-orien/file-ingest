use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use file_ingest::{NormalizationOptions, normalize_markdown};

fn usage() -> String {
    let exe = env::args()
        .next()
        .and_then(|p| Path::new(&p).file_name().map(|s| s.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "file-ingest-md".to_string());
    format!(
        "Usage:\n  {exe} -t <type> -i <input> -o <output_dir>\n\nExample:\n  {exe} -t pdf -i ./docs/sample.pdf -o ./out"
    )
}

fn require_value<'a>(args: &'a [String], i: &mut usize, flag: &str) -> Result<&'a str, String> {
    *i += 1;
    args.get(*i)
        .map(|s| s.as_str())
        .ok_or_else(|| format!("missing value for {flag}"))
}

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        return Err(usage());
    }

    let mut file_type: Option<String> = None;
    let mut input_path: Option<PathBuf> = None;
    let mut output_dir: Option<PathBuf> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-t" | "--type" => {
                let v = require_value(&args, &mut i, "-t")?;
                file_type = Some(v.to_string());
            }
            "-i" | "--input" => {
                let v = require_value(&args, &mut i, "-i")?;
                input_path = Some(PathBuf::from(v));
            }
            "-o" | "--output" => {
                let v = require_value(&args, &mut i, "-o")?;
                output_dir = Some(PathBuf::from(v));
            }
            "-h" | "--help" => {
                return Err(usage());
            }
            other => {
                return Err(format!("unknown argument: {other}\n\n{}", usage()));
            }
        }
        i += 1;
    }

    let _ = file_type.ok_or_else(|| format!("missing -t\n\n{}", usage()))?;
    let input_path = input_path.ok_or_else(|| format!("missing -i\n\n{}", usage()))?;
    let output_dir = output_dir.ok_or_else(|| format!("missing -o\n\n{}", usage()))?;

    fs::create_dir_all(&output_dir).map_err(|e| format!("create output dir failed: {e}"))?;

    let stem = input_path
        .file_stem()
        .and_then(OsStr::to_str)
        .ok_or_else(|| "invalid input filename".to_string())?;
    let output_path = output_dir.join(format!("{stem}.md"));

    let markdown = file_ingest::to_markdown(&input_path)
        .map_err(|e| format!("convert failed: {e}"))?;
    fs::write(&output_path, &markdown).map_err(|e| format!("write output failed: {e}"))?;

    let canonical_path = output_dir.join(format!("{stem}.canonical.md"));
    let options = NormalizationOptions::default();
    let canonical = normalize_markdown(&markdown, &options);
    fs::write(&canonical_path, canonical)
        .map_err(|e| format!("write canonical output failed: {e}"))?;

    println!("{}", output_path.display());
    println!("{}", canonical_path.display());
    Ok(())
}
