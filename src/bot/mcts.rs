//! Implements a bot logic that utilizes the Monte Carlo Tree Search (MCTS) algorithm for decision-making.

use crate::{ bot::botlogic::BotLogic, game::Game, grid::Grid, player_move::PlayerMove, Size };

use rand::Rng;

pub(crate) struct MCTSBot {}
impl BotLogic for MCTSBot {
    /// Generates a move using the MCTS algorithm based on the current game state.
    ///
    /// # Arguments
    ///
    /// * `id`: The bot's unique identifier.
    /// * `game`: A reference to the current game instance.
    ///
    /// # Returns
    ///
    /// A `Size` representing the determined optimal move, or the move (0, 0) if no valid moves are available.
    fn generate_move(&self, id: i32, game: &Game) -> Size {
        let mut algorithm = MCTSAlgorithm::new(id, &game.grid, game.win_length);

        let max_iter = 10000;

        for i in 0..max_iter {
            println!("MCTS: Iterating... {}/{}", i + 1, max_iter);
            algorithm.iterate(&game.player_list, game.current_turn, &game.grid, game.win_length);
        }

        algorithm.find_best_move()
    }

    /// Returns a string representing the bot logic type ("mcts").
    ///
    /// # Returns
    ///
    /// A string representing the bot logic type.
    fn get_name(&self) -> String {
        "mcts".to_string()
    }
}
impl MCTSBot {
    /// Creates a new instance of `MCTSBot`.
    ///
    /// # Returns
    ///
    /// A new `MCTSBot` instance.
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
    /// Creates a new `MCTSAlgorithm` instance.
    ///
    /// # Arguments
    ///
    /// * `id`: The bot's unique identifier.
    /// * `grid`: A reference to the game grid.
    /// * `win_length`: The game's win length.
    ///
    /// # Returns
    ///
    /// A new `MCTSAlgorithm` instance.
    pub fn new(id: i32, grid: &Grid, win_length: u32) -> Self {
        let nodes = Node::from_possible_moves(grid.get_possible_moves(id), grid, win_length);
        Self {
            total_iterations: nodes.len() as u32,
            nodes: nodes,
        }
    }

    /// Selects the most promising child node based on the Upper Confidence Bound applied to Trees (UCT) formula.
    ///
    /// # Arguments
    ///
    /// * `parent_visit_counter`: The visit count of the parent node.
    ///
    /// # Returns
    ///
    /// A mutable reference to the selected child node.
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

    /// Performs an MCTS iteration, which involves selection, expansion, simulation, and backpropagation.
    ///
    /// # Arguments
    ///
    /// * `players`: A reference to the list of players in the game.
    /// * `current_turn`: The current turn index in the game.
    /// * `grid`: A reference to the game grid.
    /// * `win_length`: The game's win length.
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

