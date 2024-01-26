use serde::Serialize;

use crate::{ common::Size, player_move::PlayerMove };

#[derive(Serialize, Clone)]
pub(crate) struct Grid {
    size: Size,
    moves: Vec<PlayerMove>,
}

impl Grid {
    pub fn new(size: Size) -> Self {
        Self {
            size: size,
            moves: Vec::new(),
        }
    }
    pub fn get_pos(&self, pos: &Size) -> Option<u32> {
        let index = self.get_index(pos);

        if index.is_some() {
            return Some(self.moves[index.unwrap()].player);
        }
        None
    }
    pub fn add(&mut self, m: PlayerMove) {
        self.moves.push(m);
    }
    fn get_index(&self, pos: &Size) -> Option<usize> {
        self.moves
            .iter()
            .rev()
            .position(|m| m.position == *pos)
    }
    pub fn is_empty(&self, pos: &Size) -> bool {
        self.get_pos(pos).is_none()
    }
    pub fn is_valid_move(&self, pos: &Size) -> bool {
        self.is_empty(pos) && pos.x < self.size.x && pos.y < self.size.y
    }
}

impl From<Grid> for String {
    fn from(value: Grid) -> Self {
        serde_json::to_string(&value).expect("Unable to serialize")
    }
}

#[test]
fn test_grid() {
    let mut grid = Grid::new(Size::new(3, 3));

    grid.add(PlayerMove::new(1000, Size::new(1, 1)));
    grid.add(PlayerMove::new(1001, Size::new(2, 0)));
    grid.add(PlayerMove::new(1000, Size::new(0, 2)));

    assert_eq!(grid.get_pos(&Size::new(0, 0)), None);
    assert_eq!(grid.get_pos(&Size::new(1, 1)), Some(1000));
    assert_eq!(grid.get_pos(&Size::new(2, 0)), Some(1001));
    assert_eq!(grid.get_pos(&Size::new(0, 2)), Some(1000));
}
