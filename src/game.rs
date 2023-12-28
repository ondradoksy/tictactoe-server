use std::sync::{ Mutex, Arc };

use serde::Serialize;
use crate::grid::Grid;

#[derive(Serialize)]
pub(crate) struct Game {
    id: u64,
    #[serde(skip_serializing)]
    grid: Grid,
}
impl Game {
    pub fn new(size: (u32, u32), game_id_counter: &Arc<Mutex<u64>>) -> Self {
        Game {
            id: *game_id_counter.lock().unwrap(),
            grid: Grid::new(size),
        }
    }
}
