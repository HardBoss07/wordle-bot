use crate::solver::Solver;
use crate::stats::SimulationResults;
use anyhow::Result;
use rand::prelude::*;
use rand::rngs::ThreadRng;
use std::fs;

pub fn run_simulation(num_runs: usize) -> Result<()> {
    let solver = Solver::new()?;
    let all_words = solver.all_words.clone();

    let stats_json = fs::read_to_string("letter_stats.json")?;
    let config_json = fs::read_to_string("solver_config.json")?;

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

    for i in 0..num_runs {
        let target_word = target_words
            .choose(&mut rng)
            .expect("Target word list is empty");

        let num_guesses = solver.simulate(target_word.to_string(), &stats_json, &config_json)?;

        results.record_game(num_guesses);

        if (i + 1) % 100 == 0 {
            println!("... {} games simulated ...", i + 1);
        }
    }

    println!("\nSimulation finished.");
    results.print_summary();

    Ok(())
}
