use crate::game::{CellData, GameData, LineData};
use anyhow::Result;
use rand::prelude::IndexedRandom;
use rand::rngs::ThreadRng;
use std::collections::HashMap;
use std::fs;
use std::io;

pub struct Play {
    word: String,
    num_guesses: usize,
    game_data: GameData,
    wordlist: Vec<String>,
    result: GameResult,
}

#[derive(PartialEq, Debug)]
pub enum GameResult {
    Win,
    Lose,
    Ongoing,
}

impl Play {
    pub fn new() -> Self {
        let content = fs::read_to_string("wordlist.txt").expect("Failed to read wordlist.txt");

        let words: Vec<String> = content
            .lines()
            .map(|w| w.trim().to_lowercase())
            .filter(|w| w.len() == 5)
            .collect();

        let allowed_words = &words[10657..];

        let mut rng = ThreadRng::default();
        let random_word = allowed_words
            .choose(&mut rng)
            .expect("No words available")
            .clone();

        Self {
            word: random_word,
            num_guesses: 6,
            game_data: GameData::new(),
            wordlist: words,
            result: GameResult::Ongoing,
        }
    }

    pub fn evaluate_word(&mut self, guessed_word: &str) -> LineData {
        let guessed_chars: Vec<char> = guessed_word.chars().collect();
        let target_chars: Vec<char> = self.word.chars().collect();

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

        // Count remaining letters in target for handling duplicates
        let mut remaining_counts: HashMap<char, usize> = HashMap::new();
        for &c in &target_chars {
            *remaining_counts.entry(c).or_insert(0) += 1;
        }

        // First pass: correct positions
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

        // Second pass: misplaced or wrong
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

    pub fn run(&mut self) -> Result<()> {
        let mut attempts = 0;

        while attempts < self.num_guesses && self.result == GameResult::Ongoing {
            self.print_summary();
            self.add_line();

            if let Some(last_line) = self.game_data.lines.last() {
                if last_line.word == self.word {
                    self.result = GameResult::Win;
                    break;
                }
            }

            if attempts == self.num_guesses - 1 {
                self.result = GameResult::Lose;
                break;
            }

            attempts += 1;
        }

        self.print_summary();

        match self.result {
            GameResult::Win => {
                println!("Congratulations! You've guessed the word: {}", self.word);
            }
            GameResult::Lose => {
                println!("Game Over! The correct word was: {}", self.word);
            }
            GameResult::Ongoing => {}
        }

        Ok(())
    }

    fn add_line(&mut self) {
        loop {
            println!("Enter your guess:");

            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_err() {
                println!("Failed to read input. Try again.");
                continue;
            }

            // Remove all whitespace and lowercase
            let cleaned: String = input
                .chars()
                .filter(|c| !c.is_whitespace())
                .collect::<String>()
                .to_lowercase();

            // Take only the first 5 letters
            let word: String = cleaned.chars().take(5).collect();

            if word.len() < 5 {
                println!("Word must be 5 letters long!");
                continue;
            }

            // Check against the wordlist in the struct
            if !self.wordlist.contains(&word) {
                println!("Word not in the allowed word list!");
                continue;
            }

            // Valid word, process it
            let line = self.evaluate_word(&word);
            let pattern = self.get_pattern(&line);

            self.game_data.add_line(&word, &pattern);
            break;
        }
    }

    fn print_summary(&self) {
        println!("\n=== Current Game State ===");
        println!("Nr.  Word");

        for (number, line) in self.game_data.lines.iter().enumerate() {
            print!("{}.   ", number + 1);

            for cell in &line.cells {
                let letter = cell.letter.to_ascii_uppercase();

                let color = match cell.state {
                    'c' => "\x1b[42m\x1b[30m",  // green background, black text
                    'm' => "\x1b[43m\x1b[30m",  // yellow background, black text
                    'w' => "\x1b[100m\x1b[37m", // gray background, white text
                    _ => "\x1b[0m",
                };

                print!("{} {} \x1b[0m", color, letter);
            }
            println!();
        }
        println!("{}", self.generate_keyboard());
        println!("==========================\n");
    }

     fn generate_keyboard(&self) -> String {
        let row1 = "QWERTYUIOP".chars().collect::<Vec<_>>();
        let row2 = "ASDFGHJKL".chars().collect::<Vec<_>>();
        let row3 = "ZXCVBNM".chars().collect::<Vec<_>>();

        let mut letter_states: HashMap<char, char> = HashMap::new();

        for line in &self.game_data.lines {
            for cell in &line.cells {
                let upper = cell.letter.to_ascii_uppercase();
                let state = letter_states.entry(upper).or_insert(cell.state);

                if *state != 'c' && cell.state == 'c' {
                    *state = 'c';
                } else if *state == 'w' && cell.state == 'm' {
                    *state = 'm';
                }
            }
        }

        let colorize = |c: char| -> &'static str {
            match letter_states.get(&c) {
                Some('c') => "\x1b[42m\x1b[30m",  // green bg, black text
                Some('m') => "\x1b[43m\x1b[30m",  // yellow bg, black text
                Some('w') => "\x1b[100m\x1b[37m", // gray bg, white text
                _ => "\x1b[0m",                   // default
            }
        };
        
        let mut output = String::new();

        for row in &[row1, row2, row3] {
            let indent = match row.len() {
                10 => 0,
                9 => 1,
                7 => 2,
                _ => 0,
            };

            output.push_str(&" ".repeat(indent));

            for &letter in row {
                output.push_str(&format!("{} {} \x1b[0m", colorize(letter), letter));
            }
            output.push('\n');

        };

        output
    }

    fn get_pattern(&self, line: &LineData) -> String {
        line.cells.iter().map(|cell| cell.state).collect()
    }
}
