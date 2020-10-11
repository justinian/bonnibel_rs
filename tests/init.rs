use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*;
use std::process::Command; // Run programs // Used for writing assertions

#[test]
fn file_doesnt_exist() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::main_binary()?;
    cmd.arg("-f").arg("test/file/doesnt/exist").arg("init");
    cmd.assert().failure().stderr(predicate::str::contains("No such file or directory"));

    Ok(())
}
