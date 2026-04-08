use std::path::Path;
use std::process::Command;

#[test]
fn ui_tests() {
    let ui_dir = Path::new("ui");

    // Auto-install node_modules if missing.
    if !ui_dir.join("node_modules").exists() {
        let install = Command::new("npm")
            .args(["install"])
            .current_dir(ui_dir)
            .status()
            .expect("Failed to run npm install");

        assert!(install.success(), "npm install failed");
    }

    let status = Command::new("npm")
        .args(["run", "test"])
        .current_dir(ui_dir)
        .status()
        .expect("Failed to run npm test");

    assert!(status.success(), "UI tests failed");
}
