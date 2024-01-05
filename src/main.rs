mod grid;
mod game;
mod player;
mod net;
mod common;
mod player_move;

use std::{ io, env };
use std::net::{ TcpListener, TcpStream };
use std::sync::{ Mutex, Arc, mpsc };
use std::thread::spawn;
use std::time::Duration;
use game::Game;
use tungstenite::accept;
use crate::common::{ Size, get_object };
use crate::player::Player;
use crate::net::{ MessageEvent, GameCreationData, Status, GameJoinData };

/// A WebSocket echo server
fn main() {
    let args: Vec<String> = env::args().collect();
    let listen_ip = if args.len() > 1 { args[1].as_str() } else { "0.0.0.0:9001" };

    let server = TcpListener::bind(listen_ip).unwrap();
    println!("Listening on {}", listen_ip);

    let mut player_id_counter: u32 = 0;
    let players: Arc<Mutex<Vec<Arc<Mutex<Player>>>>> = Arc::new(
        Mutex::new(Vec::<Arc<Mutex<Player>>>::new())
    );
    let games: Arc<Mutex<Vec<Arc<Mutex<Game>>>>> = Arc::new(
        Mutex::new(Vec::<Arc<Mutex<Game>>>::new())
    );
    let game_id_counter: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));

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
    unique_id: u32,
    players: Arc<Mutex<Vec<Arc<Mutex<Player>>>>>,
    games: Arc<Mutex<Vec<Arc<Mutex<Game>>>>>,
    game_id_counter: Arc<Mutex<u32>>
) {
    let addr = get_addr(&stream);
    println!("New connection: {}", addr);

    // Prevent read from blocking forever
    stream.set_read_timeout(Some(Duration::from_millis(100))).unwrap();

    // Create channel
    let (tx, rx) = mpsc::channel();

    let player_arc = Arc::new(Mutex::new(Player::new(unique_id, tx)));

    // Add new player to list
    players.lock().unwrap().push(player_arc.clone());

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
                        p.lock()
                            .unwrap()
                            .tx.send(MessageEvent::new(&String::from("broadcast"), &json))
                            .unwrap();
                    }
                }
                // Create new game
                "create_game" => {
                    let json = event.content;
                    let game_parameters = GameCreationData::from_json(json.as_str());
                    if game_parameters.is_ok() {
                        let game = Game::new(
                            &game_parameters.unwrap(),
                            &game_id_counter,
                            &player_arc
                        );
                        games.lock().unwrap().push(game);
                        response = MessageEvent::new("create_game", Status::new("ok", ""));
                    } else {
                        response = MessageEvent::new(
                            event.event,
                            Status::new("error", game_parameters.err().unwrap().to_string())
                        );
                    }
                }
                // Get game list
                "games" => {
                    let json = serde_json::to_string(&*games.lock().unwrap()).unwrap();
                    response = MessageEvent::new(&String::from("games"), &json);
                }
                // Join game
                "join_game" => {
                    let join_data = GameJoinData::from_json(&event.content);
                    if join_data.is_ok() {
                        let id = join_data.unwrap().id;
                        let game = get_object(&games, |p| p.lock().unwrap().id == id);

                        // Check if game exists
                        if game.is_some() {
                            game.unwrap().lock().unwrap().join_player(&player_arc);
                        } else {
                            response = MessageEvent::new(
                                event.event,
                                Status::new("error", "Game does not exist.")
                            );
                        }
                    } else {
                        response = MessageEvent::new(
                            event.event,
                            Status::new("error", join_data.err().unwrap().to_string())
                        );
                    }
                }
                // Make move
                "move" => {
                    let json = event.content;
                    let position = Size::from_json(&json);

                    // Check if json is valid
                    if position.is_ok() {
                        let game = &player_arc.lock().unwrap().joined_game;

                        // Check player is in a game
                        if game.is_some() {
                            let game_locked = game.as_ref().unwrap().lock().unwrap();
                            if game_locked.add_move(&player_arc, position.unwrap()) {
                                response = MessageEvent::new(event.event, Status::new("ok", ""));
                            } else {
                                response = MessageEvent::new(
                                    event.event,
                                    Status::new("error", "Move not allowed.")
                                );
                            }
                        } else {
                            response = MessageEvent::new(
                                event.event,
                                Status::new("error", "You are not in a game.")
                            );
                        }
                    } else {
                        response = MessageEvent::new(
                            event.event,
                            Status::new("error", position.err().unwrap().to_string())
                        );
                    }
                }
                "ready" => {
                    let game = &player_arc.lock().unwrap().joined_game;

                    // Check player is in a game
                    if game.is_some() {
                        game.as_ref().unwrap().lock().unwrap().ready_toggle(&player_arc);
                    } else {
                        response = MessageEvent::new(
                            event.event,
                            Status::new("error", "You are not in a game.")
                        );
                    }
                }
                _ => {
                    response = MessageEvent::new(
                        event.event,
                        Status::new("error", "Unknown event.")
                    );
                }
            }

            // Respond to current request first (might be best to remove in the future)
            if !response.is_empty() {
                websocket.send(response.to_message()).unwrap();
            }
        } else {
            println!("{} - {}", addr, result.err().unwrap());
        }
    }

    let player_guard = player_arc.lock().unwrap();
    if player_guard.joined_game.is_some() {
        player_guard.joined_game.as_ref().unwrap().lock().unwrap().leave_player(&player_arc);
    }

    // Remove player from list
    println!("Removing player {}", unique_id);
    let mut players_locked = players.lock().unwrap();
    let index = players_locked
        .iter()
        .position(|p| p.lock().unwrap().id == unique_id)
        .unwrap();
    players_locked.swap_remove(index);
    drop(players_locked);

    // Properly close the connection
    println!("Closing connection {}", addr);
    let close = websocket.close(None);
    if close.is_err() {
        println!("{}", close.err().unwrap().to_string());
    } else {
        println!("Connection closed");
    }
}

fn get_addr(stream: &TcpStream) -> String {
    stream.peer_addr().unwrap().to_string()
}
