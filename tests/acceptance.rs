use assert_cmd::Command;
use serde_json::Value;
use serde_json_path::JsonPath;
use std::env::current_dir;

// Acceptance tests for zizmor, on top of Json output
// For now we don't cover tests that depends on Github API under the hood

fn zizmor() -> Command {
    let mut cmd = Command::cargo_bin("zizmor").expect("Cannot create executable command");
    // All tests are currently offline, and we always need JSON output.
    cmd.args(["--offline", "--format", "json"]);
    cmd
}

fn workflow_under_test(name: &str) -> String {
    let current_dir = current_dir().expect("Cannot figure out current directory");

    let file_path = current_dir.join("tests").join("test-data").join(name);

    file_path
        .to_str()
        .expect("Cannot create string reference for file path")
        .to_string()
}

fn assert_value_match(json: &Value, path_pattern: &str, value: &str) {
    let json_path = JsonPath::parse(path_pattern).expect("Cannot evaluate json path");
    let queried = json_path
        .query(json)
        .exactly_one()
        .expect("Cannot query json path");

    // Don't bother about surrounding formatting
    assert!(queried.to_string().contains(value));
}

#[test]
fn audit_artipacked() -> anyhow::Result<()> {
    let auditable = workflow_under_test("artipacked.yml");
    let cli_args = [&auditable];

    let execution = zizmor().args(cli_args).unwrap();

    assert!(execution.status.success());

    let findings = serde_json::from_slice(&execution.stdout)?;

    assert_value_match(&findings, "$[0].determinations.confidence", "Low");
    assert_value_match(
        &findings,
        "$[0].locations[0].concrete.feature",
        "uses: actions/checkout@v4",
    );

    Ok(())
}

#[test]
fn audit_excessive_permission() -> anyhow::Result<()> {
    let auditable = workflow_under_test("excessive-permissions.yml");
    let cli_args = [&auditable];

    let execution = zizmor().args(cli_args).unwrap();

    assert!(execution.status.success());

    let findings = serde_json::from_slice(&execution.stdout)?;

    assert_value_match(&findings, "$[0].determinations.confidence", "High");
    assert_value_match(
        &findings,
        "$[0].locations[0].concrete.feature",
        "permissions: write-all",
    );

    Ok(())
}

#[test]
fn audit_hardcoded_credentials() -> anyhow::Result<()> {
    let auditable = workflow_under_test("hardcoded-credentials.yml");
    let cli_args = [&auditable];

    let execution = zizmor().args(cli_args).unwrap();

    assert!(execution.status.success());

    let findings = serde_json::from_slice(&execution.stdout)?;

    assert_value_match(&findings, "$[0].determinations.confidence", "High");
    assert_value_match(
        &findings,
        "$[0].locations[0].concrete.feature",
        "password: hackme",
    );

    Ok(())
}

#[test]
fn audit_template_injection() -> anyhow::Result<()> {
    let auditable = workflow_under_test("template-injection.yml");
    let cli_args = [&auditable];

    let execution = zizmor().args(cli_args).unwrap();

    assert!(execution.status.success());

    let findings = serde_json::from_slice(&execution.stdout)?;

    assert_value_match(&findings, "$[0].determinations.confidence", "High");
    assert_value_match(
        &findings,
        "$[0].locations[0].concrete.feature",
        "${{ github.event.issue.title }}",
    );

    Ok(())
}

#[test]
fn audit_use_trusted_publishing() -> anyhow::Result<()> {
    let auditable = workflow_under_test("use-trusted-publishing.yml");
    let cli_args = [&auditable];

    let execution = zizmor().args(cli_args).unwrap();

    assert!(execution.status.success());

    let findings = serde_json::from_slice(&execution.stdout)?;

    assert_value_match(&findings, "$[0].determinations.confidence", "High");
    assert_value_match(
        &findings,
        "$[0].locations[0].concrete.feature",
        "uses: pypa/gh-action-pypi-publish@release/v1",
    );

    Ok(())
}

#[test]
fn audit_self_hosted() -> anyhow::Result<()> {
    let auditable = workflow_under_test("self-hosted.yml");

    // Note : self-hosted audit is pedantic
    let cli_args = ["--pedantic", &auditable];

    let execution = zizmor().args(cli_args).unwrap();

    assert!(execution.status.success());

    let findings = serde_json::from_slice(&execution.stdout)?;

    assert_value_match(&findings, "$[0].determinations.confidence", "High");
    assert_value_match(
        &findings,
        "$[0].locations[0].concrete.feature",
        "runs-on: [self-hosted, my-ubuntu-box]",
    );

    Ok(())
}
