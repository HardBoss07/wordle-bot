use anyhow::{anyhow, Result};
use directories::ProjectDirs;
use std::fs;

const WORDLIST: &str = include_str!("../wordlist.txt");
const LETTER_STATS: &str = include_str!("../letter_stats.json");
const DEFAULT_CONFIG: &str = include_str!("../solver_config.json");

pub fn read_wordlist() -> Result<String> {
    Ok(WORDLIST.to_string())
}

pub fn read_letter_stats() -> Result<String> {
    Ok(LETTER_STATS.to_string())
}

pub fn read_solver_config() -> Result<Vec<(f64, f64, f64)>> {
    let content = if let Some(proj_dirs) = ProjectDirs::from("", "", "wordle-bot") {
        let config_dir = proj_dirs.config_dir();
        let config_path = config_dir.join("solver_config.json");

        if !config_path.exists() {
            if fs::create_dir_all(config_dir).is_ok() {
                let _ = fs::write(&config_path, DEFAULT_CONFIG);
            }
            DEFAULT_CONFIG.to_string()
        } else {
            fs::read_to_string(&config_path).unwrap_or_else(|_| DEFAULT_CONFIG.to_string())
        }
    } else {
        DEFAULT_CONFIG.to_string()
    };

    let weights: Vec<(f64, f64, f64)> = serde_json::from_str(&content)
        .or_else(|_| serde_json::from_str(DEFAULT_CONFIG))
        .map_err(|e| anyhow!("Failed to parse solver config: {}", e))?;

    Ok(weights)
}
