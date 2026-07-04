use crate::{
    clean_project::clean_project, models::ProccessingStats, mrclean::mrclean,
    process_command::CommandResult,
};

/// Represent the projects can be treated
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ProjectType {
    Rust,
    NodeJs,
    C,
    Unknown,
}

impl ProjectType {
    pub fn new(dir: &std::path::Path) -> Self {
        if dir.join("Cargo.toml").exists() {
            return Self::Rust;
        }
        if dir.join("package.json").exists() {
            return Self::NodeJs;
        }
        if dir.join("Makefile").exists() {
            return Self::C;
        }
        ProjectType::Unknown
    }

    pub fn ingore_dir(self, entry: &walkdir::DirEntry) -> bool {
        let name = match entry.file_name().to_str() {
            Some(name) => name,
            None => return false,
        };
        if name == ".vscode" || name == ".git" {
            return true;
        }
        match self {
            ProjectType::Rust => {
                if name == "target" {
                    return true;
                }
            }
            ProjectType::NodeJs => {
                if name == "node_modules" || name == "dist" {
                    return true;
                }
            }
            _ => return false,
        }
        false
    }

    /// List all projects can be treated
    pub const ALL_PROJECT_TYPES: [ProjectType; 3] =
        [ProjectType::Rust, ProjectType::NodeJs, ProjectType::C];

    /// Return the file or dir marker of the type of the project
    pub fn maker_file(&self) -> &'static str {
        match self {
            Self::Rust => "Cargo.toml",
            Self::NodeJs => "package.json",
            Self::C => "Makefile",
            Self::Unknown => "",
        }
    }

    /// Clean the path given
    pub fn clean(
        &self,
        path: &std::path::Path,
        stats: &mut ProccessingStats,
        force: bool,
    ) -> CommandResult {
        match self {
            Self::Rust => clean_project(path, stats, "target", force),
            Self::NodeJs => {
                let res = clean_project(path, stats, "node_modules", force);
                if res.error.is_some() {
                    return res;
                }
                clean_project(path, stats, "dist", force)
            }
            Self::C => {
                mrclean(
                    path.to_path_buf(),
                    stats,
                    &[".c".to_string(), ".h".to_string(), "Makefile".to_string()],
                );
                CommandResult {
                    message: "project in C cleaned".to_string(),
                    error: None,
                }
            }
            Self::Unknown => CommandResult {
                message: String::from("Unknown project type, nothing to clean\n"),
                error: None,
            },
        }
    }
}
