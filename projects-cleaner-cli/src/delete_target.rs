use crate::models::ProccessingStats;

pub fn delete_target(paths: Vec<&std::path::Path>, stats: &mut ProccessingStats) {
    if !paths.is_empty() {
        println!("\ndeleted :");
    }
    for path in paths {
        if path.is_dir() {
            match std::fs::remove_dir_all(path) {
                Ok(_) => {
                    stats.successful_actions += 1;
                    println!(" - {}", path.display())
                }
                Err(err) => {
                    stats.failed_actions += 1;
                    eprintln!("Error while removing {}: {}", path.display(), err);
                }
            }
        } else {
            match std::fs::remove_file(path) {
                Ok(_) => {
                    stats.successful_actions += 1;
                    println!(" - {}", path.display())
                }
                Err(err) => {
                    stats.failed_actions += 1;
                    eprintln!("Error while removing {}: {}", path.display(), err);
                }
            }
        }
    }
}
