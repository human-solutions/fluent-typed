use std::fs;
use std::path::{Path, PathBuf};

pub trait Traversable {
    fn gather_all_files(
        &self,
        condition: impl Fn(&Path) -> bool,
    ) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>>;
}

impl Traversable for Path {
    fn gather_all_files(
        &self,
        condition: impl Fn(&Path) -> bool,
    ) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        let mut paths = Vec::new();

        if self.is_file() && condition(self) {
            paths.push(self.to_path_buf());
        } else if self.is_dir() {
            gather_paths_recursive(self, &mut paths, &condition)?;
        }

        Ok(paths)
    }
}

fn gather_paths_recursive(
    dir: &Path,
    paths: &mut Vec<PathBuf>,
    condition: &impl Fn(&Path) -> bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let entries = fs::read_dir(dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && condition(&path) {
            paths.push(path);
        } else if path.is_dir() {
            gather_paths_recursive(&path, paths, condition)?;
        }
    }
    Ok(())
}
