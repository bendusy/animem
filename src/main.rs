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
            let profile: LocalProfile = read_profile(path)?;
            profile.validate()?;
            println!("profile valid: {}", profile.name);
            Ok(())
        }
        [group, command, path] if group == "extension" && command == "validate" => {
            let profile: ExtensionProfile = read_profile(path)?;
            profile.validate()?;
            println!("extension profile valid: {}", profile.name);
            Ok(())
        }
        [command, path] if command == "plan" => {
            let profile: LocalProfile = read_profile(path)?;
            let plan = MaintenancePlan::from_profile(&profile)?;
            println!("{}", serde_json::to_string_pretty(&plan)?);
            Ok(())
        }
        [command, tail @ ..] if command == "split" => {
            let text = tail.first().map(|s| s.as_str()).unwrap_or("");
            let sections = animem::split_sections(
                "stdin",
                text,
                animem::SplitOptions {
                    cjk_headings: true,
                    ..Default::default()
                },
            )
            .map_err(CliError::Animem)?;
            println!("{}", serde_json::to_string_pretty(&sections)?);
            Ok(())
        }
        _ => Err(CliError::Usage),
    }
}

fn read_profile<T>(path: impl AsRef<Path>) -> Result<T, CliError>
where
    T: serde::de::DeserializeOwned,
{
    let path = path.as_ref();
    let raw = fs::read_to_string(path).map_err(|source| CliError::Read {
        path: path.display().to_string(),
        source,
    })?;
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("json") => serde_json::from_str(&raw).map_err(CliError::Json),
        Some("toml") => toml::from_str(&raw).map_err(CliError::Toml),
        Some(extension) => Err(CliError::UnsupportedExtension(extension.to_string())),
        None => Err(CliError::MissingExtension),
    }
}

fn print_usage() {
    println!(
        "Usage:
  animem profile validate <profile.json>
  animem profile validate <profile.toml>
  animem extension validate <extension-profile.json>
  animem extension validate <extension-profile.toml>
  animem plan <profile.json|profile.toml>
  animem split <text>"
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
    Toml(toml::de::Error),
    UnsupportedExtension(String),
    MissingExtension,
    Profile(ProfileValidationError),
    Animem(animem::AnimemError),
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
            CliError::Toml(err) => write!(f, "invalid TOML: {err}"),
            CliError::UnsupportedExtension(extension) => {
                write!(f, "unsupported profile file extension: {extension}")
            }
            CliError::MissingExtension => {
                write!(f, "profile file must use .json or .toml extension")
            }
            CliError::Profile(err) => write!(f, "invalid profile: {err}"),
            CliError::Animem(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for CliError {}

impl From<serde_json::Error> for CliError {
    fn from(value: serde_json::Error) -> Self {
        CliError::Json(value)
    }
}

impl From<toml::de::Error> for CliError {
    fn from(value: toml::de::Error) -> Self {
        CliError::Toml(value)
    }
}

impl From<ProfileValidationError> for CliError {
    fn from(value: ProfileValidationError) -> Self {
        CliError::Profile(value)
    }
}

impl From<animem::AnimemError> for CliError {
    fn from(value: animem::AnimemError) -> Self {
        CliError::Animem(value)
    }
}
