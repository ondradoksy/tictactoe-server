use serde::Serialize;

use crate::{ common::Size, player_move::PlayerMove };

#[derive(Serialize, Clone)]
pub(crate) struct Grid {
    pub size: Size,
    moves: Vec<PlayerMove>,
}

impl Grid {
    pub fn new(size: Size) -> Self {
        Self {
            size: size,
            moves: Vec::new(),
        }
    }
    pub fn get_pos(&self, pos: &Size) -> Option<i32> {
        let index = self.get_index(pos);

        if index.is_some() {
            return Some(self.moves[index.unwrap()].player);
        }
        None
    }
    pub fn add_range(&mut self, moves: &Vec<PlayerMove>) {
        self.moves.extend_from_slice(moves.as_slice());
    }
    pub fn add(&mut self, m: PlayerMove) {
        self.moves.push(m);
    }
    fn get_index(&self, pos: &Size) -> Option<usize> {
        let index = self.moves
            .iter()
            .rev()
            .position(|m| m.position == *pos);
        if index.is_none() {
            return None;
        }
        Some(self.moves.len() - index.unwrap() - 1)
    }
    pub fn is_empty(&self, pos: &Size) -> bool {
        self.get_pos(pos).is_none()
    }
    pub fn is_valid_move(&self, pos: &Size) -> bool {
        self.is_empty(pos) && pos.x < self.size.x && pos.y < self.size.y
    }
    /// Returns an Vec of moves that won the game for the player. Will return an empty Vec if the player has not won.
    pub fn check_win(&self, pos: &Size, win_length: u32) -> Vec<PlayerMove> {
        let mut length = [0, 0];

        let player_id = self.get_pos(pos);
        if player_id.is_none() {
            return Vec::new();
        }

        let mut i = 0;
        let mut moves: Vec<PlayerMove> = Vec::new();

        let blocked_id = -2;

        // left
        while i < win_length && pos.x >= i {
            if self.get_pos(&Size::new(pos.x - i, pos.y)) != player_id {
                break;
            }
            length[0] += 1;
            i += 1;
        }
        // right
        i = 1;
        while i <= win_length - length[0] && pos.x + i < self.size.x {
            if self.get_pos(&Size::new(pos.x + i, pos.y)) != player_id {
                break;
            }
            length[1] += 1;
            i += 1;
        }
        if length[0] + length[1] >= win_length {
            for j in 0..length[0] {
                moves.push(PlayerMove::new(blocked_id, Size::new(pos.x - j, pos.y)));
            }
            for j in 0..length[1] {
                moves.push(PlayerMove::new(blocked_id, Size::new(pos.x + j + 1, pos.y)));
            }
            return moves;
        }

        length = [0, 0];

        // up
        i = 0;
        while i < win_length && pos.y >= i {
            if self.get_pos(&Size::new(pos.x, pos.y - i)) != player_id {
                break;
            }
            length[0] += 1;
            i += 1;
        }
        // down
        i = 1;
        while i <= win_length - length[0] && pos.y + i < self.size.y {
            if self.get_pos(&Size::new(pos.x, pos.y + i)) != player_id {
                break;
            }
            length[1] += 1;
            i += 1;
        }
        if length[0] + length[1] >= win_length {
            for j in 0..length[0] {
                moves.push(PlayerMove::new(blocked_id, Size::new(pos.x, pos.y - j)));
            }
            for j in 0..length[1] {
                moves.push(PlayerMove::new(blocked_id, Size::new(pos.x, pos.y + j + 1)));
            }
            return moves;
        }

        length = [0, 0];

        // up-left
        i = 0;
        while i < win_length && pos.x >= i && pos.y >= i {
            if self.get_pos(&Size::new(pos.x - i, pos.y - i)) != player_id {
                break;
            }
            length[0] += 1;
            i += 1;
        }
        // down-right
        i = 1;
        while i <= win_length - length[0] && pos.x + i < self.size.x && pos.y + i < self.size.y {
            if self.get_pos(&Size::new(pos.x + i, pos.y + i)) != player_id {
                break;
            }
            length[1] += 1;
            i += 1;
        }
        if length[0] + length[1] >= win_length {
            for j in 0..length[0] {
                moves.push(PlayerMove::new(blocked_id, Size::new(pos.x - j, pos.y - j)));
            }
            for j in 0..length[1] {
                moves.push(PlayerMove::new(blocked_id, Size::new(pos.x + j + 1, pos.y + j + 1)));
            }
            return moves;
        }

        length = [0, 0];
        i = 0;

        // down-left
        while i < win_length && pos.x >= i && pos.y + i < self.size.y {
            if self.get_pos(&Size::new(pos.x - i, pos.y + i)) != player_id {
                break;
            }
            length[0] += 1;
            i += 1;
        }
        // up-right
        i = 1;
        while i <= win_length - length[0] && pos.x + i < self.size.x && pos.y >= i {
            if self.get_pos(&Size::new(pos.x + i, pos.y - i)) != player_id {
                break;
            }
            length[1] += 1;
            i += 1;
        }
        if length[0] + length[1] >= win_length {
            for j in 0..length[0] {
                moves.push(PlayerMove::new(blocked_id, Size::new(pos.x - j, pos.y + j)));
            }
            for j in 0..length[1] {
                moves.push(PlayerMove::new(blocked_id, Size::new(pos.x + j + 1, pos.y - j - 1)));
            }
            return moves;
        }

        Vec::new()
    }
    pub fn get_possible_moves(&self, id: i32) -> Vec<PlayerMove> {
        let mut moves = Vec::new();
        for i in 0..self.size.x {
            for j in 0..self.size.y {
                let pos = Size::new(i, j);
                if self.is_empty(&pos) {
                    moves.push(PlayerMove::new(id, pos));
                }
            }
        }
        moves
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
