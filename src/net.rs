use std::ops::Deref;

use serde::{ Serialize, Deserialize };
use tungstenite::Message;

use crate::common::Size;

#[derive(Deserialize, Serialize)]
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
pub(crate) struct GameParameters {
    pub size: Size,
}
impl GameParameters {
    pub fn from_json(text: &str) -> Result<Self, String> {
        from_json(text)
    }
}

pub(crate) struct InternalMessage {
    pub message_type: InternalMessageType,
}

pub(crate) enum InternalMessageType {
    placeholder1,
    placeholder2,
}

fn from_json<T>(text: &str) -> Result<T, String> where T: serde::de::DeserializeOwned {
    let result: Result<T, serde_json::Error> = serde_json::from_str(text);
    if result.is_ok() {
        return Ok(result.unwrap());
    }
    let err_string = result.err().unwrap().to_string();
    Err(err_string)
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
