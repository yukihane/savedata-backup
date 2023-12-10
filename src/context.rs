use std::path::PathBuf;

pub struct AppContext {
    pub config_dir: PathBuf,
    pub search_dir_file: PathBuf,
    pub target_dir_file: PathBuf,
    pub target_file_file: PathBuf,
}
