//! Implements a bot logic that uses a random strategy.

use crate::{ bot::botlogic::BotLogic, game::Game, grid::Grid, Size };

use rand::Rng;

/// A struct representing a bot that uses a random strategy to make moves.
pub(crate) struct RandomBot {}
impl BotLogic for RandomBot {
    /// Generates a random valid move for the bot based on the current game state.
    ///
    /// # Arguments
    ///
    /// * `id`: The bot's unique identifier.
    /// * `game`: A reference to the current game instance.
    ///
    /// # Returns
    ///
    /// A `Size` representing the position of the randomly chosen move, or the move (0, 0) if no valid moves are available.
    fn generate_move(&self, id: i32, game: &Game) -> Size {
        RandomBot::get_random_move(id, &game.grid).unwrap_or(Size::new(0, 0))
    }

    /// Returns a string representing the bot logic type ("random").
    ///
    /// # Returns
    ///
    /// A string representing the bot logic type.
    fn get_name(&self) -> String {
        "random".to_string()
    }
}
impl RandomBot {
    /// Creates a new instance of `RandomBot`.
    ///
    /// # Returns
    ///
    /// A new `RandomBot` instance.
    pub fn new() -> Self {
        Self {}
    }

    /// Generates a random valid move for the bot based on the provided grid.
    ///
    /// # Arguments
    ///
    /// * `id`: The bot's unique identifier.
    /// * `grid`: A reference to the game grid.
    ///
    /// # Returns
    ///
    /// An `Option<Size>` representing the position of the randomly chosen move,
    /// or `None` if no valid moves are available.
    pub fn get_random_move(id: i32, grid: &Grid) -> Option<Size> {
        let moves = grid.get_possible_moves(id);

        if moves.len() < 1 {
            return None;
        }

        Some(moves[rand::thread_rng().gen_range(0..moves.len())].position)
    }
}
