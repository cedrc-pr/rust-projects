use crate::{delete_target::delete_target, models::ProccessingStats};

pub fn mrclean(dir: std::path::PathBuf, stats: &mut ProccessingStats, extensions: &[String]) {
    let files_to_ask = get_files_to_ask(&dir, extensions);
    let to_delete = ask_what_to_delete(&files_to_ask, &dir);
    let to_delete_dir_entry = convert_to_delete_to_dir_entry(&files_to_ask, to_delete);
    stats.add_size_to_deleted(to_delete_dir_entry.clone());
    delete_target(
        to_delete_dir_entry
            .into_iter()
            .map(|entry| entry.path())
            .collect(),
        stats,
    );
}

fn ask_what_to_delete(files_to_ask: &[walkdir::DirEntry], dir: &std::path::Path) -> Vec<String> {
    if files_to_ask.is_empty() {
        return vec![];
    }
    match inquire::Confirm::new(&format!(
        "Check {} file(s) in {:?} ?",
        files_to_ask.len(),
        dir
    ))
    .prompt()
    {
        Ok(answer) => {
            if answer {
                let file_names: Vec<String> = files_to_ask
                    .iter()
                    .map(|file| file.file_name().to_string_lossy().into_owned())
                    .collect();
                let to_delete =
                    match inquire::MultiSelect::new("Delete :", file_names.clone()).prompt() {
                        Ok(to_delete) => to_delete,
                        Err(error) => {
                            eprintln!("Inquire error: {}", error);
                            return vec![];
                        }
                    };
                return to_delete;
            }
        }
        Err(error) => eprintln!("Inquire error: {}", error),
    }
    vec![]
}

fn convert_to_delete_to_dir_entry(
    files_to_ask: &Vec<walkdir::DirEntry>,
    to_delete: Vec<String>,
) -> Vec<&walkdir::DirEntry> {
    let mut to_delete_path: Vec<&walkdir::DirEntry> = vec![];
    for file in files_to_ask {
        let name = match file.file_name().to_str() {
            Some(name) => name,
            None => continue,
        };
        if to_delete.contains(&name.to_string()) {
            to_delete_path.push(file);
        }
    }
    to_delete_path
}

fn get_files_to_ask(dir: &std::path::Path, extensions: &[String]) -> Vec<walkdir::DirEntry> {
    let mut files_to_ask: Vec<walkdir::DirEntry> = vec![];
    for entry in walkdir::WalkDir::new(dir)
        .max_depth(1)
        .into_iter()
        .filter_map(Result::ok)
    {
        if entry.file_type().is_file() {
            let name = match entry.file_name().to_str() {
                Some(name) => name,
                None => continue,
            };
            if !extensions.iter().any(|ext| name.ends_with(ext)) {
                files_to_ask.push(entry);
            }
        }
    }
    files_to_ask
}
