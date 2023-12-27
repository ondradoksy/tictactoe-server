use serde::{ Serialize, Deserialize };
use tungstenite::Message;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
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
    pub fn from_message(message: Message) -> Self {
        Self::from_json(message.to_text().unwrap())
    }
    pub fn from_json(text: &str) -> Self {
        serde_json::from_str(text).expect("malformed JSON")
    }
}
