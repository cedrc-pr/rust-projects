pub mod clean_project;
pub mod config;
pub mod delete_target;
pub mod models;
pub mod mrclean;
pub mod process_command;
pub mod project;

#[cfg(test)]
mod tests {
    use crate::process_command::process_command;
    use std::path::Path;

    #[test]
    fn test_process_command() {
        let args = vec!["Hello world"];
        let result = process_command("echo", args.as_slice(), Path::new("./"));
        assert!(
            result.error.is_none(),
            "The command 'echo' failed : {:?}",
            result.error
        );
        assert!(
            result.message.contains("Hello world"),
            "The message does not contain the expected output."
        );
    }
}