    /// Identifies the move associated with the highest win rate based on the simulation results.
    ///
    /// # Returns
    ///
    /// A `Size` representing the move determined to be optimal.
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
    win_result: Option<f32>,
}
impl Node {
    /// Creates a new `Node` instance representing a possible move and its simulation results.
    ///
    /// # Arguments
    ///
    /// * `m`: A `PlayerMove` representing the move associated with the node.
    /// * `mut grid`: A mutable reference to the game grid used for simulations.
    /// * `win_length`: The game's win length.
    ///
    /// # Returns
    ///
    /// A new `Node` instance.
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
            } else {
                0.0
            },
            visit_counter: 1,
            possible_moves: if won {
                Vec::new()
            } else {
                possible_moves
            },
            win_result: if won {
                Some(1.0)
            } else {
                None
            },
        };
        s
    }

    /// Creates a vector of `Node` instances from a list of possible moves.
    ///
    /// # Arguments
    ///
    /// * `moves`: A vector containing the `PlayerMove` options to be considered.
    /// * `grid`: A reference to the game grid.
    /// * `win_length`: The game's win length.
    ///
    /// # Returns
    ///
    /// A vector containing the created `Node` instances.
    pub fn from_possible_moves(moves: Vec<PlayerMove>, grid: &Grid, win_length: u32) -> Vec<Self> {
        let mut v: Vec<Self> = Vec::with_capacity(moves.len());
        for m in moves {
            v.push(Self::new(m, grid.clone(), win_length));
        }
        v
    }

    /// Calculates the UCT score of the node, which incorporates exploration and exploitation factors.
    ///
    /// # Arguments
    ///
    /// * `parent_visit_counter`: The visit count of the parent node.
    ///
    /// # Returns
    ///
    /// A `f32` value representing the UCT score.
    pub fn get_uct_score(&self, parent_visit_counter: u32) -> f32 {
        let exploration_parameter = (2.0f32).sqrt();

        (self.score as f32) / (self.visit_counter as f32) +
            exploration_parameter *
                ((parent_visit_counter as f32).log10() / (self.visit_counter as f32)).sqrt()
    }

    /// Expands the node by creating a child node for a new possible move.
    ///
    /// # Arguments
    ///
    /// * `id`: The bot's unique identifier.
    /// * `grid`: A reference to the game grid.
    /// * `win_length`: The game's win length.
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
    /// Performs an MCTS iteration on a child node, simulating gameplay and updating scores.
    ///
    /// # Arguments
    ///
    /// * `current_turn`: The current turn index in the game.
    /// * `players`: A reference to the list of players in the game.
    /// * `win_length`: The game's win length.
    /// * `mut grid`: A mutable reference to the game grid used for simulations.
    ///
    /// # Returns
    ///
    /// A tuple containing the next player's turn index and the score obtained from the simulation.
    pub fn iterate(
        &mut self,
        current_turn: usize,
        players: &Vec<i32>,
        win_length: u32,
        mut grid: Grid
    ) -> (usize, f32) {
        self.visit_counter += 1;
        if self.win_result.is_some() {
            self.score += self.win_result.unwrap();
            return (current_turn, self.win_result.unwrap());
        }

        grid.add_range(&self.moves);

        let selected = self.children
            .iter_mut()
            .max_by(|x, y|
                x.get_uct_score(self.visit_counter).total_cmp(&y.get_uct_score(self.visit_counter))
            );

        if selected.is_none() && self.possible_moves.len() == 0 {
            if self.possible_moves.len() == 0 {
                return (current_turn, self.score);
            }
        } else if selected.is_none() || self.possible_moves.len() > 0 {
            drop(selected);

            self.expand(players[(current_turn + 1) % players.len()], &grid, win_length);
            let child = self.children.last_mut().unwrap();

            grid.add_range(&child.moves);

            if child.possible_moves.len() > 0 {
                if child.score != 0.0 {
                    println!("score:{}", child.score); // This shouldn't happen
                    panic!();
                }

                child.score = Self::simulate(
                    &mut grid,
                    (current_turn + 1) % players.len(),
                    (current_turn + 1) % players.len(),
                    players,
                    win_length,
                    child.possible_moves.clone()
                );
            }

            self.score -= child.score;

            // println!("Expanded:{}", child.score);
            return ((current_turn + 1) % players.len(), child.score);
        }

        let selected_move = selected.unwrap();
        let result = selected_move.iterate(
            (current_turn + 1) % players.len(),
            players,
            win_length,
            grid
        );

        if current_turn == result.0 {
            self.score += result.1;
        } else {
            self.score -= result.1;
        }

        result
    }

    /// Simulates a random gameplay scenario starting from the current node's move.
    ///
    /// # Arguments
    ///
    /// * `grid`: A mutable reference to the game grid used for simulations.
    /// * `self_id`: The bot's unique identifier.
    /// * `mut current_turn`: A mutable reference to the current turn index in the simulation.
    /// * `players`: A reference to the list of players in the game.
    /// * `win_length`: The game's win length.
    /// * `possible_moves`: A vector containing the remaining possible moves for future simulation steps.
    ///
    /// # Returns
    ///
    /// A `f32` value representing the score obtained from the simulation (-1.0 for loss, 1.0 for win, 0.0 for draw).
    pub fn simulate(
        grid: &mut Grid,
        self_id: usize,
        mut current_turn: usize,
        players: &Vec<i32>,
        win_length: u32,
        mut possible_moves: Vec<Size>
    ) -> f32 {
        while possible_moves.len() > 0 {
            let pos = possible_moves.remove(rand::thread_rng().gen_range(0..possible_moves.len()));

            grid.add(PlayerMove::new(players[current_turn], pos));
            let result = grid.check_win(&pos, win_length);

            grid.add_range(&result);

            if result.len() > 0 && current_turn == self_id {
                return 1.0;
            } else if result.len() > 0 {
                return -1.0;
            }

            current_turn = (current_turn + 1) % players.len();
        }

        0.0
    }
}
