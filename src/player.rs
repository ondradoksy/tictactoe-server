use std::sync::{ mpsc::Sender, Arc, Mutex };
use image::{ codecs::png::PngEncoder, ImageBuffer, ImageEncoder, Rgba };
use serde::Serialize;
use base64::{ prelude::BASE64_STANDARD, Engine };

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
    #[serde(skip_serializing)]
    image: Option<String>,
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
            image: None,
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

    /// Returns the player's image encoded in base64.
    pub fn get_image(&mut self) -> String {
        if self.image.is_none() {
            self.image = Some(self.generate_image());
        }
        self.image.clone().unwrap()
    }

    /// Generates an image with a pattern using a function.
    fn generate_image(&self) -> String {
        // Create buffer
        let img = ImageBuffer::from_fn(16, 16, self.get_pattern_fn());

        // Write the image to a Vec<u8>
        let mut buffer: Vec<u8> = Vec::new();
        PngEncoder::new(&mut buffer)
            .write_image(&img, img.width(), img.height(), image::ColorType::Rgba8)
            .unwrap();

        // Encode the buffer as base64
        BASE64_STANDARD.encode(&buffer)
    }

    /// Returns a function used to geneerate a pattern in a new image.
    fn get_pattern_fn(&self) -> Box<dyn Fn(u32, u32) -> Rgba<u8>> {
        let primary_color = Rgba([
            if self.id % 4 == 3 { 128 } else { 0 },
            if self.id % 4 == 2 { 128 } else { 0 },
            if self.id % 4 == 1 { 128 } else { 0 },
            255,
        ]);
        let secondary_color = Rgba([
            if self.id % 3 == 0 { 255 } else { 0 },
            if self.id % 3 == 1 { 255 } else { 0 },
            if self.id % 3 == 2 { 255 } else { 0 },
            255,
        ]);
        match self.id % 3 {
            2 =>
                Box::new(move |_x, y| {
                    if y % 2 == 0 { primary_color } else { secondary_color }
                }),
            1 =>
                Box::new(move |x, _y| {
                    if x % 2 == 0 { primary_color } else { secondary_color }
                }),
            _ =>
                Box::new(move |x, y| {
                    if x % 2 == 0 && y % 2 == 0 { primary_color } else { secondary_color }
                }),
        }
    }
}
impl PartialEq for Player {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
