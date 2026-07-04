use std::io::Error as IoError;
use std::path::Path;

#[derive(Debug)]
pub enum ResError {
    Io(IoError),
    Project(String),
    NONE,
}

#[derive(Debug)]
pub struct CommandResult {
    pub message: String,
    pub error: Option<ResError>,
}

impl CommandResult {
    pub fn ok() -> Self {
        CommandResult {
            message: String::new(),
            error: None,
        }
    }
}

/// @params cmd: `str` -> Command like 'cargo clippy' \
/// @params args: `Vector<str>` -> Arguments of the command \
/// @params dit: `Path` -> Path of the command execution \
/// @returns CommandResult
pub fn process_command(cmd: &str, args: &[&str], dir: &Path) -> CommandResult {
    let mut res = CommandResult::ok();

    let output = match std::process::Command::new(cmd)
        .args(args)
        .current_dir(dir)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
    {
        Ok(output) => output,
        Err(err) => {
            res.message.push_str(&format!(
                "Error of command '{}' in '{}': {}",
                cmd,
                dir.display(),
                err
            ));
            res.error = Some(ResError::Io(err));
            return res;
        }
    };

    if !output.status.success() {
        res.message.push_str(&format!(
            "The command failed with status: {:?}\n--- STDERR ---\n{}\n--- ------ ---",
            output.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&output.stderr)
        ));
        res.error = Some(ResError::Project(format!(
            "The command '{}' as returned an error code.",
            cmd
        )));
        return res;
    }

    res.message.push_str(&format!(
        "The command '{}' success.\n--- STDOUT ---\n{}",
        cmd,
        String::from_utf8_lossy(&output.stdout)
    ));

    res
}
