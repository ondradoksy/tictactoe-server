//! Implements a bot logic that utilizes the Minimax algorithm for decision-making.

use crate::{ bot::botlogic::BotLogic, game::Game, grid::Grid, player_move::PlayerMove, Size };

/// A struct that represents a bot employing the Minimax algorithm to select moves.
pub(crate) struct MinMaxBot {}
impl BotLogic for MinMaxBot {
    /// Generates a move using Minimax reasoning based on the current game state.
    ///
    /// # Arguments
    ///
    /// * `id`: The bot's unique identifier.
    /// * `game`: A reference to the current game instance.
    ///
    /// # Returns
    ///
    /// A `Size` representing the determined optimal move, or the move (0, 0) if no valid moves are available.
    fn generate_move(&self, id: i32, game: &Game) -> Size {
        Self::get_best_move(id, game).unwrap_or(Size::new(0, 0))
    }

    /// Returns a string representing the bot logic type ("minmax").
    ///
    /// # Returns
    ///
    /// A string representing the bot logic type.
    fn get_name(&self) -> String {
        "minmax".to_string()
    }
}
impl MinMaxBot {
    /// Creates a new instance of `MinMaxBot`.
    ///
    /// # Returns
    ///
    /// A new `MinMaxBot` instance.
    pub fn new() -> Self {
        Self {}
    }

    /// Calculates and returns the move determined to be best using the Minimax algorithm.
    ///
    /// # Arguments
    ///
    /// * `id`: The bot's unique identifier.
    /// * `game`: A reference to the current game instance.
    ///
    /// # Returns
    ///
    /// An `Option<Size>` representing the move deemed as best, or `None` if no valid moves exist.
    fn get_best_move(id: i32, game: &Game) -> Option<Size> {
        let moves = game.grid.get_possible_moves(id);

        if moves.len() < 1 {
            return None;
        }

        let depth: u32 = get_depth(
            moves.len().try_into().expect("Could not convert usize to u32"),
            100000
        ) as u32;
        println!("MINMAX | Chosen depth: {}", depth);

        let mut move_counter = 0;
        let total = get_complexity(
            moves.len().try_into().expect("Could not convert usize to u32"),
            depth.into()
        );

        let (high_score, best_move) = Self::find_best_move(
            moves,
            &game.grid,
            &game.player_list,
            &game.win_length,
            game.current_turn,
            id,
            depth,
            &mut move_counter,
            &total
        );

        println!("Proceeding with move {:?} with score {:?}", best_move, high_score);
        Some(best_move)
    }

    /// Evaluates a potential move using a recursive Minimax approach.
    ///
    /// # Arguments
    ///
    /// * `m`: A `PlayerMove` representing the potential move to be evaluated.
    /// * `mut grid`: A mutable reference to the game grid used for simulation purposes.
    /// * `id`: The bot's unique identifier.
    /// * `player_list`: A reference to the list of players in the game.
    /// * `win_length`: A reference to the game's win length.
    /// * `current_turn`: The current turn index in the game.
    /// * `depth`: The remaining depth for recursive calls (used for search limitation).
    /// * `move_counter`: A mutable reference to a counter tracking the number of evaluated moves.
    /// * `total`: A reference to the total number of possible moves (used for progress tracking).
    ///
    /// # Returns
    ///
    /// A `Vec<i32>` representing the scores for each player based on the simulation outcome.
    fn get_score(
        m: &PlayerMove,
        mut grid: Grid,
        id: i32,
        player_list: &Vec<i32>,
        win_length: &u32,
        current_turn: usize,
        depth: u32,
        move_counter: &mut u128,
        total: &u128
    ) -> Vec<i32> {
        if *move_counter % 100000 == 0 {
            println!("Processing move {}/{} at depth: {}", move_counter, total, depth);
        }
        *move_counter += 1;

        let mut sum: Vec<i32> = vec![0; player_list.len()];

        grid.add(m.clone());

        let moves = grid.check_win(&m.position, *win_length);

        let won = moves.len() > 0;

        if won {
            sum[current_turn] = 2;
            return sum;
        }

        if depth <= 0 {
            return vec![1; player_list.len()];
        }

        grid.add_range(&moves);

        let next_turn = (current_turn + 1) % player_list.len();

        let possible_moves = grid.get_possible_moves(player_list[next_turn]);

        if possible_moves.len() > 0 {
            let (high_score, _best_move) = Self::find_best_move(
                possible_moves,
                &grid,
                player_list,
                win_length,
                next_turn,
                id,
                depth,
                move_counter,
                total
            );
            sum = high_score;
        } else if !won {
            return vec![1; player_list.len()];
        }

        sum
    }

