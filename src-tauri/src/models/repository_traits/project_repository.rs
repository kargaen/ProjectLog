pub trait ProjectRepository {
    /// Load the ordered list of permanent project names.
    fn load(&self) -> Vec<String>;

    /// Persist the ordered list of permanent project names.
    fn save(&self, projects: &[String]);
}
