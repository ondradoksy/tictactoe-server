use crate::common::Size;

pub(crate) struct Grid {
    size: Size,
    moves: Vec<Move>,
}

impl Grid {
    pub fn new(size: Size) -> Self {
        Self {
            size: size,
            moves: Vec::new(),
        }
    }
    pub fn get_pos(&self, pos: Size) -> u32 {
        let index = self.get_index(pos);

        if index.is_some() {
            return self.moves[index.unwrap()].player;
        }
        0
    }
    pub fn add(&mut self, player: u32, pos: Size) {
        self.moves.push(Move::new(player, pos));
    }
    fn get_index(&self, pos: Size) -> Option<usize> {
        self.moves
            .iter()
            .rev()
            .position(|m| m.position == pos)
    }
}

struct Move {
    player: u32,
    position: Size,
}
impl Move {
    pub fn new(player: u32, pos: Size) -> Self {
        Self {
            player: player,
            position: pos,
        }
    }
}

#[test]
fn test_grid() {
    let mut grid = Grid::new(Size::new(3, 3));

    grid.add(1000, Size::new(1, 1));
    grid.add(1001, Size::new(2, 0));
    grid.add(1000, Size::new(0, 2));

    assert_eq!(grid.get_pos(Size::new(0, 0)), 0);
    assert_eq!(grid.get_pos(Size::new(1, 1)), 1000);
    assert_eq!(grid.get_pos(Size::new(2, 0)), 1001);
    assert_eq!(grid.get_pos(Size::new(0, 2)), 1000);
}
