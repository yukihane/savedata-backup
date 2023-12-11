mod backup;
mod check;
mod context;
mod search;
use anyhow::{Context, Result};
use backup::backup;
use check::check;
use context::AppContext;
use directories::{ProjectDirs, UserDirs};
use getopts::Options;
use search::save_archive_candidates;
use std::{
    env,
    fs::OpenOptions,
    io::{BufWriter, Write},
};

enum Command {
    Search,
    Backup,
    Check,
}

fn main() -> Result<()> {
    let command = determine_command();

    let ctx = initialize_context()?;

    match command {
        Some(Command::Search) => save_archive_candidates(&ctx)?,
        Some(Command::Backup) => backup(&ctx)?,
        Some(Command::Check) => check(&ctx)?,
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
            ref s if s == "check" => Some(Command::Check),
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

fn print_usage(program: &str, opts: &Options) {
    let brief = format!("Usage:\n{} search\n{} backup -f FILE", program, program);
    print!("{}", opts.usage(&brief));
}
