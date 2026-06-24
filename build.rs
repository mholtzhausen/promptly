use std::path::Path;

fn main() {
    let frontend = Path::new(env!("CARGO_MANIFEST_DIR")).join("frontend");
    let dist = frontend.join("dist").join("index.html");

    println!(
        "cargo:rerun-if-changed={}",
        frontend.join("package.json").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        frontend.join("package-lock.json").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        frontend.join("index.html").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        frontend.join("vite.config.ts").display()
    );
    println!("cargo:rerun-if-changed={}", frontend.join("src").display());

    if !dist.exists() {
        panic!(
            "frontend/dist/index.html is missing; run make frontend-build before cargo build, or use make build"
        );
    }
}
