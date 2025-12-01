use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellData {
    pub letter: char,
    pub state: char,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineData {
    pub word: String,
    pub cells: [CellData; 5],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameData {
    pub lines: Vec<LineData>,
    pub contains_not: HashSet<char>,
    pub correct_positions: [Option<char>; 5],
    pub misplaced_letters: HashMap<usize, HashSet<char>>,
    pub must_contain: HashSet<char>,
}

impl GameData {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            contains_not: HashSet::new(),
            correct_positions: [None, None, None, None, None],
            misplaced_letters: HashMap::new(),
            must_contain: HashSet::new(),
        }
    }

    pub fn add_line(&mut self, word: &str, pattern: &str) {
        let mut cells = Vec::new();

        for (i, (ch, state)) in word.chars().zip(pattern.chars()).enumerate() {
            cells.push(CellData { letter: ch, state });

            match state {
                'c' => {
                    self.correct_positions[i] = Some(ch);
                    self.must_contain.insert(ch);
                }
                'm' => {
                    self.misplaced_letters.entry(i).or_default().insert(ch);
                    self.must_contain.insert(ch);
                }
                'w' => {
                    if !self.must_contain.contains(&ch) {
                        self.contains_not.insert(ch);
                    }
                }
                _ => {}
            }
        }

        let cells: [CellData; 5] = cells.try_into().expect("must be 5 letters");
        self.lines.push(LineData {
            word: word.to_string(),
            cells,
        });
    }

    pub fn print_summary(&self) {
        println!("\n=== Current Game State ===");
        println!("Guesses: {}", self.lines.len());

        println!("Not in word: {}", fmt_hashset(&self.contains_not));
        println!(
            "Correct positions: {}",
            fmt_correct_positions(&self.correct_positions)
        );
        println!(
            "Misplaced letters: {}",
            fmt_misplaced_letters(&self.misplaced_letters)
        );
        println!("Must contain: {}", fmt_hashset(&self.must_contain));

        println!("==========================\n");
    }

    pub fn reset(&mut self) {
        self.lines.clear();
        self.contains_not.clear();
        self.correct_positions = [None, None, None, None, None];
        self.misplaced_letters.clear();
        self.must_contain.clear();
    }
}

// Formatting helper functions

fn fmt_hashset(set: &HashSet<char>) -> String {
    set.iter()
        .map(|&c| c.to_uppercase().to_string())
        .collect::<Vec<String>>()
        .join(" ")
}

fn fmt_correct_positions(positions: &[Option<char>; 5]) -> String {
    positions
        .iter()
        .map(|&opt| {
            opt.map(|c| c.to_uppercase().to_string())
                .unwrap_or_else(|| "_".to_string())
        })
        .collect::<Vec<String>>()
        .join(" ")
}

fn fmt_misplaced_letters(map: &HashMap<usize, HashSet<char>>) -> String {
    let mut parts = Vec::new();

    for i in 0..5 {
        if let Some(set) = map.get(&i) {
            if !set.is_empty() {
                let letters: String = set
                    .iter()
                    .map(|c| c.to_uppercase().to_string())
                    .collect::<Vec<String>>()
                    .join("");

                parts.push(format!("{}: {}", i, letters));
            }
        }
    }

    parts.join(", ")
}
