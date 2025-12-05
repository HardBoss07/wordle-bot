use crate::filter::Filter;
use crate::game::{CellData, GameData, LineData}; // CellData and LineData for simulation helpers
use crate::ranking::{rank_words, weighted_rank};
use anyhow::anyhow;
use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write}; // Required for word evaluation in simulate

pub struct Solver {
    game: GameData,
    current_words: Vec<String>,
    pub all_words: Vec<String>, // Made public for use in simulation
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
        self.print_initial_suggestions()?;

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
            self.rank_words(&stats_json, true)?;
        }

        Ok(())
    }

    pub fn simulate(
        &self,
        target_word: String,
        stats_json: &str,
        config_json: &str,
    ) -> Result<usize> {
        let mut temp_solver = Solver {
            game: GameData::new(),
            current_words: self.all_words.clone(),
            all_words: self.all_words.clone(),
        };
        let mut guesses = 0;
        let max_guesses = 6;

        let weights: Vec<(f64, f64, f64)> = serde_json::from_str(config_json)
            .map_err(|e| anyhow!("Failed to parse solver_config.json: {}", e))?;

        while guesses < max_guesses {
            let attempt = temp_solver.game.lines.len().min(weights.len() - 1);
            let weight_tuple = weights[attempt];

            let guess_word = if guesses == 0 {
                temp_solver.get_top_suggestion_silent(stats_json, None)?
            } else {
                // Filter first, then use weighted ranking
                temp_solver.current_words = temp_solver.update_wordlist();
                temp_solver.get_top_suggestion_silent(stats_json, Some(weight_tuple))?
            };

            guesses += 1;

            // Win condition check
            if guess_word == target_word {
                return Ok(guesses);
            }

            // Safety check: this should only happen if the filtering failed somehow.
            if guess_word.len() != 5 || !temp_solver.all_words.contains(&guess_word) {
                return Ok(max_guesses + 1); // Indicate failure/loss
            }

            // Evaluate the guess against the target word
            let line = Self::evaluate_word(&guess_word, &target_word);
            let pattern = Self::get_pattern(&line);

            // Update game state
            temp_solver.game.add_line(&guess_word, &pattern);
        }

        Ok(max_guesses + 1) // Lost
    }

    fn evaluate_word(guessed_word: &str, target_word: &str) -> LineData {
        let guessed_chars: Vec<char> = guessed_word.chars().collect();
        let target_chars: Vec<char> = target_word.chars().collect();

        let mut result_cells: [CellData; 5] = [
            CellData {
                letter: ' ',
                state: 'w',
            },
            CellData {
                letter: ' ',
                state: 'w',
            },
            CellData {
                letter: ' ',
                state: 'w',
            },
            CellData {
                letter: ' ',
                state: 'w',
            },
            CellData {
                letter: ' ',
                state: 'w',
            },
        ];

        let mut remaining_counts: HashMap<char, usize> = HashMap::new();
        for &c in &target_chars {
            *remaining_counts.entry(c).or_insert(0) += 1;
        }

        for i in 0..5 {
            let g = guessed_chars[i];
            let t = target_chars[i];

            if g == t {
                result_cells[i] = CellData {
                    letter: g,
                    state: 'c',
                };
                *remaining_counts.get_mut(&g).unwrap() -= 1;
            } else {
                result_cells[i].letter = g;
            }
        }

        for i in 0..5 {
            if result_cells[i].state == 'c' {
                continue;
            }
            let g = guessed_chars[i];
            if let Some(count) = remaining_counts.get_mut(&g) {
                if *count > 0 {
                    result_cells[i].state = 'm';
                    *count -= 1;
                } else {
                    result_cells[i].state = 'w';
                }
            } else {
                result_cells[i].state = 'w';
            }
        }

        LineData {
            word: guessed_word.to_string(),
            cells: result_cells,
        }
    }

    fn get_pattern(line: &LineData) -> String {
        line.cells.iter().map(|cell| cell.state).collect()
    }

    fn get_top_suggestion_silent(
        &self,
        stats_json: &str,
        weights: Option<(f64, f64, f64)>,
    ) -> Result<String> {
        let word_refs: Vec<&str> = self.current_words.iter().map(|s| s.as_str()).collect();

        let ranked_words = if let Some(weight_tuple) = weights {
            weighted_rank(&word_refs, stats_json, weight_tuple)?
        } else {
            rank_words(&word_refs, stats_json)?
        };

        ranked_words
            .into_iter()
            .next()
            .map(|(word, _)| word)
            .ok_or_else(|| anyhow!("No suggested words remaining"))
    }

    pub fn rank_words(&mut self, stats_json: &str, print_output: bool) -> Result<()> {
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

        if print_output {
            println!("Top suggested words:");
            for (word, score) in ranked_words.iter().take(10) {
                println!("{word:<10} {score:.5}");
            }
            println!("Total Words Left: {}\n", self.current_words.len());
        }

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

    fn reset(&mut self) -> Result<()> {
        self.game.reset();
        self.current_words = self.all_words.clone();
        self.print_initial_suggestions()?;
        Ok(())
    }

    fn print_initial_suggestions(&self) -> Result<()> {
        use crate::ranking::rank_words;

        let stats_json = fs::read_to_string("letter_stats.json")?;
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
