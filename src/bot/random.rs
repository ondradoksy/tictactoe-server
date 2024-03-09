use crate::{ bot::botlogic::BotLogic, Size, grid::Grid };

use rand::Rng;

pub(crate) struct RandomBot {}
impl RandomBot {
    pub fn new() -> Self {
        Self {}
    }
}
impl BotLogic for RandomBot {
    fn generate_move(&self, id: i32, grid: &Grid) -> Size {
        let moves = grid.get_possible_moves(id);

        if moves.len() < 1 {
            return Size::new(0, 0);
        }

        moves[rand::thread_rng().gen_range(0..moves.len())].position
    }
    fn get_name(&self) -> String {
        "random".to_string()
    }
}
