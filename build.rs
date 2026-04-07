use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=ui/src");
    println!("cargo:rerun-if-changed=ui/static");
    println!("cargo:rerun-if-changed=ui/svelte.config.js");
    println!("cargo:rerun-if-changed=ui/package.json");

    if std::env::var("SKIP_UI_BUILD").is_ok() {
        let build_dir = std::path::Path::new("ui/build");
        if !build_dir.exists() {
            std::fs::create_dir_all(build_dir).unwrap();
            std::fs::write(
                build_dir.join("index.html"),
                "<h1>Binary was compiled with SKIP_UI_BUILD</h1>",
            )
            .unwrap();
        }
        return;
    }

    let ui_dir = std::path::Path::new("ui");
    let build_dir = ui_dir.join("build");

    // Clean previous build (removes stub from SKIP_UI_BUILD if present)
    if build_dir.exists() {
        std::fs::remove_dir_all(&build_dir).unwrap();
    }

    // Install dependencies if node_modules is missing
    if !ui_dir.join("node_modules").exists() {
        let status = Command::new("npm")
            .arg("install")
            .current_dir(ui_dir)
            .status()
            .expect("failed to run npm install");
        assert!(status.success(), "npm install failed");
    }

    let status = Command::new("npm")
        .args(["run", "build"])
        .current_dir(ui_dir)
        .status()
        .expect("failed to run npm build");
    assert!(status.success(), "SvelteKit build failed");
}
