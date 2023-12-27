mod grid;
mod game;
mod player;
mod net;

use std::io;
use std::net::{ TcpListener, TcpStream };
use std::sync::{ Mutex, Arc, mpsc };
use std::thread::spawn;
use std::time::Duration;
use tungstenite::accept;
use crate::player::Player;
use crate::net::MessageEvent;

/// A WebSocket echo server
fn main() {
    let server = TcpListener::bind("0.0.0.0:9001").unwrap();
    let mut id_counter: u64 = 0;
    let players: Arc<Mutex<Vec<Player>>> = Arc::new(Mutex::new(Vec::<Player>::new()));

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

    // Prevent read from blocking forever
    stream.set_read_timeout(Some(Duration::from_millis(100))).unwrap();

    // Create channel
    let (tx, rx) = mpsc::channel();

    // Add new player to list
    players.lock().unwrap().push(Player::new(unique_id, tx));

    let mut websocket = accept(stream).unwrap();

    loop {
        // Process queue
        let result = rx.try_recv();
        if result.is_ok() {
            websocket.send(result.unwrap().to_message()).unwrap();
        }

        let message = websocket.read();

        if message.is_err() {
            match message {
                Err(tungstenite::Error::Io(err)) if err.kind() == io::ErrorKind::WouldBlock => {
                    continue;
                }
                _ => {
                    break;
                }
            }
        }

        // Parse JSON
        let result = MessageEvent::from_message(message.unwrap());

        if result.is_ok() {
            let event: MessageEvent = result.unwrap();

            println!("{} - {}", addr, event.event);

            let mut response = MessageEvent::new_empty();

            // Process events
            match event.event.as_str() {
                "players" => {
                    let json = serde_json::to_string(&*players.lock().unwrap()).unwrap();
                    response = MessageEvent::new(&String::from("players"), &json);
                }
                "broadcast" => {
                    let json = event.content;
                    for p in players.lock().unwrap().iter() {
                        p.tx.send(MessageEvent::new(&String::from("broadcast"), &json)).unwrap();
                    }
                }
                _ => {}
            }

            // Respond to current request first (might be best to remove in the future)
            if !response.is_empty() {
                websocket.send(response.to_message()).unwrap();
            }
        } else {
            println!("{} - {}", addr, result.err().unwrap());
        }
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
