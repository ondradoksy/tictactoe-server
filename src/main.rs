mod grid;
mod game;
mod player;
mod net;

use std::net::{ TcpListener, TcpStream };
use std::sync::{ Mutex, Arc };
use std::thread::spawn;
use tungstenite::{ accept, Message };
use crate::player::Player;
use crate::net::MessageEvent;

/// A WebSocket echo server
fn main() {
    let server = TcpListener::bind("0.0.0.0:9001").unwrap();
    let mut id_counter: u64 = 0;
    let players: Arc<Mutex<Vec<Player>>> = Arc::new(Mutex::new(Vec::<Player>::new()));

    // This will be useful
    //let (tx, rx) = mpsc::channel();

    for stream in server.incoming() {
        spawn({
            let players_clone = Arc::clone(&players);
            move || {
                handle_connection(stream.unwrap(), id_counter, players_clone);
            }
        });
        id_counter += 1;
    }
}

fn handle_connection(stream: TcpStream, unique_id: u64, players: Arc<Mutex<Vec<Player>>>) {
    let addr = get_addr(&stream);
    println!("New connection: {}", addr);

    let player = Player::new(unique_id);
    players.lock().unwrap().push(player);

    let mut websocket = accept(stream).unwrap();

    loop {
        let message = websocket.read();

        if message.is_err() {
            break;
        }

        let event: MessageEvent = MessageEvent::from_message(message.unwrap());
        println!("{} - {}", addr, event.event);

        let mut response = MessageEvent::new_empty();

        match event.event.as_str() {
            "players" => {
                let json = serde_json::to_string(&*players.lock().unwrap()).unwrap();
                response = MessageEvent::new(&String::from("players"), &json);
            }
            _ => {}
        }

        websocket
            .send(Message::Text(String::from(serde_json::to_string(&response).unwrap())))
            .unwrap();
    }

    println!("Removing player {}", unique_id);
    let mut players_locked = players.lock().unwrap();
    let index = players_locked
        .iter()
        .position(|p| p.id == unique_id)
        .unwrap();
    players_locked.swap_remove(index);
    drop(players_locked);

    println!("Closing connection {}", addr);
    let close = websocket.close(None);
    if close.is_err() {
        println!("Connection closed by client");
    } else {
        println!("Connection closed by server");
    }
}

fn get_addr(stream: &TcpStream) -> String {
    stream.peer_addr().unwrap().to_string()
}

fn handle_players() {}
