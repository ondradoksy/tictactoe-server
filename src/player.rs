use std::sync::{ mpsc::Sender, Arc, Mutex };

use serde::Serialize;

use crate::{ net::MessageEvent, game::Game };

#[derive(Serialize)]
pub(crate) struct Player {
    pub id: u32,
    #[serde(skip_serializing)]
    pub tx: Sender<MessageEvent>,
    pub joined_game: Option<Arc<Mutex<Game>>>,
}
impl Player {
    pub fn new(id: u32, tx: Sender<MessageEvent>) -> Self {
        Self {
            id: id,
            tx: tx,
            joined_game: None,
        }
    }
}
