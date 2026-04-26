use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // 1. Tell Cargo to rerun this script if any of the asset files change
    let assets = [
        "wordlist.txt",
        "solver_config.json",
        "letter_stats.json",
        "README.md",
        "LICENSE",
    ];
    for asset in &assets {
        println!("cargo:rerun-if-changed={}", asset);
    }

    // 2. Find the project root directory
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let root_path = PathBuf::from(manifest_dir);

    // 3. Identify the target directory
    // OUT_DIR is typically target/<profile>/build/<pkg>/out
    // We want to copy files to target/<profile>/
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let mut target_dir = PathBuf::from(out_dir);

    // Navigate up from OUT_DIR to the profile directory (target/debug or target/release)
    // Structure: target/debug/build/wordle-bot-hash/out -> target/debug
    while target_dir
        .file_name()
        .map(|n| n != "debug" && n != "release")
        .unwrap_or(true)
    {
        if !target_dir.pop() {
            panic!("Could not find target directory from OUT_DIR");
        }
    }

    // 4. Copy each file
    for asset in &assets {
        let src_path = root_path.join(asset);
        let dest_path = target_dir.join(asset);

        if !src_path.exists() {
            panic!(
                "CRITICAL ERROR: Asset file '{}' is missing in the project root ({}).",
                asset,
                src_path.display()
            );
        }

        fs::copy(&src_path, &dest_path).unwrap_or_else(|e| {
            panic!(
                "Failed to copy '{}' to '{}': {}",
                src_path.display(),
                dest_path.display(),
                e
            )
        });
    }
}
