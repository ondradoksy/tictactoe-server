use crate::{ bot::botlogic::BotLogic, game::Game, grid::Grid, player_move::PlayerMove, Size };

use rand::Rng;

use super::random::RandomBot;

pub(crate) struct MCTSBot {}
impl BotLogic for MCTSBot {
    fn generate_move(&self, id: i32, game: &Game) -> Size {
        let mut algorithm = MCTSAlgorithm::new(id, &game.grid, game.win_length);

        let max_iter = 10000;

        for i in 0..max_iter {
            println!("MCTS: Iterating... {}/{}", i + 1, max_iter);
            algorithm.iterate(&game.player_list, game.current_turn, &game.grid, game.win_length);
        }

        for n in algorithm.nodes.iter() {
            println!(
                "  uct_score: {} score: {} visits: {} win_rate: {}",
                n.get_uct_score(max_iter),
                n.score,
                n.visit_counter,
                n.score / (n.visit_counter as f32)
            );
        }

        algorithm.find_best_move()
    }
    fn get_name(&self) -> String {
        "mcts".to_string()
    }
}
impl MCTSBot {
    pub fn new() -> Self {
        Self {}
    }
}
#[derive(Debug)]
struct MCTSAlgorithm {
    nodes: Vec<Node>,
    total_iterations: u32,
}
impl MCTSAlgorithm {
    pub fn new(id: i32, grid: &Grid, win_length: u32) -> Self {
        Self {
            nodes: Node::from_possible_moves(grid.get_possible_moves(id), grid, win_length),
            total_iterations: 0,
        }
    }
    pub fn select(&mut self, parent_visit_counter: u32) -> &mut Node {
        self.nodes
            .iter_mut()
            .max_by(|x, y|
                x
                    .get_uct_score(parent_visit_counter)
                    .total_cmp(&y.get_uct_score(parent_visit_counter))
            )
            .expect("Nothing to select")
    }
    pub fn iterate(
        &mut self,
        players: &Vec<i32>,
        current_turn: usize,
        grid: &Grid,
        win_length: u32
    ) {
        let parent_visits = self.total_iterations;
        let n = self.select(parent_visits);

        let mut grid_clone = grid.clone();
        grid_clone.add_range(&n.moves);
        n.iterate(current_turn, players, win_length, grid.clone());

        self.total_iterations += 1;
    }
    pub fn find_best_move(&self) -> Size {
        self.nodes
            .iter()
            .max_by(|x, y|
                ((x.score as f32) / (x.visit_counter as f32)).total_cmp(
                    &((y.score as f32) / (y.visit_counter as f32))
                )
            )
            .expect("No node found")
            .moves.first()
            .unwrap().position
    }
}
#[derive(Debug)]
struct Node {
    moves: Vec<PlayerMove>,
    children: Vec<Node>,
    score: f32,
    visit_counter: u32,
    possible_moves: Vec<Size>,
}
impl Node {
    pub fn new(m: PlayerMove, mut grid: Grid, win_length: u32) -> Self {
        // Check for win
        let mut moves = vec![m.clone()];
        grid.add(m.clone());
        let mut result = grid.check_win(&m.position, win_length);
        let won = result.len() > 0;
        moves.append(&mut result);

        let possible_moves = grid.get_possible_moves_size();

        let s = Self {
            moves: moves,
            children: Vec::new(),
            score: if won {
                1.0
            } else if possible_moves.len() > 0 {
                0.0
            } else {
                0.5
            },
            visit_counter: 1,
            possible_moves: if won {
                Vec::new()
            } else {
                possible_moves
            },
        };
        s
    }
    pub fn from_possible_moves(moves: Vec<PlayerMove>, grid: &Grid, win_length: u32) -> Vec<Self> {
        let mut v: Vec<Self> = Vec::with_capacity(moves.len());
        for m in moves {
            v.push(Self::new(m, grid.clone(), win_length));
        }
        v
    }
    pub fn get_uct_score(&self, parent_visit_counter: u32) -> f32 {
        let exploration_parameter = (2.0f32).sqrt();

        (self.score as f32) / (self.visit_counter as f32) +
            exploration_parameter *
                ((parent_visit_counter as f32).log10() / (self.visit_counter as f32)).sqrt()
    }
    pub fn expand(&mut self, id: i32, grid: &Grid, win_length: u32) {
        self.children.push(
            Node::new(
                PlayerMove::new(
                    id,
                    self.possible_moves.remove(
                        rand::thread_rng().gen_range(0..self.possible_moves.len())
                    )
                ),
                grid.clone(),
                win_length
            )
        );
    }
    pub fn iterate(
        &mut self,
        current_turn: usize,
        players: &Vec<i32>,
        win_length: u32,
        mut grid: Grid
    ) -> (usize, f32) {
        // println!("Children: {}", self.children.len());
        print!(
            ">>({},{})-{}",
            self.moves.first().unwrap().position.x,
            self.moves.first().unwrap().position.y,
            self.score
        );
        if current_turn == 0 {
            print!("#");
        } else {
            print!(" ");
        }
        grid.add_range(&self.moves);

        let selected = self.children
            .iter_mut()
            .max_by(|x, y|
                x.get_uct_score(self.visit_counter).total_cmp(&y.get_uct_score(self.visit_counter))
            );

        if selected.is_none() && self.possible_moves.len() == 0 {
            // println!("Possible moves: {}", self.possible_moves.len());
            if self.possible_moves.len() == 0 {
                println!("Reached end:{}", self.score);
                return (current_turn, self.score);
            }
        } else if selected.is_none() || self.possible_moves.len() > 0 {
            drop(selected);

            self.expand(players[current_turn], &grid, win_length);
            let child = self.children.last_mut().unwrap();

            grid.add_range(&child.moves);

            if child.possible_moves.len() > 0 {
                if child.score != 0.0 {
                    println!("score:{}", child.score);
                    panic!();
                }
                for _i in 0..10 {
                    child.score +=
                        Self::simulate(
                            &mut grid,
                            (current_turn + 1) % players.len(),
                            (current_turn + 1) % players.len(),
                            players,
                            win_length
                        ) / 10.0;
                }
            }
            self.visit_counter += 1;

            println!("Expanded:{}", child.score);
            return ((current_turn + 1) % players.len(), child.score);
        }

        let selected_move = selected.unwrap();
        let result = selected_move.iterate(
            (current_turn + 1) % players.len(),
            players,
            win_length,
            grid
        );

        if current_turn == result.0 || result.1 == 0.5 {
            self.score += result.1;
        } else {
            self.score -= result.1;
        }
        self.visit_counter += 1;

        result
    }
    pub fn simulate(
        grid: &mut Grid,
        self_id: usize,
        mut current_turn: usize,
        players: &Vec<i32>,
        win_length: u32
    ) -> f32 {
        let mut score = 0.0;
        let mut m = RandomBot::get_random_move(players[current_turn], grid);
        while m.is_some() {
            let pos = m.unwrap();

            grid.add(PlayerMove::new(players[current_turn], pos));
            let result = grid.check_win(&pos, win_length);

            grid.add_range(&result);

            if result.len() > 0 && current_turn == self_id {
                return 1.0;
            } else if result.len() > 0 {
                return 0.0;
            }

            current_turn = (current_turn + 1) % players.len();

            m = RandomBot::get_random_move(players[current_turn], grid);
        }

        0.5
    }
}
