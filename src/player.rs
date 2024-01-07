use std::sync::{ mpsc::Sender, Arc, Mutex };

use serde::Serialize;

use crate::{ net::MessageEvent, game::Game };

#[derive(Serialize)]
pub(crate) struct Player {
    pub id: u32,
    #[serde(skip_serializing)]
    pub tx: Sender<MessageEvent>,
    #[serde(skip_serializing)]
    pub joined_game: Option<Arc<Mutex<Game>>>,
    pub joined_game_id: Option<u32>,
    pub ready: bool,
    pub name: String,
}
impl Player {
    pub fn new(id: u32, tx: Sender<MessageEvent>) -> Self {
        Self {
            id: id,
            tx: tx,
            joined_game: None,
            joined_game_id: None,
            ready: false,
            name: "Unnamed".to_string(),
        }
    }
}
impl PartialEq for Player {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
