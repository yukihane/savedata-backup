use std::path::Path;

use crate::{backup::read_paths_from_file, context::AppContext};
use anyhow::Result;
use fs_extra::dir::get_size;

struct Directory<'a> {
    path: &'a Path,
    size: Option<u64>,
}

pub fn check(ctx: &AppContext) -> Result<()> {
    // バックアップ対象ディレクトリを特定
    let target_dir_file = &ctx.target_dir_file;
    if !target_dir_file.exists() {
        return Ok(());
    }
    let dirs = read_paths_from_file(&target_dir_file)?;
    let mut dirs = dirs
        .iter()
        .map(|e| {
            let path = Path::new(e);
            let size = get_size(path).ok();
            Directory { path, size }
        })
        .collect::<Vec<_>>();
    dirs.sort_by(|a, b| b.size.cmp(&a.size));

    dirs.iter().for_each(|e| {
        println!(
            "{}, {}",
            e.size.map_or("none".to_string(), |e| e.to_string()),
            e.path.display()
        );
    });

    Ok(())
}
