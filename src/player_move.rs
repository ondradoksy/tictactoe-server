use serde::Serialize;

use crate::common::Size;

#[derive(Serialize)]
pub(crate) struct PlayerMove {
    pub player: u32,
    pub position: Size,
}
impl PlayerMove {
    pub fn new(player: u32, pos: Size) -> Self {
        Self {
            player: player,
            position: pos,
        }
    }
}
