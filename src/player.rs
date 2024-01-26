use std::sync::{ mpsc::Sender, Arc, Mutex };

use serde::Serialize;

use crate::{
    net::{ MessageEvent, GameJoinData, Status, broadcast_players, send_to_player },
    game::Game,
    common::get_object,
};

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
    pub fn join_game(
        player: &Arc<Mutex<Player>>,
        event: &MessageEvent,
        games: &Arc<Mutex<Vec<Arc<Mutex<Game>>>>>,
        players: &Arc<Mutex<Vec<Arc<Mutex<Player>>>>>
    ) -> MessageEvent {
        let join_data = GameJoinData::from_json(&event.content);
        if join_data.is_err() {
            return MessageEvent::new(
                event.event.clone(),
                Status::new("error", join_data.err().unwrap().to_string())
            );
        }

        let id = join_data.unwrap().id;
        let game = get_object(&games, |p| p.lock().unwrap().id == id);

        // Check if game exists
        if game.is_none() {
            return MessageEvent::new(
                event.event.clone(),
                Status::new("error", "Game does not exist.")
            );
        }

        if !game.unwrap().lock().unwrap().join_player(&player) {
            return MessageEvent::new(event.event.clone(), Status::new("error", "Can't join game."));
        }

        broadcast_players(&players);
        send_to_player(&player, &MessageEvent::new("joined_game", GameJoinData::new(id)));
        MessageEvent::new(event.event.clone(), Status::new("ok", ""))
    }
}
impl PartialEq for Player {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
