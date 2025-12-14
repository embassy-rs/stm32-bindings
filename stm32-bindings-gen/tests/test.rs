use assert_cmd::Command;

use predicates::prelude::*;

#[test]
fn displays_help_message() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("stm32-bindings-gen"));
    cmd.arg("--help");

    cmd.assert()
        .success()
        .stderr(predicate::str::contains("Usage: stm32-bindings-gen"))
        .stdout(predicate::str::is_empty());
}
