use std::collections::HashSet;

pub struct TrapInfo {
    pub distinguishing_letters: HashSet<char>,
    pub varying_positions: Vec<usize>,
}

pub fn detect_trap(words: &[String]) -> Option<TrapInfo> {
    if words.len() < 3 || words.len() > 10 {
        return None;
    }

    let mut identical_count = 0;
    let mut varying_positions = Vec::new();
    let mut distinguishing_letters = HashSet::new();

    for i in 0..5 {
        let mut unique_chars = HashSet::new();
        for word in words {
            if let Some(c) = word.chars().nth(i) {
                unique_chars.insert(c);
            }
        }

        if unique_chars.len() == 1 {
            identical_count += 1;
        } else {
            varying_positions.push(i);
            for c in unique_chars {
                distinguishing_letters.insert(c);
            }
        }
    }

    // Trigger if we share at least 3 positions
    if identical_count >= 3 {
        Some(TrapInfo {
            distinguishing_letters,
            varying_positions,
        })
    } else {
        None
    }
}

pub fn score_elimination_word(word: &str, trap: &TrapInfo) -> usize {
    let word_chars: HashSet<char> = word.chars().collect();
    let mut score = 0;
    for &c in &trap.distinguishing_letters {
        if word_chars.contains(&c) {
            score += 1;
        }
    }
    score
}

pub fn find_best_elimination(all_words: &[String], trap: &TrapInfo) -> Option<(String, usize)> {
    all_words
        .iter()
        .map(|w| (w.clone(), score_elimination_word(w, trap)))
        .max_by_key(|&(_, score)| score)
        .filter(|&(_, score)| score > 0)
}
