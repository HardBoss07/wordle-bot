use std::fs;
use std::path::PathBuf;
use std::env;
use anyhow::{Ok, Result, anyhow};

fn exe_dir() -> Result<PathBuf> {
    let exe_path = env::current_exe()
        .map_err(|e| anyhow!("Failed to get executable path: {}", e))?;
    let exe_dir = exe_path.parent()
        .ok_or_else(|| anyhow!("Executable has no parent directory"))?;
    Ok(exe_dir.to_path_buf())
}

pub fn read_wordlist() -> Result<String> {
    let path = exe_dir()?.join("wordlist.txt");
    fs::read_to_string(&path)
        .map_err(|e| anyhow!("Failed to read {}: {}", path.display(), e))
}

pub fn read_letter_stats() -> Result<String> {
    let path = exe_dir()?.join("letter_stats.json");
    fs::read_to_string(&path)
        .map_err(|e| anyhow!("Failed to read {}: {}", path.display(), e))
}

pub fn read_solver_config() -> Result<Vec<(f64, f64, f64)>> {
    let path = exe_dir()?.join("solver_config.json");
    let content = fs::read_to_string(&path)
        .map_err(|e| anyhow!("Failed to read {}: {}", path.display(), e))?;
    let weights: Vec<(f64, f64, f64)> = serde_json::from_str(&content)
        .map_err(|e| anyhow!("Failed to parse {}: {}", path.display(), e))?;
    Ok(weights)    
}