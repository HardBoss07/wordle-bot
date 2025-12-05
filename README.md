# wordle-bot

A command-line Rust bot for analyzing, ranking, solving, playing, and simulating Wordle puzzles.
It provides detailed letter statistics, ranks words based on frequency and position, simulates solver strategies, and includes an interactive game mode.

With a minor self test of 50 games I averaged 4.42 guesses per game with 0 missed words. This data may not be accurate nor guarenteed.

## Features

  * **Analyze**: Generate letter statistics from a word list (`letter_stats.json`).
  * **Rank**: Rank words by letter frequency and positional value.
  * **Solve**: Solve Wordle puzzles using a frequency-based algorithm with adjustable weighting.
  * **Play**: Play an interactive Wordle game directly in the terminal.
  * **Simulate**: Run bulk simulations of the solver against random target words to determine win rate, average guesses, and guess distribution.
  * Fully written in Rust, with a modular design (`analysis`, `ranking`, `solver`, `filter`, `game`, `play`, `simulate`, `stats`).

## Installation

### From crates.io

```bash
cargo install wordle-bot
```

### From source (GitHub)

```bash
git clone https://github.com/HardBoss07/wordle-bot.git
cd wordle-bot
cargo build --release
```

## Usage

```bash
\.wordle-bot.exe <analyze|rank|solve|play|simulate>
```

## Commands

#### **analyze**

Generates `letter_stats.json` from `wordlist.txt`, containing frequency and positional statistics for all letters.

#### **rank**

Ranks all words in `wordlist.txt` using the precomputed letter statistics.
Outputs the top-ranked words based on configurable weighting.

#### **solve**

Runs the automated solver module.
You can enter guesses and feedback (`w`, `m`, `c`) to progressively narrow down possible words.

#### **play**

Starts an interactive Wordle game in your terminal.
The bot selects a random word from the word list, and you have six guesses to find it.
Each guess displays feedback in a color-coded grid (e.g. green = correct position, yellow = correct letter, gray = absent).

```bash
\.wordle-bot.exe play
```

Example session:

```
=== Current Game State ===
Nr.  Word
1.   C  R  A  N  E
2.   S  T  O  N  E
==========================

Congratulations! You've guessed the word: STONE
```

#### **simulate**

Runs a simulation of the solver for a specified number of games against random target words. The solver will automatically pick its top-ranked word for each guess.

The output will include:

  * Total Win Rate
  * Average guesses needed (for games won)
  * Guess Distribution (how many games were solved in 1, 2, 3... guesses, and how many were lost)

**Usage:**

```bash
\.wordle-bot.exe simulate <num_runs>
# Example: Run 1000 simulated games
\.wordle-bot.exe simulate 1000
```

## Tweaking the Solver (`solver_config.json`)

The solver uses a **weighted ranking system** to balance three factors when suggesting the next guess:

| Weight      | Meaning              | Description                                     |
| ----------- | -------------------- | ----------------------------------------------- |
| `w_pos`     | Positional frequency | How common a letter is in a specific position   |
| `w_overall` | Overall frequency    | How common a letter is overall in all positions |
| `w_unique`  | Uniqueness           | Preference for words with more unique letters   |

These values are configured in `solver_config.json`, which defines a list of weights applied per turn (first guess, second guess, etc.):

```json
[
    [0.1, 0.2, 0.7],
    [0.15, 0.25, 0.6],
    [0.25, 0.35, 0.4],
    [0.35, 0.45, 0.2],
    [0.45, 0.45, 0.1],
    [0.6, 0.35, 0.05]
]
```

Each entry corresponds to a turn number:

| Turn | `[w_pos, w_overall, w_unique]`     | Behavior                                                              |
| ---- | ---------------------------------- | --------------------------------------------------------------------- |
| 1    | `[0.1, 0.2, 0.7]`                  | Focus on letter variety to reveal as many unique letters as possible. |
| 2    | `[0.15, 0.25, 0.6]`                | Still prioritizes diversity but starts weighing frequency more.       |
| 3-6  | Increasing `w_pos` and `w_overall` | Gradually shifts toward accuracy and positional matching.             |

To tweak solver behavior:

1.  Open `solver_config.json`.
2.  Adjust the numbers (they should roughly sum to 1.0, but it's not required).
3.  **Run `wordle-bot simulate <num_runs>`** to test your new configuration across many games and measure its impact on average guesses and win rate.
4.  Run the solver again — it automatically reloads the new weights each turn.

### Tips

  * Increase `w_unique` for early-game exploration.
  * Increase `w_pos` and `w_overall` for late-game precision.
  * You can define more entries for longer simulations (e.g., 7th or 8th guesses).

## Project Structure

```
src/
├── analysis.rs   # Letter statistics computation
├── ranking.rs    # Word ranking logic
├── solver.rs     # Wordle solving logic
├── filter.rs     # Word filtering logic
├── game.rs       # Game management and state
├── play.rs       # Interactive game mode
├── simulate.rs   # Simulation driver loop
├── stats.rs      # Simulation statistics collection and reporting
└── main.rs       # CLI entry point
```

  * `wordlist.txt` : Input word list (5-letter words)
  * `letter_stats.json` : Generated letter statistics
  * `solver_config.json` : Solver weight configuration file

## License

AGPL-3.0 (see [LICENSE]([LICENSE](https://github.com/HardBoss07/wordle-bot/blob/main/LICENSE))) 