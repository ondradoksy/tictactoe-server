use serde::{ Deserialize, Serialize };

use crate::common::Size;

#[derive(Serialize, Deserialize, Clone)]
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
    pub fn from_string(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}
impl Into<String> for &PlayerMove {
    fn into(self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
