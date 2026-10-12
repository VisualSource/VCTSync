use std::process::Command;

fn main() {
    for path in ["view", "package.json", "pnpm-lock.yaml", "vite.config.ts", "tsconfig.json"] {
        println!("cargo:rerun-if-changed={path}");
    }

    let pnpm = if cfg!(windows) { "pnpm.cmd" } else { "pnpm" };
    let status = Command::new(pnpm)
        .args(["exec", "vite", "build"])
        .status()
        .expect("failed to run pnpm; is it installed and on PATH?");
    assert!(status.success(), "frontend build failed: {status}");
}
