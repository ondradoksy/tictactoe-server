use crate::{ Size, game::Game };

pub(crate) trait BotLogic {
    fn generate_move(&self, id: i32, game: &Game) -> Size;
    fn get_name(&self) -> String;
}
