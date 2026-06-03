use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

use animem::{ExtensionProfile, LocalProfile, MaintenancePlan, ProfileValidationError};

fn main() -> ExitCode {
    match run(env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::from(1)
        }
    }
}

fn run(args: Vec<String>) -> Result<(), CliError> {
    match args.as_slice() {
        [] => {
            print_usage();
            Ok(())
        }
        [arg] if arg == "--help" || arg == "-h" => {
            print_usage();
            Ok(())
        }
        [group, command, path] if group == "profile" && command == "validate" => {
            let profile: LocalProfile = read_json(path)?;
            profile.validate()?;
            println!("profile valid: {}", profile.name);
            Ok(())
        }
        [group, command, path] if group == "extension" && command == "validate" => {
            let profile: ExtensionProfile = read_json(path)?;
            profile.validate()?;
            println!("extension profile valid: {}", profile.name);
            Ok(())
        }
        [command, path] if command == "plan" => {
            let profile: LocalProfile = read_json(path)?;
            let plan = MaintenancePlan::from_profile(&profile)?;
            println!("{}", serde_json::to_string_pretty(&plan)?);
            Ok(())
        }
        _ => Err(CliError::Usage),
    }
}

fn read_json<T>(path: impl AsRef<Path>) -> Result<T, CliError>
where
    T: serde::de::DeserializeOwned,
{
    let path = path.as_ref();
    let raw = fs::read_to_string(path).map_err(|source| CliError::Read {
        path: path.display().to_string(),
        source,
    })?;
    serde_json::from_str(&raw).map_err(CliError::Json)
}

fn print_usage() {
    println!(
        "Usage:
  animem profile validate <profile.json>
  animem extension validate <extension-profile.json>
  animem plan <profile.json>"
    );
}

#[derive(Debug)]
enum CliError {
    Usage,
    Read {
        path: String,
        source: std::io::Error,
    },
    Json(serde_json::Error),
    Profile(ProfileValidationError),
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::Usage => write!(
                f,
                "invalid arguments; run `animem --help` for supported commands"
            ),
            CliError::Read { path, source } => write!(f, "failed to read {path}: {source}"),
            CliError::Json(err) => write!(f, "invalid JSON: {err}"),
            CliError::Profile(err) => write!(f, "invalid profile: {err}"),
        }
    }
}

impl std::error::Error for CliError {}

impl From<serde_json::Error> for CliError {
    fn from(value: serde_json::Error) -> Self {
        CliError::Json(value)
    }
}

impl From<ProfileValidationError> for CliError {
    fn from(value: ProfileValidationError) -> Self {
        CliError::Profile(value)
    }
}
