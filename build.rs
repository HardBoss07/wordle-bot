use anyhow::{Context, Result};
use glob::glob;
use std::env;
use std::fs;
use std::path::PathBuf;

/// Robust build script for wordle-bot.
/// This script dynamically locates non-Rust assets and ensures they are
/// available at both build time (OUT_DIR) and runtime (next to the executable).
fn main() -> Result<()> {
    // 1. Locate the output directory provided by Cargo
    let out_dir =
        PathBuf::from(env::var("OUT_DIR").context("OUT_DIR environment variable not set")?);

    // 2. Identify the profile directory (e.g., target/debug or target/release)
    // OUT_DIR is typically: target/<profile>/build/<pkg-name>-<hash>/out
    // We go up three levels to reach target/<profile>
    let profile_dir = out_dir.ancestors().nth(3).map(|p| p.to_path_buf());

    // 3. Define patterns for non-Rust assets to be included in the build
    let asset_patterns = ["*.json", "*.txt", "LICENSE*", "README.md", "docs/*.md"];

    println!("cargo:rerun-if-changed=build.rs");

    for pattern in &asset_patterns {
        for entry in glob(pattern).context("Failed to process glob pattern")? {
            let path = entry.context("Failed to read glob entry")?;

            if path.is_file() {
                let file_name = path.file_name().context("Failed to extract file name")?;

                // Copy to OUT_DIR (standard practice for generated/processed files)
                let out_dest = out_dir.join(file_name);
                fs::copy(&path, &out_dest)
                    .with_context(|| format!("Failed to copy {} to OUT_DIR", path.display()))?;

                // Copy to Profile Directory (makes assets available next to the binary for local execution/testing)
                if let Some(ref dir) = profile_dir {
                    let profile_dest = dir.join(file_name);

                    // Only copy if the destination is different from the source
                    if let (Ok(s), Ok(d)) = (path.canonicalize(), profile_dest.canonicalize()) {
                        if s == d {
                            continue;
                        }
                    }

                    // We use a best-effort copy for the profile directory
                    let _ = fs::copy(&path, &profile_dest);
                }

                // 4. Emit rerun-if-changed instruction for Cargo's dependency tracking
                println!("cargo:rerun-if-changed={}", path.display());
            }
        }
    }

    Ok(())
}
