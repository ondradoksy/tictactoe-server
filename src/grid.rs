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
    pub fn get_pos(&self, pos: (u32, u32)) -> u64 {
        let index = self.moves.iter().position(|m| m.position == pos);

        if index.is_some() {
            return self.moves[index.unwrap()].player;
        }
        0
    }
    pub fn add(&mut self, player: u64, pos: (u32, u32)) {
        self.moves.push(Move::new(player, pos));
    }
}

struct Move {
    player: u64,
    position: (u32, u32),
}
impl Move {
    pub fn new(player: u64, pos: (u32, u32)) -> Self {
        Self {
            player: player,
            position: pos,
        }
    }
}
