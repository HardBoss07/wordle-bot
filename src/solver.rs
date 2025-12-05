use crate::filter::Filter;
use crate::game::GameData;
use crate::ranking::weighted_rank;
use anyhow::anyhow;
use anyhow::Result;
use std::fs;
use std::io::{self, Write};

pub struct Solver {
    game: GameData,
    current_words: Vec<String>,
    all_words: Vec<String>,
}

impl Solver {
    pub fn new() -> Result<Self> {
        let content = fs::read_to_string("wordlist.txt")
            .map_err(|e| anyhow!("Failed to read wordlist.txt: {}", e))?;

        let words: Vec<String> = content
            .lines()
            .map(|w| w.trim().to_lowercase())
            .filter(|w| w.len() == 5)
            .collect();

        if words.is_empty() {
            return Err(anyhow!("wordlist.txt is empty or invalid"));
        }

        Ok(Self {
            game: GameData::new(),
            current_words: words.clone(), // filtered, may shrink during filtering
            all_words: words,             // full list stays available for checking
        })
    }

    pub fn run(&mut self) -> Result<()> {
        use crate::ranking::rank_words;
        let stats_json = fs::read_to_string("letter_stats.json")?;
        let word_refs: Vec<&str> = self.current_words.iter().map(|s| s.as_str()).collect();
        let start_results = rank_words(&word_refs, &stats_json)?;

        println!("Top 10 words by letter position frequency:");
        for (word, score) in start_results.iter().take(10) {
            println!("{word:<10} {score:.5}");
        }

        loop {
            // Step 1: enter word
            print!("Enter your 5-letter guess (or 'exit'): ");
            io::stdout().flush()?;
            let mut word = String::new();
            io::stdin().read_line(&mut word)?;
            let word = word.trim().to_lowercase();

            if word == "exit" {
                println!("Exiting solver.");
                break;
            }

            if word == "-r" {
                self.reset();
                println!("Solver has been reset.\n");
                continue; // skip the rest of the loop entirely
            }

            if word.len() != 5 {
                println!("Please enter a 5-letter word.\n");
                continue;
            }

            // Check if guess exists in wordlist (uses preloaded all_words)
            if !self.all_words.contains(&word) {
                println!("'{}' is not in the wordlist.\n", word);
                continue;
            }

            // Step 2: enter pattern
            print!("Enter pattern (w = wrong, m = misplaced, c = correct): ");
            io::stdout().flush()?;
            let mut pattern = String::new();
            io::stdin().read_line(&mut pattern)?;
            let pattern = pattern.trim().to_lowercase();

            if pattern.len() != 5 || !pattern.chars().all(|c| "wmc".contains(c)) {
                println!("Invalid pattern. Use only w, m, c.\n");
                continue;
            }

            // Update game
            self.game.add_line(&word, &pattern);

            // Show summary
            self.game.print_summary();

            // Break if Game Won
            if self.is_game_won() {
                let word = self.get_solved_word().unwrap();
                println!(
                    "Congratulations! You've solved the puzzle! The word is '{}'.",
                    word
                );
                break;
            }

            // Update suggestions
            let stats_json = fs::read_to_string("letter_stats.json")
                .map_err(|e| anyhow!("Failed to read letter_stats.json: {}", e))?;
            self.rank_words(&stats_json)?;
        }

        Ok(())
    }

    pub fn rank_words(&mut self, stats_json: &str) -> Result<()> {
        // Read solver_config.json as Vec of tuples
        let config_content = fs::read_to_string("solver_config.json")
            .map_err(|e| anyhow!("Failed to read solver_config.json: {}", e))?;

        let weights: Vec<(f64, f64, f64)> = serde_json::from_str(&config_content)
            .map_err(|e| anyhow!("Failed to parse solver_config.json: {}", e))?;

        // Select weight set based on number of guesses
        let attempt = self.game.lines.len().min(weights.len() - 1);
        let weight_tuple = weights[attempt];

        // Update wordlist (filtered)
        self.current_words = self.update_wordlist();

        // Prepare for ranking
        let word_refs: Vec<&str> = self.current_words.iter().map(|s| s.as_str()).collect();
        let ranked_words = weighted_rank(&word_refs, stats_json, weight_tuple)?;

        println!("Top suggested words:");
        for (word, score) in ranked_words.iter().take(10) {
            println!("{word:<10} {score:.5}");
        }
        println!("Total Words Left: {}\n", self.current_words.len());

        Ok(())
    }

    pub fn update_wordlist(&self) -> Vec<String> {
        let filter = Filter::new(&self.game, &self.current_words);
        filter.filter_words()
    }

    fn is_game_won(&self) -> bool {
        self.game.correct_positions.iter().all(|&pos| pos.is_some())
    }

    fn get_solved_word(&self) -> Option<String> {
        if self.is_game_won() {
            Some(
                self.game
                    .correct_positions
                    .iter()
                    .map(|&c| c.unwrap())
                    .collect(),
            )
        } else {
            None
        }
    }

    fn reset(&mut self) {
        self.game.reset();
        self.current_words = self.all_words.clone();
    }
}
