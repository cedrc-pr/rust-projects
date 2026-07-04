use crate::{
    delete_target::delete_target, models::ProccessingStats, process_command::CommandResult,
};

pub fn clean_project(
    path: &std::path::Path,
    stats: &mut ProccessingStats,
    to_remove_folder_name: &str,
    force: bool,
) -> CommandResult {
    let to_remove_folder = path.join(to_remove_folder_name);
    if !to_remove_folder.exists() {
        return CommandResult {
            message: format!(
                "No {} found to clean in {}",
                to_remove_folder_name,
                path.display()
            ),
            error: None,
        };
    }
    let entries: Vec<walkdir::DirEntry> = walkdir::WalkDir::new(&to_remove_folder)
        .into_iter()
        .filter_map(Result::ok)
        .collect();

    let entry_refs: Vec<&walkdir::DirEntry> = entries.iter().collect();
    let detete = if force {
        force
    } else {
        let Ok(res) =
            inquire::Confirm::new(&format!("Do you want to delete: {:#?}", to_remove_folder))
                .with_default(true)
                .prompt()
        else {
            return CommandResult {
                message: "Inquire error".to_string(),
                error: None,
            };
        };
        res
    };
    if detete {
        stats.add_size_to_deleted(entry_refs);
        delete_target(vec![&to_remove_folder], stats);
        return CommandResult {
            message: format!("{} deleted", to_remove_folder.display()),
            error: None,
        };
    }
    CommandResult {
        message: "finished".to_string(),
        error: None,
    }
}
