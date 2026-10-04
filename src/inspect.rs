use anyhow::{bail, Context, Result};
use std::{
    collections::BTreeSet,
    env, fs,
    path::{Path, PathBuf},
};

const MAX_FILE_SIZE: u64 = 512 * 1024 * 1024;

/// Show path-like string constants found in a game executable. These are hints,
/// not proof that the game writes save data there.
pub fn inspect_executable(exe: &Path) -> Result<()> {
    let metadata = fs::metadata(exe).with_context(|| format!("Cannot read {}", exe.display()))?;
    if !metadata.is_file() {
        bail!("Not a file: {}", exe.display());
    }
    if metadata.len() > MAX_FILE_SIZE {
        bail!("File exceeds 512 MiB: {}", exe.display());
    }

    let bytes = fs::read(exe).with_context(|| format!("Cannot read {}", exe.display()))?;
    let mut paths = extract_ascii_paths(&bytes);
    paths.extend(extract_utf16_paths(&bytes));

    if paths.is_empty() {
        println!("No path-like save hints found in {}", exe.display());
        return Ok(());
    }

    let total = paths.len();
    for path in paths.iter().take(100) {
        println!("Hint: {path}");
        if let Some(resolved) = resolve_path(&path, exe) {
            println!(
                "  {}: {}",
                if resolved.exists() {
                    "exists"
                } else {
                    "not found"
                },
                resolved.display()
            );
        }
    }
    if total > 100 {
        println!("... {} more hints omitted", total - 100);
    }
    println!("Review these hints before adding a directory or file to the backup targets.");
    Ok(())
}

fn extract_ascii_paths(bytes: &[u8]) -> BTreeSet<String> {
    let mut result = BTreeSet::new();
    let mut current = Vec::new();
    for &byte in bytes {
        if (0x20..=0x7e).contains(&byte) || byte >= 0x80 {
            current.push(byte);
            if current.len() > 1024 {
                current.clear();
            }
        } else {
            if current.len() >= 4 {
                if let Ok(s) = std::str::from_utf8(&current) {
                    if let Some(path) = possible_path(s) {
                        result.insert(path);
                    }
                }
            }
            current.clear();
        }
    }
    if current.len() >= 4 {
        if let Ok(s) = std::str::from_utf8(&current) {
            if let Some(path) = possible_path(s) {
                result.insert(path);
            }
        }
    }
    result
}

fn extract_utf16_paths(bytes: &[u8]) -> BTreeSet<String> {
    let mut result = BTreeSet::new();
    for offset in 0..2 {
        if offset >= bytes.len() {
            continue;
        }
        let mut current = Vec::new();
        for pair in bytes[offset..].chunks_exact(2) {
            let unit = u16::from_le_bytes([pair[0], pair[1]]);
            if unit >= 0x20 && unit != 0x7f {
                current.push(unit);
                if current.len() > 1024 {
                    current.clear();
                }
            } else {
                if current.len() >= 4 {
                    if let Ok(s) = String::from_utf16(&current) {
                        if let Some(path) = possible_path(&s) {
                            result.insert(path);
                        }
                    }
                }
                current.clear();
            }
        }
        if current.len() >= 4 {
            if let Ok(s) = String::from_utf16(&current) {
                if let Some(path) = possible_path(&s) {
                    result.insert(path);
                }
            }
        }
    }
    result
}

fn possible_path(value: &str) -> Option<String> {
    let value = value.trim_matches(|c: char| c.is_whitespace() || "\"'()[]".contains(c));
    let upper = value.to_ascii_uppercase();
    let anchors = [
        "%APPDATA%",
        "%LOCALAPPDATA%",
        "%USERPROFILE%",
        "%PROGRAMDATA%",
        "%PUBLIC%",
        "\\SAVED GAMES\\",
        "\\MY GAMES\\",
    ];
    let anchor = anchors.iter().filter_map(|a| upper.find(a)).min();
    let drive = upper.as_bytes().windows(3).position(|w| {
        w[0].is_ascii_alphabetic() && w[1] == b':' && (w[2] == b'\\' || w[2] == b'/')
    });
    if let Some(start) = anchor.into_iter().chain(drive).min() {
        let path =
            value[start..].trim_matches(|c: char| c.is_whitespace() || "\"'()[]".contains(c));
        let has_specific_location = if let Some(rest) = path.strip_prefix('%') {
            rest.find('%')
                .map(|end| rest[end + 1..].starts_with(['\\', '/']) && rest.len() > end + 2)
                .unwrap_or(false)
        } else {
            path.len() > 3
        };
        if has_specific_location
            && path.len() <= 260
            && !path.contains(['<', '>', '|', '"', '\n', '\r'])
            && !path.to_ascii_lowercase().contains("assertion failed")
            && (!path.starts_with('%') || path.matches('%').count() == 2)
        {
            return Some(path.to_owned());
        }
    }

    let lower = value.to_lowercase();
    let first = lower.split(['\\', '/']).next().unwrap_or("");
    let save_name = matches!(first, "save" | "saves" | "savedata")
        || lower == "save.dat"
        || lower.ends_with(".sav")
        || lower.ends_with(".save");
    if save_name
        && value.chars().next().is_some_and(char::is_alphanumeric)
        && value.len() <= 260
        && !value.contains([' ', ':', '<', '>', '|', '*', '?'])
    {
        return Some(value.to_owned());
    }
    None
}

fn resolve_path(value: &str, exe: &Path) -> Option<PathBuf> {
    if let Some(rest) = value.strip_prefix('%') {
        let end = rest.find('%')?;
        let variable = &rest[..end];
        let base = env::var(variable).ok()?;
        let suffix = rest[end + 1..].trim_start_matches(['\\', '/']);
        return Some(PathBuf::from(base).join(suffix));
    }
    if value.as_bytes().get(1) == Some(&b':') {
        return Some(PathBuf::from(value));
    }
    if value.starts_with(['\\', '/']) {
        return None;
    }
    exe.parent().map(|parent| parent.join(value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_ascii_and_utf16_paths() {
        let ascii = b"\0%APPDATA%\\Game\\Save\0unrelated\0";
        assert!(extract_ascii_paths(ascii).contains("%APPDATA%\\Game\\Save"));
        let wide: Vec<u8> = "C:\\Game\\セーブ"
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect();
        assert!(extract_utf16_paths(&wide).contains("C:\\Game\\セーブ"));
        assert!(extract_utf16_paths(&[]).is_empty());
    }

    #[test]
    fn finds_paths_but_ignores_plain_messages() {
        assert_eq!(possible_path("Save failed"), None);
        assert_eq!(possible_path("save.dat"), Some("save.dat".to_owned()));
        assert_eq!(possible_path(".sav.save"), None);
        assert_eq!(
            possible_path("%APPDATA%\\Game\\Saveassertion failed: test"),
            None
        );
        assert_eq!(possible_path("%APPDATA%%LOCALAPPDATA%"), None);
        assert_eq!(possible_path("%APPDATA%"), None);
        assert_eq!(possible_path("SaveData"), Some("SaveData".to_owned()));
        assert_eq!(
            possible_path("%LOCALAPPDATA%\\Studio\\Game"),
            Some("%LOCALAPPDATA%\\Studio\\Game".to_owned())
        );
    }
}
