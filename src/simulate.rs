use crate::analysis::LetterStats;
use crate::filter::Filter;
use crate::game::GameData;
use crate::ranking::weighted_rank_stats;
use crate::solver::Solver;
use crate::stats::SimulationResults;
use crate::trap;
use crate::util::{self, ANSWER_WORD_INDEX};
use anyhow::Result;
use indicatif::ParallelProgressIterator;
use rand::prelude::*;
use rand::rngs::ThreadRng;
use rayon::prelude::*;
use std::time::Instant;

pub fn run_simulation(num_runs: usize) -> Result<()> {
    let start_time = Instant::now();

    let solver = Solver::new()?;
    let all_words = solver.all_words.clone();
    let all_word_refs: Vec<&str> = all_words.iter().map(|s| s.as_str()).collect();

    let stats_json = util::read_letter_stats()?;
    let stats: LetterStats = serde_json::from_str(&stats_json)?;

    let first_guess_str = solver.get_top_suggestion_silent_stats(&stats, None)?;
    let first_guess: &str = all_word_refs
        .iter()
        .find(|&&w| w == first_guess_str)
        .copied()
        .unwrap_or(all_word_refs[0]);

    println!("Starting simulation of {} games...", num_runs);
    println!("Opening guess precomputed: '{}'", first_guess);

    let target_words = if all_words.len() > ANSWER_WORD_INDEX {
        &all_words[ANSWER_WORD_INDEX..]
    } else {
        &all_words[..]
    };

    if target_words.is_empty() {
        return Err(anyhow::anyhow!(
            "No target words available for simulation. Ensure 'wordlist.txt' is correct."
        ));
    }

    let weights = util::read_solver_config()?;

    let results = (0..num_runs)
        .into_par_iter()
        .progress_count(num_runs as u64)
        .fold(SimulationResults::new, |mut local_results, _| {
            let mut local_rng = ThreadRng::default();
            let target_word = target_words
                .choose(&mut local_rng)
                .expect("Target word list is empty");

            let num_guesses = simulate_game(
                &all_words,
                &all_word_refs,
                target_word,
                first_guess,
                &stats,
                &weights,
            )
            .expect("Simulation failed");

            local_results.record_game(num_guesses);
            local_results
        })
        .reduce(SimulationResults::new, |mut a, b| {
            a.merge(b);
            a
        });

    let elapsed = start_time.elapsed();

    println!("\nSimulation finished.");
    results.print_summary();

    if elapsed.as_secs_f64() >= 1.0 {
        println!("Execution time: {:.3}s", elapsed.as_secs_f64());
    } else {
        println!("Execution time: {}ms", elapsed.as_millis());
    }

    Ok(())
}

fn simulate_game<'a>(
    all_words: &'a [String],
    all_word_refs: &[&'a str],
    target_word: &str,
    first_guess: &'a str,
    stats: &LetterStats,
    weights: &[(f64, f64, f64)],
) -> Result<usize> {
    let mut game = GameData::new();
    let mut current_words: Vec<&'a str> = all_word_refs.to_vec();
    let mut guesses = 0;
    let max_guesses = 6;

    while guesses < max_guesses {
        let attempt = game.lines.len().min(weights.len() - 1);
        let weight_tuple = weights[attempt];

        let guess_word: &'a str = if guesses == 0 {
            first_guess
        } else {
            let filter = Filter::new(&game, &current_words);
            current_words = filter.filter_words();

            if current_words.is_empty() {
                return Ok(max_guesses + 1);
            }

            if let Some(trap) = trap::detect_trap(&current_words) {
                if let Some((elim_word, _)) = trap::find_best_elimination(all_words, &trap) {
                    all_word_refs
                        .iter()
                        .find(|&&w| w == elim_word)
                        .copied()
                        .unwrap_or(current_words[0])
                } else {
                    let ranked = weighted_rank_stats(&current_words, stats, weight_tuple);
                    if ranked.is_empty() {
                        current_words[0]
                    } else {
                        let top_word = &ranked[0].0;
                        all_word_refs
                            .iter()
                            .find(|&&w| w == top_word)
                            .copied()
                            .unwrap_or(current_words[0])
                    }
                }
            } else {
                let ranked = weighted_rank_stats(&current_words, stats, weight_tuple);
                if ranked.is_empty() {
                    current_words[0]
                } else {
                    let top_word = &ranked[0].0;
                    all_word_refs
                        .iter()
                        .find(|&&w| w == top_word)
                        .copied()
                        .unwrap_or(current_words[0])
                }
            }
        };

        guesses += 1;

        if guess_word == target_word {
            return Ok(guesses);
        }

        let pattern_code = Solver::evaluate_pattern(guess_word, target_word);
        game.add_line_code(guess_word, pattern_code);
    }

    Ok(max_guesses + 1)
}
