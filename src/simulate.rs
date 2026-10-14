use crate::game::GameData;
use crate::solver::Solver;
use crate::stats::SimulationResults;
use crate::trap;
use crate::util::{self, ANSWER_WORD_INDEX};
use anyhow::Result;
use indicatif::ParallelProgressIterator;
use rand::prelude::*;
use rand::rngs::ThreadRng;
use rayon::prelude::*;

pub fn run_simulation(num_runs: usize) -> Result<()> {
    let solver = Solver::new()?;
    let all_words = solver.all_words.clone();

    let stats_json = util::read_letter_stats()?;

    println!("Starting simulation of {} games...", num_runs);

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

            // Use custom simulation loop here to inject trap catching logic
            let num_guesses =
                simulate_game(&solver, target_word.to_string(), &stats_json, &weights)
                    .expect("Simulation failed");

            local_results.record_game(num_guesses);
            local_results
        })
        .reduce(SimulationResults::new, |mut a, b| {
            a.merge(b);
            a
        });

    println!("\nSimulation finished.");
    results.print_summary();

    Ok(())
}

fn simulate_game(
    solver: &Solver,
    target_word: String,
    stats_json: &str,
    weights: &Vec<(f64, f64, f64)>,
) -> Result<usize> {
    let mut temp_solver = Solver {
        game: GameData::new(),
        current_words: solver.all_words.clone(),
        all_words: solver.all_words.clone(),
    };
    let mut guesses = 0;
    let max_guesses = 6;

    while guesses < max_guesses {
        let attempt = temp_solver.game.lines.len().min(weights.len() - 1);
        let weight_tuple = weights[attempt];

        let guess_word = if guesses == 0 {
            temp_solver.get_top_suggestion_silent(stats_json, None)?
        } else {
            // Update wordlist first
            temp_solver.current_words = temp_solver.update_wordlist();

            // TRAP CATCHING LOGIC
            if let Some(trap) = trap::detect_trap(&temp_solver.current_words) {
                if let Some((elim_word, _)) =
                    trap::find_best_elimination(&temp_solver.all_words, &trap)
                {
                    elim_word
                } else {
                    temp_solver.get_top_suggestion_silent(stats_json, Some(weight_tuple))?
                }
            } else {
                temp_solver.get_top_suggestion_silent(stats_json, Some(weight_tuple))?
            }
        };

        guesses += 1;

        if guess_word == target_word {
            return Ok(guesses);
        }

        // Evaluate and update
        let line = Solver::evaluate_word(&guess_word, &target_word);
        let pattern = Solver::get_pattern(&line);
        temp_solver.game.add_line(&guess_word, &pattern);
    }

    Ok(max_guesses + 1)
}
