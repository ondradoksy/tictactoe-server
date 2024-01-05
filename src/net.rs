use std::sync::{ Arc, Mutex };

use serde::{ Serialize, Deserialize };
use tungstenite::Message;

use crate::{ common::{ Size, from_json }, player::Player };

#[derive(Deserialize, Serialize, Clone)]
pub(crate) struct MessageEvent {
    pub event: String,
    pub content: String,
}
impl MessageEvent {
    pub fn new(event: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            event: event.into(),
            content: content.into(),
        }
    }
    pub fn new_empty() -> Self {
        Self {
            event: String::from(""),
            content: String::from(""),
        }
    }
    pub fn from_message(message: Message) -> Result<Self, String> {
        Self::from_json(message.to_text().unwrap())
    }
    pub fn from_json(text: &str) -> Result<Self, String> {
        from_json(text)
    }
    pub fn to_message(&self) -> Message {
        Message::Text(String::from(serde_json::to_string(&self).unwrap()))
    }
    pub fn is_empty(&self) -> bool {
        self.event.as_str() == ""
    }
}

#[derive(Deserialize)]
pub(crate) struct GameCreationData {
    pub size: Size,
    pub hotjoin: bool,
    pub player_limit: usize,
}
impl GameCreationData {
    #[cfg(test)]
    pub fn new(size: Size, hotjoin: bool, player_limit: usize) -> Self {
        Self {
            size: size,
            hotjoin: hotjoin,
            player_limit: player_limit,
        }
    }
    pub fn from_json(text: &str) -> Result<Self, String> {
        from_json(text)
    }
}

#[derive(Deserialize)]
pub(crate) struct GameJoinData {
    pub id: u32,
}
impl GameJoinData {
    pub fn from_json(text: &str) -> Result<Self, String> {
        from_json(text)
    }
}

pub(crate) struct InternalMessage {
    pub kind: InternalMessageKind,
    pub player: Arc<Mutex<Player>>,
    pub position: Option<Size>,
}

impl InternalMessage {
    pub fn new_join(player: Arc<Mutex<Player>>) -> Self {
        Self {
            kind: InternalMessageKind::PlayerJoin,
            player: player,
            position: None,
        }
    }
    pub fn new_move(player: Arc<Mutex<Player>>, pos: Size) -> Self {
        Self {
            kind: InternalMessageKind::PlayerMove,
            player: player,
            position: Some(pos),
        }
    }
    pub fn new_leave(player: Arc<Mutex<Player>>) -> Self {
        Self {
            kind: InternalMessageKind::PlayerLeave,
            player: player,
            position: None,
        }
    }
}

pub(crate) enum InternalMessageKind {
    PlayerJoin,
    PlayerMove,
    PlayerLeave,
}

#[derive(Serialize)]
pub(crate) struct Status {
    pub status: String,
    pub details: String,
}

impl Status {
    pub fn new(status: impl Into<String>, details: impl Into<String>) -> Self {
        Self {
            status: status.into(),
            details: details.into(),
        }
    }
}

impl From<Status> for String {
    fn from(value: Status) -> Self {
        serde_json::to_string(&value).unwrap()
    }
}
