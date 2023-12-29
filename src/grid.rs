pub(crate) struct Grid {
    size: (u32, u32),
    moves: Vec<Move>,
}

impl Grid {
    pub fn new(size: (u32, u32)) -> Self {
        Self {
            size: size,
            moves: Vec::new(),
        }
    }
    pub fn get_pos(&self, pos: (u32, u32)) -> u32 {
        let index = self.get_index(pos);

        if index.is_some() {
            return self.moves[index.unwrap()].player;
        }
        0
    }
    pub fn add(&mut self, player: u32, pos: (u32, u32)) {
        self.moves.push(Move::new(player, pos));
    }
    fn get_index(&self, pos: (u32, u32)) -> Option<usize> {
        self.moves
            .iter()
            .rev()
            .position(|m| m.position == pos)
    }
}

struct Move {
    player: u32,
    position: (u32, u32),
}
impl Move {
    pub fn new(player: u32, pos: (u32, u32)) -> Self {
        Self {
            player: player,
            position: pos,
        }
    }
}

#[test]
fn test_grid() {
    let mut grid = Grid::new((3, 3));

    grid.add(1000, (1, 1));
    grid.add(1001, (2, 0));
    grid.add(1000, (0, 2));

    assert_eq!(grid.get_pos((0, 0)), 0);
    assert_eq!(grid.get_pos((1, 1)), 1000);
    assert_eq!(grid.get_pos((2, 0)), 1001);
    assert_eq!(grid.get_pos((0, 2)), 1000);
}
