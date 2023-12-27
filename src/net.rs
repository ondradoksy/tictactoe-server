use serde::{ Serialize, Deserialize };
use tungstenite::Message;

#[derive(Deserialize, Serialize)]
pub(crate) struct MessageEvent {
    pub event: String,
    pub content: String,
}
impl MessageEvent {
    pub fn new(event: &String, content: &String) -> Self {
        Self {
            event: event.to_string(),
            content: content.to_string(),
        }
    }
    pub fn new_empty() -> Self {
        Self {
            event: String::from(""),
            content: String::from(""),
        }
    }
    pub fn from_message(message: Message) -> Result<Self, &'static str> {
        Self::from_json(message.to_text().unwrap())
    }
    pub fn from_json(text: &str) -> Result<Self, &'static str> {
        let result: Result<Self, serde_json::Error> = serde_json::from_str(text);
        if result.is_ok() {
            return Ok(result.unwrap());
        }
        Err("Malformed JSON")
    }
    pub fn to_message(&self) -> Message {
        Message::Text(String::from(serde_json::to_string(&self).unwrap()))
    }
    pub fn is_empty(&self) -> bool {
        self.event.as_str() == ""
    }
}
