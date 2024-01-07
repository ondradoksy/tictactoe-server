use std::{ sync::{ Mutex, Arc, mpsc::{ self, Sender, Receiver } }, thread::spawn };

use serde::Serialize;
use crate::{
    grid::Grid,
    net::{ InternalMessage, InternalMessageKind, MessageEvent, GameCreationData },
    player::Player,
    common::{ Size, find_index },
    player_move::PlayerMove,
};

#[derive(Serialize)]
pub(crate) struct Game {
    pub id: u32,
    #[serde(skip_serializing)]
    grid: Grid,
    #[serde(skip_serializing)]
    pub tx: Sender<InternalMessage>,
    player_list: Vec<Arc<Mutex<Player>>>,
    creator: Arc<Mutex<Player>>,
    current_turn: usize,
    hotjoin: bool,
    player_limit: usize,
    running: bool,
    length_to_win: u32,
}
impl Game {
    pub fn new(
        parameters: &GameCreationData,
        game_id_counter: &Arc<Mutex<u32>>,
        creator: &Arc<Mutex<Player>>
    ) -> Arc<Mutex<Self>> {
        let (tx, rx) = mpsc::channel::<InternalMessage>();
        let mut id_counter_locked = game_id_counter.lock().unwrap();
        let instance = Self {
            id: *id_counter_locked,
            grid: Grid::new(parameters.size),
            tx: tx,
            player_list: Vec::new(),
            creator: creator.clone(),
            current_turn: 0,
            hotjoin: parameters.hotjoin,
            player_limit: parameters.player_limit,
            running: false,
            length_to_win: parameters.length_to_win,
        };
        *id_counter_locked += 1;
        let arc = Arc::new(Mutex::new(instance));
        spawn({
            let arc_clone = Arc::clone(&arc);
            move || {
                Self::run(arc_clone, rx);
            }
        });
        arc
    }

    pub fn run(game: Arc<Mutex<Game>>, rx: Receiver<InternalMessage>) {
        for msg in rx.iter() {
            match msg.kind {
                InternalMessageKind::PlayerJoin => {
                    let mut player_guard = msg.player.lock().unwrap();
                    player_guard.joined_game = Some(game.clone());
                    player_guard.joined_game_id = Some(game.lock().unwrap().id);
                    drop(player_guard);

                    game.lock().unwrap().player_list.push(msg.player);
                }
                InternalMessageKind::PlayerMove => {
                    let mut game_guard = game.lock().unwrap();
                    let m = PlayerMove::new(msg.player.lock().unwrap().id, msg.position.unwrap());
                    game_guard.broadcast_move(&m);
                    game_guard.grid.add(m);
                    game_guard.next_turn();
                }
                InternalMessageKind::PlayerLeave => {
                    game.lock().unwrap().remove_player(&mut msg.player.lock().unwrap());
                }
                InternalMessageKind::PlayerReady => {
                    let mut game_guard = game.lock().unwrap();
                    if game_guard.can_start() {
                        game_guard.start();
                    }
                }
            }
            println!("message");
        }
    }
    fn can_start(&self) -> bool {
        for player in &self.player_list {
            if !player.lock().unwrap().ready {
                return false;
            }
        }
        true
    }
    fn start(&mut self) {
        self.running = true;
        self.broadcast_turn();
    }
    pub fn ready_toggle(&self, player: &Arc<Mutex<Player>>) {
        let mut player_guard = player.lock().unwrap();
        player_guard.ready = !player_guard.ready;
        if player_guard.ready {
            self.tx.send(InternalMessage::new_ready(player.clone())).unwrap();
        }
    }

    fn send_to_player(&self, player: &Player, msg: &MessageEvent) {
        player.tx.send(msg.clone()).unwrap();
    }

    fn broadcast(&self, msg: &MessageEvent) {
        for player in &self.player_list {
            self.send_to_player(&player.lock().unwrap(), msg);
        }
    }

    fn broadcast_turn(&self) {
        self.broadcast(
            &MessageEvent::new("turn", serde_json::to_string(&self.current_turn).unwrap())
        );
    }

    fn broadcast_move(&self, m: &PlayerMove) {
        self.broadcast(&MessageEvent::new("move", serde_json::to_string(m).unwrap()))
    }

    fn remove_player(&mut self, player: &mut Player) {
        let index_option = find_index(&self.player_list, |p| *p.lock().unwrap() == *player);
        if index_option.is_none() {
            println!("Player {} not found in game {}.", player.id, self.id);
            return;
        }

        let index = index_option.unwrap();
        if index > self.current_turn {
            self.current_turn -= 1;
        }
        self.player_list.remove(index);
        self.broadcast_turn();
        player.joined_game = None;
        player.joined_game_id = None;
    }

    fn next_turn(&mut self) {
        self.current_turn += 1;
        if self.current_turn >= self.player_list.len() {
            self.current_turn = 0;
        }
        self.broadcast_turn();
    }

    pub fn join_player(&self, player: &Arc<Mutex<Player>>) -> bool {
        if (self.running && !self.hotjoin) || self.player_list.len() == self.player_limit {
            return false;
        }

        self.tx.send(InternalMessage::new_join(player.clone())).unwrap();
        true
    }

    pub fn add_move(&self, player: &Arc<Mutex<Player>>, pos: Size) -> bool {
        if
            !self.grid.is_valid_move(&pos) ||
            *player.lock().unwrap() != *self.player_list[self.current_turn].lock().unwrap()
        {
            return false;
        }

        self.tx.send(InternalMessage::new_move(player.clone(), pos)).unwrap();
        true
    }
    pub fn leave_player(&self, player: &Arc<Mutex<Player>>) {
        self.tx.send(InternalMessage::new_leave(player.clone())).unwrap();
    }
}

#[test]
fn player_join() {
    let (tx, _rx) = mpsc::channel();
    let player = Arc::new(Mutex::new(Player::new(0, tx)));
    let game_id_counter = Arc::new(Mutex::new(0));
    let game_parameters = GameCreationData::new(Size::new(5, 5), true, 10, 4);
    let game = Game::new(&game_parameters, &game_id_counter, &player.clone());

    let mut players: Vec<Arc<Mutex<Player>>> = Vec::new();
    for i in 1..10 {
        let (tx, _rx) = mpsc::channel();
        players.push(Arc::new(Mutex::new(Player::new(i, tx))));
        game.lock().unwrap().join_player(&players.last().unwrap());
    }

    // TODO: finish test
}
