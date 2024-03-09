use crate::{ Size, bot::Grid };

pub(crate) trait BotLogic {
    fn generate_move(&self, id: i32, grid: &Grid) -> Size;
    fn get_name(&self) -> String;
}
