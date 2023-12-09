use std::{
    collections::{BTreeSet, HashSet},
    path::Path,
    vec,
};

use anyhow::{Context, Result};
use directories::UserDirs;
use walkdir::WalkDir;

fn main() -> Result<()> {
    generate_targets()?;
    Ok(())
}

/// バックアップ対象ディレクトリを推定します。
fn generate_targets() -> Result<()> {
    let user_dir = UserDirs::new().context("Not found: user_dir")?;
    let dir = user_dir.document_dir().context("Not found: document_dir")?;

    let new_dir = WalkDir::new(dir)
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

    new_dir.iter().for_each(|e| println!("{}", e.display()));

    let mut maybe_parent: Option<&Path> = None;
    let mut save_dirs = vec![];
    for dir in new_dir.iter() {
        let is_parent = is_parent(&maybe_parent, &dir);
        if is_parent {
            continue;
        } else {
            save_dirs.push(dir);
            maybe_parent = Some(dir.as_path());
        }
    }

    // save_dirs.iter().for_each(|e| println!("{}", e.display()));

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
