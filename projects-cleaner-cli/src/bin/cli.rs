use std::path::PathBuf;

use clap::Parser;
use hoarder_dragon::{
    config::Config,
    models::{Cli, DirError, ProccessingStats},
    mrclean::mrclean,
    process_command::CommandResult,
    project::ProjectType,
};

fn main() {
    let mut cli = Cli::parse();
    let config = Config::new(cli.add, cli.remove);
    let mut stats = ProccessingStats::new();
    println!("\nStarting cleanup process...");

    check_given_dirs(&mut cli.dirs).unwrap_or_else(|errs| {
        errs.iter()
            .for_each(|err| eprintln!("Invalid dir: {}", err))
    });

    for dir in cli.dirs {
        println!("\nProcessing directory: {}", dir.display());
        let to_mrclean = clean_dir(dir, &mut stats, cli.force);
        if cli.mrclean {
            println!("\nMrclean processing ...\n");
            for dir in to_mrclean {
                mrclean(dir, &mut stats, &config.extensions);
            }
        }
    }

    println!("{}", stats);
    if let Err(err) = config.save() {
        eprintln!("Could not write config in config file: {}", err);
    };

    if stats.failed_actions > 0 {
        eprintln!("\nSome cleanup actions failed!");
        std::process::exit(1);
    } else {
        println!("\nCleanup completed successfully!");
    }
}

fn check_given_dirs(dirs: &mut Vec<std::path::PathBuf>) -> Result<(), Vec<DirError>> {
    let mut errors = Vec::new();
    dirs.retain(|path| {
        if path.exists() {
            true
        } else {
            errors.push(DirError::DontExist(path.clone()));
            false
        }
    });
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn clean_dir(dir: PathBuf, stats: &mut ProccessingStats, force: bool) -> Vec<std::path::PathBuf> {
    let mut to_mrclean = vec![];
    let mut it = walkdir::WalkDir::new(&dir).into_iter();
    while let Some(entry_result) = it.next() {
        let entry = match entry_result {
            Ok(entry) => entry,
            Err(err) => {
                stats.failed_actions += 1;
                eprintln!("Error while getting DirEntry: {}", err);
                continue;
            }
        };
        if !entry.file_type().is_dir() {
            stats.files_treated += 1;
            continue;
        }
        stats.dirs_treated += 1;
        let parent_project = match entry.path().parent().map(|p| p.to_path_buf()) {
            Some(path) => ProjectType::new(&path),
            None => ProjectType::Unknown,
        };
        if parent_project.ingore_dir(&entry) {
            it.skip_current_dir();
        } else {
            let project = ProjectType::new(entry.path());
            if project != ProjectType::Unknown {
                println!(
                    "\nProject in {} is type of: {:#?}",
                    entry.path().display(),
                    project
                );
                let res: CommandResult = project.clean(entry.path(), stats, force);
                if res.error.is_some() {
                    stats.failed_actions += 1;
                    eprintln!("Error while cleaning the project:\n{}", res.message);
                }
            } else if in_a_git_repo(entry.path()) {
                to_mrclean.push(entry.path().to_path_buf());
            }
        }
    }
    to_mrclean
}

fn in_a_git_repo(path: &std::path::Path) -> bool {
    for parent in path.ancestors() {
        if parent.join(".git").exists() {
            return true;
        }
    }
    false
}
