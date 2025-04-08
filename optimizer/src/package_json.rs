use std::io::Result;
use std::path::{Path, PathBuf};

pub fn find_modules(
    src_dir: &Path,
    vendor_dirs: Vec<String>,
    files: &mut Vec<PathBuf>,
) -> Result<()> {
    for root in &vendor_dirs {
        find_files(Path::new(root), files)?;
    }
    find_files(src_dir, files)
}

fn find_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                find_files(&path, files)?;
            } else if should_capture_file(&path) {
                files.push(path);
            }
        }
    } else if should_capture_file(dir) {
        files.push(dir.to_path_buf());
    }
    Ok(())
}

fn should_capture_file(path: &Path) -> bool {
    let ext = path.extension().and_then(|p| p.to_str());
    matches!(
        ext,
        Some("ts" | "tsx" | "js" | "jsx" | "mjs" | "mts" | "mtsx" | "mjsx")
    )
}
