use std::{
    collections::BTreeSet,
    env,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, BufWriter, Write},
    path::{Component, Path, PathBuf, Prefix},
    vec,
};

use anyhow::{Context, Result};
use directories::{ProjectDirs, UserDirs};
use getopts::Options;
use walkdir::WalkDir;

struct AppContext {
    config_dir: PathBuf,
    search_dir_file: PathBuf,
    target_dir_file: PathBuf,
    target_file_file: PathBuf,
}

enum Command {
    Search,
    Backup,
}

fn main() -> Result<()> {
    let command = determine_command();

    let ctx = initialize_context()?;

    match command {
        Some(Command::Search) => save_archive_candidates(&ctx)?,
        Some(Command::Backup) => backup(&ctx)?,
        None => {
            initialize_config_if_not_exists(&ctx)?;
            println!("use: {}", ctx.search_dir_file.display());
        }
    }
    return Ok(());
}

fn determine_command() -> Option<Command> {
    let args = env::args().collect::<Vec<_>>();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("f", "", "backup target file", "FILE");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => panic!("{}", e),
    };

    if !matches.free.is_empty() {
        match matches.free[0] {
            ref s if s == "search" => Some(Command::Search),
            ref s if s == "backup" => Some(Command::Backup),
            _ => None,
        }
    } else {
        print_usage(&program, &opts);
        None
    }
}

fn initialize_context() -> Result<AppContext> {
    let config_dir = ProjectDirs::from("com", "yukihane", "savedata-backup")
        .context("Not found: target_file")?
        .config_dir()
        .to_owned();

    let search_dir_file = config_dir.join("search_dir.txt");
    let target_dir_file = config_dir.join("target_dir.txt");
    let target_file_file = config_dir.join("target_file.txt");

    Ok(AppContext {
        config_dir,
        search_dir_file,
        target_dir_file,
        target_file_file,
    })
}

fn initialize_config_if_not_exists(ctx: &AppContext) -> Result<()> {
    let config_dir = &ctx.config_dir;
    if !config_dir.exists() {
        std::fs::create_dir_all(config_dir)?;
    }
    let search_dir_file = &ctx.search_dir_file;
    if !search_dir_file.exists() {
        let mut search_dir_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(search_dir_file)?;
        let mut writer = BufWriter::new(&mut search_dir_file);
        let user_dir = UserDirs::new().context("Not found: user_dir")?;
        let document_dir = user_dir.document_dir().context("Not found: document_dir")?;
        writeln!(writer, "{}", document_dir.display()).unwrap();
        writer.flush().unwrap();
    }

    Ok(())
}

/// アーカイブ対象となるディレクトリの候補を推定し、ファイルに書き出します。
fn save_archive_candidates(ctx: &AppContext) -> Result<()> {
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

/// バックアップ(tarファイル)を生成します。
fn backup(ctx: &AppContext) -> Result<()> {
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
        let src = Path::new(&file);
        let dest = get_dest(src)?;

        tar.append_path_with_name(src, &dest)?;
    }

    Ok(())
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

fn print_usage(program: &str, opts: &Options) {
    let brief = format!("Usage:\n{} search\n{} backup -f FILE", program, program);
    print!("{}", opts.usage(&brief));
}
