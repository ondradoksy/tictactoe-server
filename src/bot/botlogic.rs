//! Provides a trait for defining bot logic to be used by the [`crate::bot::Bot`] struct.
use crate::{ Size, game::Game };

/// A trait for implementing bot logic that can be used with the [`crate::bot::Bot`] struct.
pub(crate) trait BotLogic {
    /// Generates a move for the bot based on its current state and the game state.
    ///
    /// # Parameters
    ///
    /// * `id`: The bot's unique identifier.
    /// * `game`: The current state of the game.
    ///
    /// # Returns
    ///
    /// The position of the move the bot wants to make as a [`Size`].
    fn generate_move(&self, id: i32, game: &Game) -> Size;

    /// Returns the name of the bot algorithm.
    ///
    /// # Returns
    ///
    /// The name of the bot algorithm as a [`String`].
    fn get_name(&self) -> String;
}
