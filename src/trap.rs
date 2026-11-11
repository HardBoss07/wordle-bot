use std::collections::HashSet;

pub struct TrapInfo {
    pub distinguishing_letters: HashSet<char>,
    pub varying_positions: Vec<usize>,
}

pub fn detect_trap<S: AsRef<str>>(words: &[S]) -> Option<TrapInfo> {
    if words.len() < 3 || words.len() > 12 {
        return None;
    }

    let mut identical_count = 0;
    let mut varying_positions = Vec::new();
    let mut distinguishing_letters = HashSet::new();

    for i in 0..5 {
        let mut unique_chars = HashSet::new();
        for word in words {
            if let Some(c) = word.as_ref().chars().nth(i) {
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

    if identical_count >= 2 {
        Some(TrapInfo {
            distinguishing_letters,
            varying_positions,
        })
    } else {
        None
    }
}

pub fn score_elimination_word<S: AsRef<str>>(
    word: &str,
    trap: &TrapInfo,
    current_words: &[S],
) -> usize {
    let word_chars: HashSet<char> = word.chars().collect();
    let mut score = 0;

    for &c in &trap.distinguishing_letters {
        if word_chars.contains(&c) {
            let count = current_words
                .iter()
                .filter(|w| w.as_ref().contains(c))
                .count();
            score += count;
        }
    }
    score
}

pub fn find_best_elimination<S: AsRef<str>, C: AsRef<str>>(
    all_words: &[S],
    trap: &TrapInfo,
    current_words: &[C],
) -> Option<(String, usize)> {
    all_words
        .iter()
        .map(|w| {
            (
                w.as_ref().to_string(),
                score_elimination_word(w.as_ref(), trap, current_words),
            )
        })
        .max_by_key(|&(_, score)| score)
        .filter(|&(_, score)| score > 0)
}
