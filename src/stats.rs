use std::collections::HashMap;

pub struct SimulationResults {
    total_games: usize,
    wins: usize,
    total_guesses: usize,
    guess_distribution: HashMap<usize, usize>, // Guesses -> Count
}

impl SimulationResults {
    pub fn new() -> Self {
        Self {
            total_games: 0,
            wins: 0,
            total_guesses: 0,
            guess_distribution: HashMap::new(),
        }
    }

    pub fn record_game(&mut self, num_guesses: usize) {
        self.total_games += 1;

        if num_guesses <= 6 {
            self.wins += 1;
            self.total_guesses += num_guesses;
            *self.guess_distribution.entry(num_guesses).or_insert(0) += 1;
        } else {
            *self.guess_distribution.entry(7).or_insert(0) += 1;
        }
    }

    pub fn print_summary(&self) {
        let avg_guesses = if self.wins > 0 {
            self.total_guesses as f64 / self.wins as f64
        } else {
            0.0
        };

        let win_rate = (self.wins as f64 / self.total_games as f64) * 100.0;

        println!("\n === Simulation Summary ===");
        println!("Total Games Simulated: {}", self.total_games);
        println!("Wins: {} (Win Rate: {:.2}%)", self.wins, win_rate);
        println!("Average Guesses (for wins): {:.3}", avg_guesses);
        println!("============================");

        println!("\nGuess Distribution (Guesses -> Count):");

        let bar_unit = (self.total_games / 50).max(1); // Calculate a unit for the bar

        for i in 1..=7 {
            let count = self.guess_distribution.get(&i).unwrap_or(&0);
            let label = if i <= 6 {
                format!("{}: ", i)
            } else {
                "Loss:".to_string()
            };
            // Print a simple distribution bar
            let bar = "█".repeat(*count / bar_unit);
            println!("{:<5}{:<8}{}", label, count, bar);
        }
        println!("============================\n");
    }
}
