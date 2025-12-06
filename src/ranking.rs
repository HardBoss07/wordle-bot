use crate::analysis::LetterStats;
use anyhow::Result;
use serde_json;
use std::collections::HashMap;

pub fn rank_words<'a>(words: &[&'a str], stats_json: &str) -> Result<Vec<(String, f64)>> {
    // === Adjustable weight parameters ===
    let w_pos = 0.2; // weight for positional frequency
    let w_overall = 0.1; // weight for overall frequency
    let w_unique = 0.7; // weight for letter uniqueness

    weighted_rank(words, stats_json, (w_pos, w_overall, w_unique))
}

pub fn weighted_rank<'a>(
    words: &[&'a str],
    stats_json: &str,
    weights: (f64, f64, f64),
) -> Result<Vec<(String, f64)>> {
    let (w_pos, w_overall, w_unique) = weights;
    let stats: LetterStats = serde_json::from_str(stats_json)?;

    // === Compute positional totals ===
    let mut totals = [0.0; 5];
    for counts in stats.counts.values() {
        for (i, &c) in counts.iter().enumerate() {
            totals[i] += c as f64;
        }
    }

    // === Compute overall totals ===
    let mut overall_totals = HashMap::new();
    let mut grand_total = 0.0;
    for (ch, counts) in &stats.counts {
        let sum: f64 = counts.iter().map(|&c| c as f64).sum();
        overall_totals.insert(*ch, sum);
        grand_total += sum;
    }

    // === Rank each word ===
    let mut scores: Vec<(String, f64)> = Vec::new();

    for &word in words {
        let chars: Vec<char> = word.chars().collect();
        if chars.len() != 5 || !chars.iter().all(|c| c.is_ascii_lowercase()) {
            continue;
        }

        let mut score_pos = 0.0;
        let mut score_overall = 0.0;

        for (i, &ch) in chars.iter().enumerate() {
            // Positional frequency score
            let pos_score = if let Some(counts) = stats.counts.get(&ch) {
                if totals[i] > 0.0 {
                    counts[i] as f64 / totals[i]
                } else {
                    0.0
                }
            } else {
                0.0
            };

            // Overall frequency score
            let overall_score = if let Some(&sum) = overall_totals.get(&ch) {
                sum / grand_total
            } else {
                0.0
            };

            score_pos += pos_score;
            score_overall += overall_score;
        }

        // Normalize by length (so 5-letter words stay comparable)
        score_pos /= 5.0;
        score_overall /= 5.0;

        // === Uniqueness score ===
        let unique_letters = chars.iter().collect::<std::collections::HashSet<_>>();
        let uniqueness = unique_letters.len() as f64 / 5.0; // 0.2–1.0 range

        // === Weighted blend ===
        let blended_score = w_pos * score_pos + w_overall * score_overall + w_unique * uniqueness;

        scores.push((word.to_string(), blended_score));
    }

    // Sort descending by score
    scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    Ok(scores)
}
