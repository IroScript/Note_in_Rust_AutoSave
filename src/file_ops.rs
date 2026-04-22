// ============================================================
//  file_ops.rs
//
//  Mirrors the Python file-operation methods:
//    • create_new_file()
//    • auto_save()            (called on every KeyRelease)
//    • delete_current_file()  (Alt+Delete)
//    • rename_file()          (Ctrl+R)
//    • open_save_folder()     (Ctrl+O)
//
//  The save folder is:  ~/Desktop/Irak Notes Auto Saved/
//  Note files are named  Note_1.txt, Note_2.txt, …
// ============================================================

use std::fs;
use std::path::{Path, PathBuf};

// ── resolve save folder (mirrors __init__ desktop_path logic) ─────────────────

/// Returns `~/Desktop/Irak Notes Auto Saved`, creating it if needed.
/// Falls back to `~/Irak Notes Auto Saved` if Desktop doesn't exist.
pub fn resolve_save_folder() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    // Try Desktop first (Windows / macOS / some Linux)
    let desktop = home.join("Desktop").join("Irak Notes Auto Saved");
    if desktop.parent().map(|p| p.exists()).unwrap_or(false) {
        fs::create_dir_all(&desktop).ok();
        return desktop;
    }
    // Fallback: home directory
    let fallback = home.join("Irak Notes Auto Saved");
    fs::create_dir_all(&fallback).ok();
    fallback
}

// ── create_new_file ───────────────────────────────────────────────────────────

/// Mirrors `create_new_file()`.
/// Scans the save folder, finds the highest Note_N.txt number,
/// creates Note_(N+1).txt, and returns its path.
pub fn create_new_file(save_folder: &Path) -> Result<PathBuf, String> {
    let entries = fs::read_dir(save_folder)
        .map_err(|e| e.to_string())?;

    let max_num: u64 = entries
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let name = e.file_name().into_string().ok()?;
            if name.starts_with("Note_") && name.ends_with(".txt") {
                name[5..name.len() - 4].parse::<u64>().ok()
            } else {
                None
            }
        })
        .max()
        .unwrap_or(0);

    let new_path = save_folder.join(format!("Note_{}.txt", max_num + 1));

    // Create empty file (mirrors `with open(self.current_file, 'w') as f: pass`)
    fs::write(&new_path, b"").map_err(|e| e.to_string())?;

    Ok(new_path)
}

// ── auto_save ─────────────────────────────────────────────────────────────────

/// Mirrors `auto_save()`.
/// Writes `content` to `file_path` only when the content has actually changed.
/// Returns Ok(true) if a write happened, Ok(false) if content was identical.
pub fn auto_save(
    file_path: &Path,
    content: &str,
    last_saved_content: &str,
) -> Result<bool, String> {
    if content == last_saved_content {
        return Ok(false); // nothing changed – mirrors the early-return guard
    }
    fs::write(file_path, content.as_bytes()).map_err(|e| e.to_string())?;
    Ok(true)
}

// ── delete_current_file ───────────────────────────────────────────────────────

/// Mirrors `delete_current_file()` minus the messagebox confirm (handled by UI).
/// Deletes `file_path` and returns Ok(()) on success.
pub fn delete_file(file_path: &Path) -> Result<(), String> {
    fs::remove_file(file_path).map_err(|e| e.to_string())
}

// ── rename_file ───────────────────────────────────────────────────────────────

/// Mirrors `validate_and_rename()` inside `rename_file()`.
/// Returns the new PathBuf on success, or an error string to display.
pub fn rename_file(
    current_path: &Path,
    save_folder: &Path,
    new_name: &str,
) -> Result<PathBuf, String> {
    // Validation: not empty
    if new_name.trim().is_empty() {
        return Err("Filename cannot be empty".to_string());
    }

    // Ensure .txt extension (mirrors `if not new_name.endswith('.txt'):`)
    let mut name = new_name.trim().to_string();
    if !name.to_lowercase().ends_with(".txt") {
        name.push_str(".txt");
    }

    // Validation: invalid characters (mirrors `invalid_chars = '<>:"/\\|?*'`)
    let invalid_chars = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
    if let Some(bad) = name.chars().find(|c| invalid_chars.contains(c)) {
        return Err(format!("Filename cannot contain '{}'", bad));
    }

    let new_path = save_folder.join(&name);

    // Check if target already exists and is a different file
    if new_path.exists() && new_path != current_path {
        return Err("A file with this name already exists".to_string());
    }

    fs::rename(current_path, &new_path).map_err(|e| e.to_string())?;
    Ok(new_path)
}

// ── open_save_folder ─────────────────────────────────────────────────────────

/// Mirrors `open_save_folder()`.
/// Uses the `open` crate which dispatches to os.startfile / xdg-open / open.
pub fn open_save_folder(save_folder: &Path) -> Result<(), String> {
    open::that(save_folder).map_err(|e| e.to_string())
}
