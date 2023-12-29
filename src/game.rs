use std::{ sync::{ Mutex, Arc, mpsc::{ self, Sender, Receiver } }, thread::spawn };

use serde::Serialize;
use crate::{ grid::Grid, net::{ MessageEvent, InternalMessage }, player::Player, common::Size };

#[derive(Serialize)]
pub(crate) struct Game {
    id: u32,
    #[serde(skip_serializing)]
    grid: Grid,
    #[serde(skip_serializing)]
    pub tx: Sender<InternalMessage>,
    player_list: Vec<Player>,
}
impl Game {
    pub fn new(size: Size, game_id_counter: &Arc<Mutex<u32>>) -> Arc<Mutex<Self>> {
        let (tx, rx) = mpsc::channel::<InternalMessage>();
        let mut id_counter_locked = game_id_counter.lock().unwrap();
        let instance = Self {
            id: *id_counter_locked,
            grid: Grid::new(size),
            tx: tx,
            player_list: Vec::new(),
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
            println!("message");
        }
    }
}
