use crate::game::GameData;
use std::collections::HashSet;

pub struct Filter<'a> {
    game: &'a GameData,
    words: &'a [String],
}

impl<'a> Filter<'a> {
    pub fn new(game: &'a GameData, words: &'a [String]) -> Self {
        Self { game, words }
    }

    pub fn filter_words(&self) -> Vec<String> {
        self.words
            .iter()
            .filter(|word| {
                let chars: Vec<char> = word.chars().collect();

                self.not_strictly_forbidden(&chars)
                    && self.matches_correct_positions(&chars)
                    && self.respects_misplaced_constraints(&chars)
                    && self.contains_required_letters(&chars)
            })
            .cloned()
            .collect()
    }

    fn not_strictly_forbidden(&self, chars: &[char]) -> bool {
        for ch in chars {
            if self.game.contains_not.contains(ch) && !self.game.must_contain.contains(ch) {
                return false;
            }
        }
        true
    }

    fn matches_correct_positions(&self, chars: &[char]) -> bool {
        for (i, maybe_correct) in self.game.correct_positions.iter().enumerate() {
            if let Some(expected) = maybe_correct {
                if chars[i] != *expected {
                    return false;
                }
            }
        }
        true
    }

    fn respects_misplaced_constraints(&self, chars: &[char]) -> bool {
        for (pos, letters) in &self.game.misplaced_letters {
            for &letter in letters {
                if chars[*pos] == letter {
                    return false;
                }
                if !chars.contains(&letter) {
                    return false;
                }
            }
        }
        true
    }

    fn contains_required_letters(&self, chars: &[char]) -> bool {
        let present: HashSet<char> = chars.iter().copied().collect();
        for &req in &self.game.must_contain {
            if !present.contains(&req) {
                return false;
            }
        }
        true
    }
}
