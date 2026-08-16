use std::path::PathBuf;

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("installer sits inside the repo")
        .to_path_buf();

    let helper = root
        .join("helper")
        .join("target")
        .join("release")
        .join("duckify-helper.exe");

    if !helper.exists() {
        panic!(
            "helper not built. Run `cargo build --release` in helper/ first.\nExpected: {}",
            helper.display()
        );
    }

    let extension = root.join("extension").join("duckify.js");
    if !extension.exists() {
        panic!("extension not found at {}", extension.display());
    }

    println!("cargo:rustc-env=DUCKIFY_HELPER={}", helper.display());
    println!("cargo:rustc-env=DUCKIFY_EXTENSION={}", extension.display());
    println!("cargo:rerun-if-changed={}", helper.display());
    println!("cargo:rerun-if-changed={}", extension.display());
}
