use crate::analysis::LetterStats;
use crate::filter::Filter;
use crate::game::GameData;
use crate::ranking::{weighted_rank, weighted_rank_stats};
use crate::trap;
use crate::util;
use anyhow::anyhow;
use anyhow::Result;
use std::io::{self, Write};

pub struct Solver {
    pub game: GameData,
    pub current_words: Vec<String>,
    pub all_words: Vec<String>,
}

impl Solver {
    pub fn new() -> Result<Self> {
        let content =
            util::read_wordlist().map_err(|e| anyhow!("Failed to read wordlist.txt: {}", e))?;

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
            current_words: words.clone(),
            all_words: words,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        self.print_initial_suggestions()?;

        loop {
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
                self.reset()?;
                println!("Solver has been reset.\n");
                continue;
            }

            if word.len() != 5 {
                println!("Please enter a 5-letter word.\n");
                continue;
            }

            if !self.all_words.contains(&word) {
                println!("'{}' is not in the wordlist.\n", word);
                continue;
            }

            print!("Enter pattern (w = wrong, m = misplaced, c = correct): ");
            io::stdout().flush()?;
            let mut pattern = String::new();
            io::stdin().read_line(&mut pattern)?;
            let pattern = pattern.trim().to_lowercase();

            if pattern.len() != 5 || !pattern.chars().all(|c| "wmc".contains(c)) {
                println!("Invalid pattern. Use only w, m, c.\n");
                continue;
            }

            self.game.add_line(&word, &pattern);
            self.game.print_summary();

            if self.is_game_won() {
                let word = self.get_solved_word().unwrap();
                println!(
                    "Congratulations! You've solved the puzzle! The word is '{}'.",
                    word
                );
                break;
            }

            let stats_json = util::read_letter_stats()?;
            self.rank_words(&stats_json, true)?;
        }

        Ok(())
    }

    pub fn evaluate_pattern(guessed_word: &str, target_word: &str) -> u8 {
        let guess = guessed_word.as_bytes();
        let target = target_word.as_bytes();

        let mut states = [0u8; 5]; // 0 = 'w', 1 = 'm', 2 = 'c'
        let mut remaining = [0u8; 26];

        for &b in target {
            remaining[(b - b'a') as usize] += 1;
        }

        for i in 0..5 {
            if guess[i] == target[i] {
                states[i] = 2;
                remaining[(guess[i] - b'a') as usize] -= 1;
            }
        }

        for i in 0..5 {
            if states[i] == 0 {
                let idx = (guess[i] - b'a') as usize;
                if remaining[idx] > 0 {
                    states[i] = 1;
                    remaining[idx] -= 1;
                }
            }
        }

        states[0] + states[1] * 3 + states[2] * 9 + states[3] * 27 + states[4] * 81
    }

    pub fn get_top_suggestion_silent_stats(
        &self,
        stats: &LetterStats,
        weights: Option<(f64, f64, f64)>,
    ) -> Result<String> {
        let word_refs: Vec<&str> = self.current_words.iter().map(|s| s.as_str()).collect();

        if let Some(trap) = trap::detect_trap(&word_refs) {
            if let Some((word, _)) = trap::find_best_elimination(&self.all_words, &trap) {
                return Ok(word);
            }
        }

        let default_weights = (0.2, 0.1, 0.7);
        let weight_tuple = weights.unwrap_or(default_weights);

        let ranked_words = weighted_rank_stats(&word_refs, stats, weight_tuple);

        ranked_words
            .into_iter()
            .next()
            .map(|(word, _)| word)
            .ok_or_else(|| anyhow!("No suggested words remaining"))
    }

    pub fn rank_words(&mut self, stats_json: &str, print_output: bool) -> Result<()> {
        let weights = util::read_solver_config()?;
        let attempt = self.game.lines.len().min(weights.len() - 1);
        let weight_tuple = weights[attempt];

        self.current_words = self.update_wordlist();

        if let Some(trap) = trap::detect_trap(&self.current_words) {
            if let Some((word, score)) = trap::find_best_elimination(&self.all_words, &trap) {
                if print_output {
                    println!("!!! TRAP DETECTED !!!");
                    println!("Varying positions: {:?}", trap.varying_positions);
                    println!("Distinguishing letters: {:?}", trap.distinguishing_letters);
                    println!(
                        "ELIMINATION MODE: Recommended guess is '{}' (score: {})",
                        word, score
                    );
                    println!("Remaining valid words: {:?}\n", self.current_words);
                }
            }
        }

        let word_refs: Vec<&str> = self.current_words.iter().map(|s| s.as_str()).collect();
        let ranked_words = weighted_rank(&word_refs, stats_json, weight_tuple)?;

        if print_output {
            println!("Top suggested words (Normal Mode):");
            for (word, score) in ranked_words.iter().take(10) {
                println!("{word:<10} {score:.5}");
            }
            println!("Total Words Left: {}\n", self.current_words.len());
        }

        Ok(())
    }

    pub fn update_wordlist(&self) -> Vec<String> {
        let refs: Vec<&str> = self.current_words.iter().map(|s| s.as_str()).collect();
        let filter = Filter::new(&self.game, &refs);
        filter
            .filter_words()
            .into_iter()
            .map(|s| s.to_string())
            .collect()
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

    fn reset(&mut self) -> Result<()> {
        self.game.reset();
        self.current_words = self.all_words.clone();
        self.print_initial_suggestions()?;
        Ok(())
    }

    fn print_initial_suggestions(&self) -> Result<()> {
        use crate::ranking::rank_words;

        let stats_json = util::read_letter_stats()?;
        let word_refs: Vec<&str> = self.current_words.iter().map(|s| s.as_str()).collect();
        let start_results = rank_words(&word_refs, &stats_json)?;

        println!("Top 10 words by letter position frequency:");
        for (word, score) in start_results.iter().take(10) {
            println!("{word:<10} {score:.5}");
        }
        println!();

        Ok(())
    }
}
