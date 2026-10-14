use crate::game::GameData;

pub struct Filter<'g, 'w, 's> {
    game: &'g GameData,
    words: &'w [&'s str],
}

impl<'g, 'w, 's> Filter<'g, 'w, 's> {
    pub fn new(game: &'g GameData, words: &'w [&'s str]) -> Self {
        Self { game, words }
    }

    pub fn filter_words(&self) -> Vec<&'s str> {
        let mut must_contain_mask = 0u32;
        for &ch in &self.game.must_contain {
            if ch.is_ascii_lowercase() {
                must_contain_mask |= 1 << (ch as u8 - b'a');
            }
        }

        let mut correct = [0u8; 5];
        for (i, opt) in self.game.correct_positions.iter().enumerate() {
            if let Some(ch) = opt {
                correct[i] = *ch as u8;
            }
        }

        let mut misplaced_at = [0u32; 5];
        for (&pos, letters) in &self.game.misplaced_letters {
            if pos < 5 {
                for &ch in letters {
                    if ch.is_ascii_lowercase() {
                        misplaced_at[pos] |= 1 << (ch as u8 - b'a');
                    }
                }
            }
        }

        let mut forbidden_mask = 0u32;
        for &ch in &self.game.contains_not {
            if ch.is_ascii_lowercase() && !self.game.must_contain.contains(&ch) {
                forbidden_mask |= 1 << (ch as u8 - b'a');
            }
        }

        self.words
            .iter()
            .copied()
            .filter(|word| {
                let bytes = word.as_bytes();
                if bytes.len() != 5 {
                    return false;
                }

                let mut word_mask = 0u32;
                for &b in bytes {
                    word_mask |= 1 << (b - b'a');
                }

                if (word_mask & forbidden_mask) != 0 {
                    return false;
                }

                if (word_mask & must_contain_mask) != must_contain_mask {
                    return false;
                }

                for i in 0..5 {
                    let b = bytes[i];
                    let bit = 1 << (b - b'a');

                    if correct[i] != 0 && b != correct[i] {
                        return false;
                    }

                    if (misplaced_at[i] & bit) != 0 {
                        return false;
                    }
                }

                true
            })
            .collect()
    }
}
