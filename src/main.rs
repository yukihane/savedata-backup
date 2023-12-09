use anyhow::{Context, Result};
use directories::UserDirs;
use walkdir::WalkDir;

fn main() -> Result<()> {
    let user_dir = UserDirs::new().context("Not found: user_dir")?;
    let dir = user_dir.document_dir().context("Not found: document_dir")?;

    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .to_lowercase()
                .contains("sav")
        })
    {
        // println!("{}", entry.path().display());
        println!("{:?}", entry.file_type());
    }

    Ok(())
}
