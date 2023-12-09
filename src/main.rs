use std::path::Path;

use anyhow::{Context, Result};
use directories::UserDirs;

fn main() -> Result<()> {
    let user_dir = UserDirs::new().context("Not found: user_dir")?;
    let dir: &Path = user_dir.document_dir().context("Not found: document_dir")?;

    println!("{:?}", dir);

    Ok(())
}
