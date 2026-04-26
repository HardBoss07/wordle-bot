use crate::game::GameData;
use crate::solver::Solver;
use crate::stats::SimulationResults;
use crate::trap;
use crate::util;
use anyhow::Result;
use rand::prelude::*;
use rand::rngs::ThreadRng;

pub fn run_simulation(num_runs: usize) -> Result<()> {
    let solver = Solver::new()?;
    let all_words = solver.all_words.clone();

    let stats_json = util::read_letter_stats()?;

    let mut results = SimulationResults::new();
    let mut rng = ThreadRng::default();

    println!("Starting simulation of {} games...", num_runs);

    let target_words = if all_words.len() > 10657 {
        &all_words[10657..]
    } else {
        &all_words[..]
    };

    if target_words.is_empty() {
        return Err(anyhow::anyhow!(
            "No target words available for simulation. Ensure 'wordlist.txt' is correct."
        ));
    }

    let weights = util::read_solver_config()?;

    for i in 0..num_runs {
        let target_word = target_words
            .choose(&mut rng)
            .expect("Target word list is empty");

        // Use custom simulation loop here to inject trap catching logic
        let num_guesses = simulate_game(&solver, target_word.to_string(), &stats_json, &weights)?;

        results.record_game(num_guesses);

        if (i + 1) % 100 == 0 {
            println!("... {} games simulated ...", i + 1);
        }
    }

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
