use std::path::Path;

use crate::models::repository_traits::project_repository::ProjectRepository;
use crate::repositories::file_project_repository::FileProjectRepository;

pub fn load(data_dir: &Path) -> Vec<String> {
    FileProjectRepository::new(data_dir).load()
}

pub fn save(data_dir: &Path, projects: &[String]) {
    FileProjectRepository::new(data_dir).save(projects);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
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
