use std::net::{ TcpListener, TcpStream };
use std::thread::spawn;
use tungstenite::{ accept, Message };
use serde::{ Serialize, Deserialize };

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
struct MessageEvent {
    event: String,
    content: String,
}
impl MessageEvent {
    pub fn new(event: &String, content: &String) -> Self {
        MessageEvent {
            event: event.to_string(),
            content: content.to_string(),
        }
    }
    pub fn from_message(message: Message) -> Self {
        MessageEvent::from_json(message.to_text().unwrap())
    }
    pub fn from_json(text: &str) -> Self {
        serde_json::from_str(text).expect("malformed JSON")
    }
}

/// A WebSocket echo server
fn main() {
    let server = TcpListener::bind("0.0.0.0:9001").unwrap();
    for stream in server.incoming() {
        spawn(move || {
            handle_connection(stream.unwrap());
        });
    }
}

fn handle_connection(stream: TcpStream) {
    let addr = get_addr(&stream);
    println!("New connection: {}", addr);

    let mut websocket = accept(stream).unwrap();

    loop {
        let message = websocket.read().unwrap();
        let event: MessageEvent = MessageEvent::from_message(message);
        println!("{} - {}", addr, event.event);

        let response = MessageEvent::new(&String::from("ip"), &addr);

        websocket
            .send(Message::Text(String::from(serde_json::to_string(&response).unwrap())))
            .unwrap();
    }

    //websocket.close(None).unwrap();
}

fn get_addr(stream: &TcpStream) -> String {
    stream.peer_addr().unwrap().to_string()
}
