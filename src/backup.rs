use crate::context::AppContext;
use anyhow::Result;
use glob::glob;
use std::io::BufRead;
use std::{
    fs::{File, OpenOptions},
    io::BufReader,
    path::{Component, Path, PathBuf, Prefix},
};

/// バックアップ(tarファイル)を生成します。
pub fn backup(ctx: &AppContext) -> Result<()> {
    // バックアップ対象ディレクトリを特定
    let target_dir_file = &ctx.target_dir_file;
    let dirs = if target_dir_file.exists() {
        read_paths_from_file(&target_dir_file)?
    } else {
        vec![]
    };

    // バックアップ対象ファイルを特定
    let target_file_file = &ctx.target_file_file;
    let files = if target_file_file.exists() {
        read_paths_from_file(&target_file_file)?
    } else {
        vec![]
    };

    let mut tar_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("archive.tar")?;
    let mut tar = tar::Builder::new(&mut tar_file);

    for path in dirs {
        let src = Path::new(&path);
        let dest = get_dest(src)?;

        tar.append_dir_all(&dest, src)?;
    }

    for file in files {
        glob(&file)?.into_iter().map(|e| e.unwrap()).for_each(|e| {
            let dest = get_dest(&e).unwrap();

            tar.append_path_with_name(&e, &dest).unwrap();
        });
    }

    Ok(())
}

fn read_paths_from_file(file: &Path) -> Result<Vec<String>> {
    let reader = BufReader::new(File::open(file)?);
    let paths = reader
        .lines()
        .into_iter()
        .map(|e| {
            let path = e.unwrap();
            // println!("{}", path);
            // Path::new(&path)
            //     .canonicalize()
            //     .context("Not found: canonicalize")
            //     .map(|e| e.to_owned())
            path
        })
        .collect::<Vec<_>>();

    Ok(paths)
}

fn get_dest(src: &Path) -> Result<PathBuf> {
    let mut iter = src.components().into_iter();
    let prefix = iter.next().unwrap();
    let rootdir = iter.next().unwrap();
    let rel_path = src.strip_prefix(&prefix)?.strip_prefix(&rootdir)?;

    let drive_letter = match prefix {
        Component::Prefix(prefix_component) => match prefix_component.kind() {
            Prefix::Disk(drive_letter) => String::from_utf8(vec![drive_letter]).unwrap(),
            _ => panic!("not disk"),
        },
        _ => panic!("not disk"),
    };
    let drive_letter = PathBuf::from(drive_letter);
    let dest = drive_letter.join(rel_path);

    Ok(dest)
}
