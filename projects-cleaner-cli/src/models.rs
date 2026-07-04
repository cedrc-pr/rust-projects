use clap::Parser;
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub enum DirError {
    DontExist(std::path::PathBuf),
}

impl std::fmt::Display for DirError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DirError::DontExist(dir) => write!(f, "{:?} does not exist", dir),
        }
    }
}

#[derive(Debug, Default)]
pub struct ProccessingStats {
    pub files_treated: i32,
    pub dirs_treated: i32,
    pub successful_actions: i32,
    pub failed_actions: i32,
    pub size_deleted: u64,
}

impl ProccessingStats {
    pub fn new() -> Self {
        Self {
            files_treated: 0,
            dirs_treated: 0,
            successful_actions: 0,
            failed_actions: 0,
            size_deleted: 0,
        }
    }

    pub fn add_size_to_deleted(&mut self, entries: Vec<&walkdir::DirEntry>) {
        for entry in entries {
            let metadata = match std::fs::metadata(entry.path()) {
                Ok(metadata) => metadata,
                Err(err) => {
                    eprintln!("Could not get metadata of {:?}: {}", entry.path(), err);
                    self.failed_actions += 1;
                    continue;
                }
            };
            self.size_deleted += metadata.len()
        }
    }
}

fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    const TB: f64 = GB * 1024.0;

    let b = bytes as f64;
    if b < KB {
        format!("{} bytes", bytes)
    } else if b < MB {
        format!("{:.1} KB", b / KB)
    } else if b < GB {
        format!("{:.1} MB", b / MB)
    } else if b < TB {
        format!("{:.1} GB", b / GB)
    } else {
        format!("{:.1} TB", b / TB)
    }
}

impl Display for ProccessingStats {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\nProcessus Statistics:\n\
            Files processed: {}\n\
            Directories processed: {}\n\
            Successful actions: {}\n\
            Failed actions: {}\n\
            Total size cleaned: {}",
            self.files_treated,
            self.dirs_treated,
            self.successful_actions,
            self.failed_actions,
            format_bytes(self.size_deleted)
        )
    }
}

#[derive(Debug, Parser)]
#[command(
    about = "Project deposit cleaner, it searches for project directories and deletes temporary files and executables"
)]
pub struct Cli {
    #[arg(
        long,
        short,
        help = "Don't ask to keep or delete for : node_modules/, dist/, target/"
    )]
    pub force: bool,
    #[arg(long, short, help = "Use mrclean")]
    pub mrclean: bool,
    #[arg(help = "Directories to ckeck")]
    pub dirs: Vec<std::path::PathBuf>,
    #[arg(short, long, num_args = 1.., help = "to add extensions to the config file")]
    pub add: Vec<String>,
    #[arg(short, long, num_args = 1.., help = "to remove extensions form the config file")]
    pub remove: Vec<String>,
}
