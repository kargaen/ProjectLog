use std::fs;
use std::path::{Path, PathBuf};

use crate::models::repository_traits::project_repository::ProjectRepository;
use crate::log;

pub struct FileProjectRepository {
    data_dir: PathBuf,
}

impl FileProjectRepository {
    pub fn new(data_dir: &Path) -> Self {
        Self {
            data_dir: data_dir.to_path_buf(),
        }
    }
}

impl ProjectRepository for FileProjectRepository {
    fn load(&self) -> Vec<String> {
        let path = self.data_dir.join("projects.dat");
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

    fn save(&self, projects: &[String]) {
        log!("save {} projects", projects.len());
        let path = self.data_dir.join("projects.dat");
        let mut content = projects.join("\n");
        if !projects.is_empty() {
            content.push('\n');
        }
        fs::write(path, content).expect("failed to save projects");
    }
}