    /// Discovers the optimal move and its corresponding score within a set of possibilities.
    /// This function utilizes a recursive Minimax approach to evaluate potential moves.
    ///
    /// # Arguments
    ///
    /// * `moves`: A vector containing the `PlayerMove` options to be considered.
    /// * `grid`: A reference to the game grid used for simulation purposes.
    /// * `player_list`: A reference to the list of players in the game.
    /// * `win_length`: A reference to the game's win length.
    /// * `current_turn`: The current turn index in the game.
    /// * `id`: The bot's unique identifier.
    /// * `depth`: The remaining depth for recursive calls (used for search limitation).
    /// * `move_counter`: A mutable reference to a counter tracking the number of evaluated moves.
    /// * `total`: A reference to the total number of possible moves (used for progress tracking).
    ///
    /// # Returns
    ///
    /// A tuple containing a `Vec<i32>` representing the scores for each player based on the
    /// simulation outcome, and a `Size` representing the move determined to be optimal.

    fn find_best_move(
        moves: Vec<PlayerMove>,
        grid: &Grid,
        player_list: &Vec<i32>,
        win_length: &u32,
        current_turn: usize,
        id: i32,
        depth: u32,
        move_counter: &mut u128,
        total: &u128
    ) -> (Vec<i32>, Size) {
        let mut high_score: Vec<i32> = vec![i32::MIN; player_list.len()];
        let mut best_move = moves[0].position;

        for m in moves {
            let score = Self::get_score(
                &m,
                grid.clone(),
                id,
                player_list,
                win_length,
                current_turn,
                depth - 1,
                move_counter,
                total
            );
            if score[current_turn] > high_score[current_turn] {
                high_score = score;
                best_move = m.position;
            }
            *move_counter += 1;
        }
        (high_score, best_move)
    }
}

/// Calculates the estimated number of possible game states based on the number of moves and depth.
///
/// # Arguments
///
/// * `num`: The number of available moves at a specific point in the game.
/// * `depth`: The remaining depth for recursive calls (used for search limitation).
///
/// # Returns
///
/// A `u128` representing the estimated number of possible game states.
fn get_complexity(num: u128, depth: u128) -> u128 {
    if depth <= 0 {
        return 1;
    }
    match num {
        0 => 1,
        1 => 1,
        _ => num * get_complexity(num - 1, depth - 1),
    }
}

/// Determines the maximum achievable search depth based on the number of possible moves
/// and a specified complexity limit. This helps prevent excessive calculations.
///
/// # Arguments
///
/// * `num`: The number of possible moves (represented as a u128).
/// * `complexity`: The maximum allowed complexity (represented as a u128).
///
/// # Returns
///
/// A u128 value representing the chosen search depth.
fn get_depth(num: u128, complexity: u128) -> u128 {
    let mut depth = 1;
    let mut current_complexity = num;
    while depth < num && current_complexity * (num - depth) < complexity {
        current_complexity *= num - depth;
        depth += 1;
    }

    return depth;
}

#[test]
fn test_complexity() {
    for i in 1..10 {
        for j in 1..10 {
            let num = i * 10;
            let max_complexity = j * 100000;
            let depth = get_depth(num, max_complexity);
            let complexity = get_complexity(i * 10, depth);
            assert!(
                complexity <= max_complexity,
                "{} <= {} is not true!",
                complexity,
                max_complexity
            );
        }
    }
}
