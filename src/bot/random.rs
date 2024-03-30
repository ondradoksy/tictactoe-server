use crate::{ bot::botlogic::BotLogic, game::Game, grid::Grid, Size };

use rand::Rng;

pub(crate) struct RandomBot {}
impl BotLogic for RandomBot {
    fn generate_move(&self, id: i32, game: &Game) -> Size {
        RandomBot::get_random_move(id, &game.grid).unwrap_or(Size::new(0, 0))
    }
    fn get_name(&self) -> String {
        "random".to_string()
    }
}
impl RandomBot {
    pub fn new() -> Self {
        Self {}
    }
    pub fn get_random_move(id: i32, grid: &Grid) -> Option<Size> {
        let moves = grid.get_possible_moves(id);

        if moves.len() < 1 {
            return None;
        }

        Some(moves[rand::thread_rng().gen_range(0..moves.len())].position)
    }
}
