use std::{collections::BTreeSet, env, path::Path, vec};

use anyhow::{Context, Result};
use directories::UserDirs;
use getopts::Options;
use walkdir::WalkDir;

fn main() -> Result<()> {
    let args = env::args().collect::<Vec<_>>();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("f", "", "backup target file", "FILE");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => panic!("{}", e),
    };

    let command = if !matches.free.is_empty() {
        Some(matches.free[0].clone())
    } else {
        None
    };

    match command.as_ref().map(|e| e.as_str()) {
        Some("search") => generate_targets()?,
        Some("backup") => {
            let file = matches.opt_str("f").context("Not found: file")?;
            backup(&file)?
        }
        _ => {
            print_usage(&program, &opts);
        }
    }
    return Ok(());
}

fn print_usage(program: &str, opts: &Options) {
    let brief = format!("Usage:\n{} search\n{} backup -f FILE", program, program);
    print!("{}", opts.usage(&brief));
}

fn backup(file: &str) -> Result<()> {
    println!("backup: {}", file);
    Ok(())
}

/// バックアップ対象ディレクトリを推定します。
fn generate_targets() -> Result<()> {
    let user_dir = UserDirs::new().context("Not found: user_dir")?;
    let dir = user_dir.document_dir().context("Not found: document_dir")?;

    let filtered_paths = WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .to_lowercase()
                .contains("sav")
        })
        .filter_map(|e| {
            if e.file_type().is_file() {
                if let Some(path) = e.path().parent() {
                    Some(path.to_owned())
                } else {
                    None
                }
            } else if e.file_type().is_dir() {
                Some(e.path().to_owned())
            } else {
                None
            }
        })
        .collect::<BTreeSet<_>>();

    // filtered_paths
    //     .iter()
    //     .for_each(|e| println!("{}", e.display()));

    let mut maybe_parent: Option<&Path> = None;
    let mut save_dirs = vec![];
    for dir in filtered_paths.iter() {
        let is_parent = is_parent(&maybe_parent, &dir);
        if is_parent {
            continue;
        } else {
            save_dirs.push(dir);
            maybe_parent = Some(dir.as_path());
        }
    }

    save_dirs.iter().for_each(|e| println!("{}", e.display()));

    Ok(())
}

fn is_parent(maybe_parent: &Option<&Path>, dir: &Path) -> bool {
    if let Some(parent) = *maybe_parent {
        if dir.starts_with(parent) {
            return true;
        }
    }
    false
}
