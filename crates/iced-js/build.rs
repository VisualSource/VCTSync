use std::path::Path;
use std::process::Command;

fn pnpm(manifest_dir: &Path, args: &[&str]) -> std::io::Result<std::process::Output> {
    let mut cmd = if cfg!(windows) {
        let mut c = Command::new("cmd");
        c.args(["/C", "pnpm"]);
        c
    } else {
        Command::new("pnpm")
    };
    cmd.args(args).current_dir(manifest_dir).output()
}

fn run_pnpm(manifest_dir: &Path, args: &[&str]) {
    let out = pnpm(manifest_dir, args)
        .unwrap_or_else(|e| panic!("failed to spawn `pnpm {}`: {e}", args.join(" ")));
    if !out.status.success() {
        panic!(
            "`pnpm {}` failed in {}:\n{}\n{}",
            args.join(" "),
            manifest_dir.display(),
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr),
        );
    }
}

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest_dir = Path::new(&manifest_dir);

    println!("cargo:rerun-if-changed=js/iced-dom.ts");
    println!("cargo:rerun-if-changed=js/vendor");
    println!("cargo:rerun-if-changed=package.json");
    println!("cargo:rerun-if-changed=pnpm-lock.yaml");

    let dist_ready = ["iced-dom.js", "react.js", "jsx-runtime.js"]
        .iter()
        .all(|f| manifest_dir.join("js/dist").join(f).exists());

    // `embed!` in src/runtime.rs reads js/dist at macro expansion, so the
    // bundles must exist before rustc runs.
    if pnpm(manifest_dir, &["--version"]).map_or(true, |o| !o.status.success()) {
        if dist_ready {
            println!("cargo:warning=pnpm not found; using existing js/dist bundles");
            return;
        }
        panic!(
            "pnpm not found and js/dist bundles are missing. \
             Install pnpm, or run `pnpm install && pnpm run build:dev` in {}",
            manifest_dir.display()
        );
    }

    if !manifest_dir.join("node_modules").exists() {
        run_pnpm(manifest_dir, &["install"]);
    }

    let script = if std::env::var("PROFILE").as_deref() == Ok("release") {
        "build:prod"
    } else {
        "build:dev"
    };
    run_pnpm(manifest_dir, &["run", script]);
}
