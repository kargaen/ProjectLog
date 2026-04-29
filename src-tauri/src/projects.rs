use std::fs;
use std::path::Path;

use crate::log;

/// Load projects from projects.dat, one per line.
pub fn load(data_dir: &Path) -> Vec<String> {
    let path = data_dir.join("projects.dat");
    if !path.exists() {
        log!("projects.dat missing; starting with empty project list");
        return Vec::new();
    }
    let projects: Vec<String> = fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect();
    log!("loaded {} projects", projects.len());
    projects
}

/// Save projects to projects.dat, one per line.
pub fn save(data_dir: &Path, projects: &[String]) {
    log!("save {} projects", projects.len());
    let path = data_dir.join("projects.dat");
    let mut content = projects.join("\n");
    if !projects.is_empty() {
        content.push('\n');
    }
    fs::write(path, content).expect("failed to save projects");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("projectlog-{name}-{stamp}"));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn save_and_load_projects_roundtrip() {
        let dir = temp_dir("projects");
        let projects = vec!["Alpha".to_string(), "Beta".to_string()];

        save(&dir, &projects);

        assert_eq!(load(&dir), projects);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn load_returns_empty_when_file_is_missing() {
        let dir = temp_dir("projects-missing");

        assert!(load(&dir).is_empty());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn load_skips_blank_lines() {
        let dir = temp_dir("projects-blanks");
        fs::write(
            dir.join("projects.dat"),
            "Alpha\n\nBeta\n\nGamma\n",
        )
        .unwrap();

        assert_eq!(
            load(&dir),
            vec!["Alpha".to_string(), "Beta".to_string(), "Gamma".to_string()]
        );
        let _ = fs::remove_dir_all(dir);
    }
}
