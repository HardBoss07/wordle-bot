use crate::analysis::LetterStats;
use anyhow::Result;
use serde_json;

pub fn rank_words<'a>(words: &[&'a str], stats_json: &str) -> Result<Vec<(String, f64)>> {
    let w_pos = 0.2;
    let w_overall = 0.1;
    let w_unique = 0.7;

    weighted_rank(words, stats_json, (w_pos, w_overall, w_unique))
}

pub fn weighted_rank<'a>(
    words: &[&'a str],
    stats_json: &str,
    weights: (f64, f64, f64),
) -> Result<Vec<(String, f64)>> {
    let stats: LetterStats = serde_json::from_str(stats_json)?;
    Ok(weighted_rank_stats(words, &stats, weights))
}

pub fn weighted_rank_stats<'a>(
    words: &[&'a str],
    stats: &LetterStats,
    weights: (f64, f64, f64),
) -> Vec<(String, f64)> {
    let (w_pos, w_overall, w_unique) = weights;

    let mut pos_scores = [[0.0f64; 5]; 26];
    let mut overall_scores = [0.0f64; 26];

    let mut totals = [0.0f64; 5];
    for counts in stats.counts.values() {
        for (i, &c) in counts.iter().enumerate() {
            totals[i] += c as f64;
        }
    }

    let mut grand_total = 0.0f64;
    for (&ch, counts) in &stats.counts {
        if ch.is_ascii_lowercase() {
            let idx = (ch as u8 - b'a') as usize;
            let sum: f64 = counts.iter().map(|&c| c as f64).sum();
            overall_scores[idx] = sum;
            grand_total += sum;

            for i in 0..5 {
                if totals[i] > 0.0 {
                    pos_scores[idx][i] = counts[i] as f64 / totals[i];
                }
            }
        }
    }

    if grand_total > 0.0 {
        for score in &mut overall_scores {
            *score /= grand_total;
        }
    }

    let mut scores: Vec<(String, f64)> = Vec::with_capacity(words.len());

    for &word in words {
        let bytes = word.as_bytes();
        if bytes.len() != 5 {
            continue;
        }

        let mut score_pos = 0.0;
        let mut score_overall = 0.0;
        let mut mask = 0u32;

        for (i, &b) in bytes.iter().enumerate() {
            let idx = (b - b'a') as usize;
            score_pos += pos_scores[idx][i];
            score_overall += overall_scores[idx];
            mask |= 1 << (b - b'a');
        }

        score_pos /= 5.0;
        score_overall /= 5.0;

        let uniqueness = mask.count_ones() as f64 / 5.0;

        let blended_score = w_pos * score_pos + w_overall * score_overall + w_unique * uniqueness;
        scores.push((word.to_string(), blended_score));
    }

    scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scores
}
