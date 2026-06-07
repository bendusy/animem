use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn animem() -> Command {
    Command::new(env!("CARGO_BIN_EXE_animem"))
}

#[test]
fn profile_validate_accepts_example_profile() {
    assert_stdout(
        animem()
            .args(["profile", "validate", "examples/profile.example.json"])
            .output()
            .expect("run animem profile validate"),
        "profile valid: example\n",
    );
}

#[test]
fn profile_validate_accepts_toml_profile() {
    assert_stdout(
        animem()
            .args(["profile", "validate", "examples/profile.example.toml"])
            .output()
            .expect("run animem profile validate"),
        "profile valid: example\n",
    );
}

#[test]
fn extension_validate_accepts_example_profile() {
    assert_stdout(
        animem()
            .args([
                "extension",
                "validate",
                "examples/extension-profile.example.json",
            ])
            .output()
            .expect("run animem extension validate"),
        "extension profile valid: example-extension\n",
    );
}

#[test]
fn extension_validate_accepts_toml_profile() {
    assert_stdout(
        animem()
            .args([
                "extension",
                "validate",
                "examples/extension-profile.example.toml",
            ])
            .output()
            .expect("run animem extension validate"),
        "extension profile valid: example-extension\n",
    );
}

#[test]
fn plan_prints_maintenance_plan_json() {
    let output = animem()
        .args(["plan", "examples/profile.example.json"])
        .output()
        .expect("run animem plan");

    assert_plan_output(output);
}

#[test]
fn plan_accepts_toml_profile() {
    let output = animem()
        .args(["plan", "examples/profile.example.toml"])
        .output()
        .expect("run animem plan");

    assert_plan_output(output);
}

#[test]
fn scan_prints_storage_free_scan_plan_json() {
    let output = animem()
        .args(["scan", "examples/profile.example.json"])
        .output()
        .expect("run animem scan");

    assert_scan_output(output);
}

#[test]
fn scan_accepts_toml_profile() {
    let output = animem()
        .args(["scan", "examples/profile.example.toml"])
        .output()
        .expect("run animem scan");

    assert_scan_output(output);
}

#[test]
fn profile_validate_rejects_missing_sources() {
    let path = temp_json_path("animem-invalid-profile");
    fs::write(
        &path,
        r#"{
  "schema_version": "1",
  "name": "invalid",
  "sources": []
}"#,
    )
    .expect("write temp profile");

    let output = animem()
        .args(["profile", "validate", path.to_str().unwrap()])
        .output()
        .expect("run animem profile validate");

    let _ = fs::remove_file(&path);

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("sources: at least one source is required"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn profile_validate_rejects_unknown_extension() {
    let path = temp_path("animem-profile", "txt");
    fs::write(
        &path,
        r#"{
  "schema_version": "1",
  "name": "example",
  "sources": []
}"#,
    )
    .expect("write temp profile");

    let output = animem()
        .args(["profile", "validate", path.to_str().unwrap()])
        .output()
        .expect("run animem profile validate");

    let _ = fs::remove_file(&path);

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("unsupported profile file extension"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn temp_json_path(prefix: &str) -> std::path::PathBuf {
    temp_path(prefix, "json")
}

fn temp_path(prefix: &str, extension: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "{prefix}-{}-{nanos}.{extension}",
        std::process::id()
    ))
}

fn assert_stdout(output: std::process::Output, expected: &str) {
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), expected);
}

fn assert_plan_output(output: std::process::Output) {
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("plan output is json");
    assert_eq!(json["profile_name"], "example");
    assert_eq!(json["dry_run_required"], true);
    assert_eq!(json["jobs"][0]["source_id"], "policy-memos");
    assert_eq!(json["jobs"][0]["write_source_paths"], false);
}

fn assert_scan_output(output: std::process::Output) {
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("scan output is json");
    assert_eq!(json["profile_name"], "example");
    assert_eq!(json["dry_run_required"], true);
    assert_eq!(json["jobs"][0]["source_id"], "policy-memos");
    assert_eq!(json["jobs"][0]["write_source_paths"], false);
    assert!(json["jobs"][0].get("root").is_none());
}
