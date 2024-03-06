use std::{ sync::{ Mutex, Arc, mpsc::{ self, Sender, Receiver } }, thread::spawn };

use serde::Serialize;
use crate::{
    grid::Grid,
    net::{ GameCreationData, InternalMessage, InternalMessageKind, MessageEvent, Status },
    player::Player,
    common::{ Size, get_object },
    player_move::PlayerMove,
};

#[derive(Serialize)]
pub(crate) struct Game {
    pub id: u32,
    #[serde(skip_serializing)]
    grid: Grid,
    #[serde(skip_serializing)]
    pub tx: Sender<InternalMessage>,
    player_list: Vec<i32>,
    creator: i32,
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
        creator: &Arc<Mutex<Player>>,
        players: &Arc<Mutex<Vec<Arc<Mutex<Player>>>>>
    ) -> Arc<Mutex<Self>> {
        let (tx, rx) = mpsc::channel::<InternalMessage>();
        let mut id_counter_locked = game_id_counter.lock().unwrap();
        let instance = Self {
            id: *id_counter_locked,
            grid: Grid::new(parameters.size),
            tx: tx,
            player_list: Vec::new(),
            creator: creator.lock().unwrap().id,
            current_turn: 0,
            hotjoin: parameters.hotjoin,
            player_limit: parameters.player_limit,
            running: false,
            length_to_win: parameters.length_to_win,
        };
        *id_counter_locked += 1;
        let self_arc = Arc::new(Mutex::new(instance));
        let players_clone = players.clone();
        spawn({
            let self_arc_clone = Arc::clone(&self_arc);
            move || {
                Self::run(self_arc_clone, rx, players_clone);
            }
        });
        self_arc
    }

    pub fn run(
        game: Arc<Mutex<Game>>,
        rx: Receiver<InternalMessage>,
        players: Arc<Mutex<Vec<Arc<Mutex<Player>>>>>
    ) {
        for msg in rx.iter() {
            match msg.kind {
                InternalMessageKind::PlayerJoin => {
                    game.lock().unwrap().handle_player_join(&game, &msg.player);
                }
                InternalMessageKind::PlayerMove => {
                    let mut game_guard = game.lock().unwrap();
                    let m = PlayerMove::new(msg.player.lock().unwrap().id, msg.position.unwrap());
                    game_guard.broadcast_move(&m, &players);
                    game_guard.grid.add(m.clone());

                    let moves = game_guard.grid.check_win(
                        &m.position.clone(),
                        game_guard.length_to_win
                    );
                    if moves.len() > 0 {
                        for mv in moves {
                            game_guard.broadcast_move(&mv, &players);
                            game_guard.grid.add(mv);
                        }
                        // TODO: Implement score
                    }

                    game_guard.next_turn(&players);
                }
                InternalMessageKind::PlayerLeave => {
                    game.lock().unwrap().remove_player(&msg.player, &players);
                }
                InternalMessageKind::PlayerReady => {
                    let mut game_guard = game.lock().unwrap();
                    if game_guard.can_start(&players) {
                        game_guard.start(&players);
                    }
                }
            }
            println!("message");
        }
    }
    fn can_start(&self, players: &Arc<Mutex<Vec<Arc<Mutex<Player>>>>>) -> bool {
        for p_id in &self.player_list {
            let player = get_object(&players, |p| { &p.lock().unwrap().id == p_id });
            if !player.expect("This should never happen").lock().unwrap().ready {
                return false;
            }
        }
        true
    }
    fn start(&mut self, players: &Arc<Mutex<Vec<Arc<Mutex<Player>>>>>) {
        self.running = true;
        // Make sure to send current_state first to avoid breaking client
        self.broadcast(&MessageEvent::new("current_state", self.grid.clone()), players);
        self.broadcast_turn(players);
    }
    fn handle_player_join(&mut self, self_arc: &Arc<Mutex<Self>>, player: &Arc<Mutex<Player>>) {
        let mut player_guard = player.lock().unwrap();
        player_guard.joined_game = Some(self_arc.clone());
        player_guard.joined_game_id = Some(self.id);
        self.player_list.push(player_guard.id);
        drop(player_guard);
        if self.running {
            Self::send_to_player_arc(
                player,
                &MessageEvent::new("current_state", self.grid.clone())
            );
        }
    }
    pub fn ready_toggle(&self, player: &Arc<Mutex<Player>>) -> Status {
        if self.running {
            return Status::new("error", "Game is already running.");
        }
        let mut player_guard = player.lock().unwrap();
        player_guard.ready = !player_guard.ready;
        if player_guard.ready {
            self.tx.send(InternalMessage::new_ready(player.clone())).unwrap();
        }
        Status::new("ok", "")
    }
    fn send_to_player(player: &Player, msg: &MessageEvent) {
        player.tx.send(msg.clone()).unwrap();
    }
    fn send_to_player_arc(player: &Arc<Mutex<Player>>, msg: &MessageEvent) {
        let player_guard = player.lock().unwrap();
        Self::send_to_player(&player_guard, msg);
    }
    fn send_to_player_id(
        &self,
        player_id: i32,
        msg: &MessageEvent,
        players: &Arc<Mutex<Vec<Arc<Mutex<Player>>>>>
    ) {
        let player = get_object(players, |p| { p.lock().unwrap().id == player_id }).expect(
            "Unable to find player"
        );
        Self::send_to_player_arc(&player, msg);
    }

    fn broadcast(&self, msg: &MessageEvent, players: &Arc<Mutex<Vec<Arc<Mutex<Player>>>>>) {
        for player in &self.player_list {
            self.send_to_player_id(*player, msg, players);
        }
    }

    fn broadcast_turn(&self, players: &Arc<Mutex<Vec<Arc<Mutex<Player>>>>>) {
        self.broadcast(
            &MessageEvent::new("turn", serde_json::to_string(&self.current_turn).unwrap()),
            players
        );
    }

    fn broadcast_move(&self, m: &PlayerMove, players: &Arc<Mutex<Vec<Arc<Mutex<Player>>>>>) {
        self.broadcast(&MessageEvent::new("new_move", serde_json::to_string(m).unwrap()), players);
    }

    fn remove_player(
        &mut self,
        player: &Arc<Mutex<Player>>,
        players: &Arc<Mutex<Vec<Arc<Mutex<Player>>>>>
    ) {
        let id = player.lock().unwrap().id;
        let index_option = self.player_list.iter().position(|p| { p == &id });
        if index_option.is_none() {
            println!("Player {} not found in game {}.", player.lock().unwrap().id, self.id);
            return;
        }

        let index = index_option.unwrap();
        if index < self.current_turn {
            self.current_turn -= 1;
        }

        player.lock().unwrap().joined_game = None;
        player.lock().unwrap().joined_game_id = None;

        self.player_list.remove(index);
        self.broadcast_turn(players);
    }

    fn next_turn(&mut self, players: &Arc<Mutex<Vec<Arc<Mutex<Player>>>>>) {
        self.current_turn += 1;
        if self.current_turn >= self.player_list.len() {
            self.current_turn = 0;
        }
        self.broadcast_turn(players);
    }

    pub fn join_player(&self, player: &Arc<Mutex<Player>>) -> bool {
        if
            (self.running && !self.hotjoin) ||
            self.player_list.len() >= self.player_limit ||
            self.player_list.contains(&player.lock().unwrap().id)
        {
            return false;
        }

        self.tx.send(InternalMessage::new_join(player.clone())).unwrap();
        true
    }

    pub fn add_move(&self, player: &Arc<Mutex<Player>>, pos: Size) -> bool {
        if
            !self.running ||
            !self.grid.is_valid_move(&pos) ||
            player.lock().unwrap().id != self.player_list[self.current_turn] // This was observed go out of bounds if player leaves, unable to reproduce again
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
    let players_all: Arc<Mutex<Vec<Arc<Mutex<Player>>>>> = Arc::new(Mutex::new(Vec::new()));
    let (tx, _rx) = mpsc::channel();
    let player = Arc::new(Mutex::new(Player::new(0, tx)));
    players_all.lock().unwrap().push(player.clone());
    let game_id_counter = Arc::new(Mutex::new(0));
    let game_parameters = GameCreationData::new(Size::new(5, 5), true, 10, 4);
    let game = Game::new(&game_parameters, &game_id_counter, &player.clone(), &players_all);

    let mut players: Vec<Arc<Mutex<Player>>> = Vec::new();
    for i in 1..10 {
        let (tx, _rx) = mpsc::channel();
        let p = Arc::new(Mutex::new(Player::new(i, tx)));
        players.push(p.clone());
        players_all.lock().unwrap().push(p);
        game.lock().unwrap().join_player(&players.last().unwrap());
    }

    // TODO: finish test
}
