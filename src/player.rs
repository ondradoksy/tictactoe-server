use std::sync::mpsc::Sender;

use serde::Serialize;

use crate::net::MessageEvent;

#[derive(Serialize)]
pub(crate) struct Player {
    pub id: u32,
    #[serde(skip_serializing)]
    pub tx: Sender<MessageEvent>,
}
impl Player {
    pub fn new(id: u32, tx: Sender<MessageEvent>) -> Self {
        Self {
            id: id,
            tx: tx,
        }
    }
}
