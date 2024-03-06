use serde::Serialize;

use crate::common::Size;

#[derive(Serialize, Clone)]
pub(crate) struct PlayerMove {
    pub player: i32,
    pub position: Size,
}
impl PlayerMove {
    pub fn new(player: i32, pos: Size) -> Self {
        Self {
            player: player,
            position: pos,
        }
    }
}
