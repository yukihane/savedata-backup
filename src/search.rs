use std::{
    collections::BTreeSet,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};

use crate::context::AppContext;
use anyhow::Result;
use walkdir::WalkDir;

/// アーカイブ対象となるディレクトリの候補を推定し、ファイルに書き出します。
pub fn save_archive_candidates(ctx: &AppContext) -> Result<()> {
    let search_dir_file = &ctx.search_dir_file;
    if !search_dir_file.exists() {
        return Err(anyhow::anyhow!("Not found: {}", search_dir_file.display()));
    }
    let reader = BufReader::new(File::open(search_dir_file)?);
    let search_paths = reader
        .lines()
        .into_iter()
        .map(|e| {
            let path = e.unwrap();
            PathBuf::from(&path)
        })
        .collect::<Vec<_>>();

    let targets = search_paths
        .iter()
        .map(|search_path| search_archive_targets(&search_path).unwrap())
        .flatten()
        .collect::<Vec<_>>();

    save_dirs(&ctx, &targets)?;

    Ok(())
}

/// `search_path` 以下に存在するバックアップ対象ディレクトリを推定します。
fn search_archive_targets(search_path: &Path) -> Result<Vec<PathBuf>> {
    let filtered_paths = WalkDir::new(search_path)
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
            save_dirs.push(dir.to_owned());
            maybe_parent = Some(dir.as_path());
        }
    }

    return Ok(save_dirs);
}

/// 推定したバックアップ対象ディレクトリをファイルに書き出します。
fn save_dirs(ctx: &AppContext, save_targets: &Vec<PathBuf>) -> Result<()> {
    // save_dirs.iter().for_each(|e| println!("{}", e.display()));

    let mut target_dir_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&ctx.target_dir_file)?;
    let mut writer = BufWriter::new(&mut target_dir_file);

    save_targets.iter().for_each(|e| {
        writer.write_all(e.to_string_lossy().as_bytes()).unwrap();
        writer.write_all(b"\n").unwrap();
    });

    writer.flush().unwrap();

    let target_file_file = &ctx.target_file_file;
    if !target_file_file.exists() {
        let mut target_file_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(target_file_file)?;
        let mut writer = BufWriter::new(&mut target_file_file);
        writer.flush().unwrap();
    }

    // save_dirs.iter().for_each(|e| {
    //     writeln!(target_file, "{}", e.display()).unwrap();
    // });

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
