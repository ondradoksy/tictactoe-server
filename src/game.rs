use crate::grid::Grid;

struct Game {
    grid: Grid,
}
impl Game {
    pub fn new(size: (u32, u32)) -> Self {
        Game {
            grid: Grid::new(size),
        }
    }
}
