mod analysis;
mod filter;
mod game;
mod play;
mod ranking;
mod solver;

use analysis::LetterStats;
use anyhow::Result;
use play::Play;
use solver::Solver;
use std::fs;

// TODO: Add simulate mode which plays game to caculate average number of guesses

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: wordle-bot <solve|play|analyze|rank>");
        std::process::exit(1);
    }

    match args[1].as_str() {
        "analyze" => analyze()?,
        "rank" => rank()?,
        "solve" => solve()?,
        "play" => play()?,
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            std::process::exit(1);
        }
    }

    Ok(())
}

fn play() -> Result<()> {
    let mut play = Play::new();
    play.run()?;

    Ok(())
}

fn solve() -> Result<()> {
    let solver = Solver::new();
    solver?.run()?;

    Ok(())
}

fn analyze() -> Result<()> {
    let content = fs::read_to_string("wordlist.txt")?;
    let words: Vec<&str> = content.lines().collect();
    let stats = LetterStats::from_words(&words);

    let mut json = serde_json::to_string_pretty(&stats)?;

    // This regex joins lines between '[' and ']'
    let re = regex::Regex::new(r"\[\s*((?:\d+,\s*)*\d+)\s*\]").unwrap();
    json = re
        .replace_all(&json, |caps: &regex::Captures| {
            let inner = caps[1].split_whitespace().collect::<Vec<_>>().join(" ");
            format!("[{}]", inner.replace(", ", ", "))
        })
        .to_string();

    fs::write("letter_stats.json", json)?;
    println!("Saved letter stats to letter_stats.json");

    Ok(())
}

fn rank() -> Result<()> {
    use ranking::rank_words;
    let content = fs::read_to_string("wordlist.txt")?;
    let words: Vec<&str> = content.lines().collect();

    let stats_json = fs::read_to_string("letter_stats.json")?;
    let results = rank_words(&words, &stats_json)?;

    println!("Top 10 words by letter position frequency:");
    for (word, score) in results.iter().take(10) {
        println!("{word:<10} {score:.5}");
    }

    Ok(())
}
