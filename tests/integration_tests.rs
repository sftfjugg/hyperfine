mod common;
use common::hyperfine;

use predicates::prelude::*;

#[test]
fn hyperfine_runs_successfully() {
    hyperfine()
        .arg("--runs=2")
        .arg("echo dummy benchmark")
        .assert()
        .success();
}

#[test]
fn one_run_is_supported() {
    hyperfine()
        .arg("--runs=1")
        .arg("echo dummy benchmark")
        .assert()
        .success();
}

#[test]
fn fails_with_wrong_number_of_command_name_arguments() {
    hyperfine()
        .arg("--command-name=a")
        .arg("--command-name=b")
        .arg("echo a")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Too many --command-name options"));
}

#[test]
fn fails_with_wrong_number_of_prepare_options() {
    hyperfine()
        .arg("--runs=1")
        .arg("--prepare=echo a")
        .arg("--prepare=echo b")
        .arg("echo a")
        .arg("echo b")
        .assert()
        .success();

    hyperfine()
        .arg("--prepare=echo a")
        .arg("--prepare=echo b")
        .arg("echo a")
        .arg("echo b")
        .arg("echo c")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "The '--prepare' option has to be provided",
        ));
}

#[test]
fn fails_with_duplicate_parameter_names() {
    hyperfine()
        .arg("--parameter-list")
        .arg("x")
        .arg("1,2,3")
        .arg("--parameter-list")
        .arg("x")
        .arg("a,b,c")
        .arg("echo test")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Duplicate parameter names: x"));
}

#[test]
fn fails_for_unknown_command() {
    hyperfine()
        .arg("some-nonexisting-program-b5d9574198b7e4b12a71fa4747c0a577")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Command terminated with non-zero exit code",
        ));
}

#[cfg(unix)]
#[test]
fn can_run_failing_commands_with_ignore_failure_option() {
    hyperfine()
        .arg("false")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Command terminated with non-zero exit code",
        ));

    hyperfine()
        .arg("--runs=1")
        .arg("--ignore-failure")
        .arg("false")
        .assert()
        .success();
}
