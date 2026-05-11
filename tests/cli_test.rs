use std::process::Command;

fn kmlcli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_kmlcli"))
}

#[test]
fn test_info_subcommand_outputs_json() {
    let output = kmlcli()
        .args(["info", "tests/fixtures/multi.kml"])
        .output()
        .expect("failed to run kmlcli");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("invalid JSON output");
    assert_eq!(json["name"], "Multi Test");
    assert!(json["placemark_count"].as_u64().unwrap() >= 3);
}

#[test]
fn test_list_subcommand_outputs_json_array() {
    let output = kmlcli()
        .args(["list", "tests/fixtures/simple.kml"])
        .output()
        .expect("failed to run kmlcli");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("invalid JSON output");
    assert!(json.is_array());
    assert!(!json.as_array().unwrap().is_empty());
}

#[test]
fn test_tree_subcommand_outputs_text() {
    let output = kmlcli()
        .args(["tree", "tests/fixtures/multi.kml"])
        .output()
        .expect("failed to run kmlcli");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Routes"));
    assert!(stdout.contains("Zone A"));
}

#[test]
fn test_no_args_shows_error() {
    let output = kmlcli().output().expect("failed to run kmlcli");
    assert!(!output.status.success());
}

#[test]
fn test_nonexistent_file_shows_error() {
    let output = kmlcli()
        .args(["info", "nonexistent.kml"])
        .output()
        .expect("failed to run kmlcli");
    assert!(!output.status.success());
}
