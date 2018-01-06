use std::path::PathBuf;
use std::env;
use std::fs;

// Find project root directory from current directory
pub fn project_root_path() -> Option<PathBuf> {
    env::current_dir().ok().and_then(|mut cwd| {
        loop {
            cwd.push("Cargo.toml");
            if fs::metadata(cwd.as_path())
                .map(|meta| meta.is_file())
                .unwrap_or(false)
            {
                cwd.pop();
                return Some(cwd);
            }

            cwd.pop();
            if !cwd.pop() {
                return None;
            }
        }
    })
}
