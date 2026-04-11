use std::fs;
use std::path::Path;

/// Load projects from projects.dat, one per line.
pub fn load(data_dir: &Path) -> Vec<String> {
    let path = data_dir.join("projects.dat");
    if !path.exists() {
        return Vec::new();
    }
    fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect()
}

/// Save projects to projects.dat, one per line.
pub fn save(data_dir: &Path, projects: &[String]) {
    let path = data_dir.join("projects.dat");
    let mut content = projects.join("\n");
    if !projects.is_empty() {
        content.push('\n');
    }
    fs::write(path, content).expect("failed to save projects");
}
