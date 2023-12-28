mod grid;
mod game;
mod player;
mod net;

use std::io;
use std::net::{ TcpListener, TcpStream };
use std::sync::{ Mutex, Arc, mpsc };
use std::thread::spawn;
use std::time::Duration;
use game::Game;
use tungstenite::accept;
use crate::player::Player;
use crate::net::{ MessageEvent, GameParameters };

/// A WebSocket echo server
fn main() {
    let server = TcpListener::bind("0.0.0.0:9001").unwrap();
    let mut player_id_counter: u64 = 0;
    let players: Arc<Mutex<Vec<Player>>> = Arc::new(Mutex::new(Vec::<Player>::new()));
    let games: Arc<Mutex<Vec<Game>>> = Arc::new(Mutex::new(Vec::<Game>::new()));
    let game_id_counter: Arc<Mutex<u64>> = Arc::new(Mutex::new(0));

    for stream in server.incoming() {
        spawn({
            let players_clone = Arc::clone(&players);
            let games_clone = Arc::clone(&games);
            let game_id_counter_clone = Arc::clone(&game_id_counter);
            move || {
                handle_connection(
                    stream.unwrap(),
                    player_id_counter,
                    players_clone,
                    games_clone,
                    game_id_counter_clone
                );
            }
        });
        player_id_counter += 1;
    }
}

fn handle_connection(
    stream: TcpStream,
    unique_id: u64,
    players: Arc<Mutex<Vec<Player>>>,
    games: Arc<Mutex<Vec<Game>>>,
    game_id_counter: Arc<Mutex<u64>>
) {
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
                // Get player list
                "players" => {
                    let json = serde_json::to_string(&*players.lock().unwrap()).unwrap();
                    response = MessageEvent::new(&String::from("players"), &json);
                }
                // Broadcast message to all players
                "broadcast" => {
                    let json = event.content;
                    for p in players.lock().unwrap().iter() {
                        p.tx.send(MessageEvent::new(&String::from("broadcast"), &json)).unwrap();
                    }
                }
                // Create new game
                "create_game" => {
                    let json = event.content;
                    let game_parameters = GameParameters::from_json(json.as_str());
                    if game_parameters.is_ok() {
                        let game = Game::new(game_parameters.unwrap().size, &game_id_counter);
                        games.lock().unwrap().push(game);
                    }
                }
                // Get game list
                "games" => {
                    let json = serde_json::to_string(&*games.lock().unwrap()).unwrap();
                    response = MessageEvent::new(&String::from("games"), &json);
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
