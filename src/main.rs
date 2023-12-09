use directories::UserDirs;

fn main() {
    let user_dir = UserDirs::new().unwrap();
    let dir = user_dir.document_dir().unwrap();
}
