use crate::{ bot::botlogic::BotLogic, game::Game, grid::Grid, player_move::PlayerMove, Size };

pub(crate) struct MinMaxBot {}
impl MinMaxBot {
    pub fn new() -> Self {
        Self {}
    }
}
impl BotLogic for MinMaxBot {
    fn generate_move(&self, id: i32, game: &Game) -> Size {
        Self::get_best_move(id, game, 5).unwrap_or(Size::new(0, 0))
    }
    fn get_name(&self) -> String {
        "minmax".to_string()
    }
}
impl MinMaxBot {
    fn get_best_move(id: i32, game: &Game, depth: u32) -> Option<Size> {
        let moves = game.grid.get_possible_moves(id);

        if moves.len() < 1 {
            return None;
        }

        let mut move_counter = 0;
        let total = Self::get_complexity(
            moves.len().try_into().expect("Could not convert usize to u32"),
            depth.into()
        );

        let (high_score, best_move) = Self::find_best_move(
            moves,
            &game.grid,
            &game.player_list,
            &game.win_length,
            game.current_turn,
            id,
            depth,
            &mut move_counter,
            &total
        );

        println!("Proceeding with move {:?} with score {}", best_move, high_score);
        Some(best_move)
    }
    fn get_score(
        m: &PlayerMove,
        mut grid: Grid,
        id: i32,
        player_list: &Vec<i32>,
        win_length: &u32,
        current_turn: usize,
        depth: u32,
        move_counter: &mut u128,
        total: &u128
    ) -> i32 {
        if *move_counter % 100000 == 0 {
            println!("Processing move {}/{} at depth: {}", move_counter, total, depth);
        }
        *move_counter += 1;

        let mut sum: i32 = 0;

        grid.add(m.clone());

        let moves = grid.check_win(&m.position, *win_length);

        if moves.len() > 0 {
            sum +=
                <u32 as TryInto<i32>>::try_into(depth).expect("Could not convert u32 to i32") + 1;
        }

        if depth <= 0 {
            return sum;
        }

        grid.add_range(&moves);

        let next_turn = if current_turn + 1 < player_list.len() { current_turn + 1 } else { 0 };

        let possible_moves = grid.get_possible_moves(player_list[next_turn]);

        if possible_moves.len() > 0 {
            let (high_score, _best_move) = Self::find_best_move(
                possible_moves,
                &grid,
                player_list,
                win_length,
                next_turn,
                id,
                depth,
                move_counter,
                total
            );
            sum -= high_score;
        }

        sum
    }
    fn find_best_move(
        moves: Vec<PlayerMove>,
        grid: &Grid,
        player_list: &Vec<i32>,
        win_length: &u32,
        current_turn: usize,
        id: i32,
        depth: u32,
        move_counter: &mut u128,
        total: &u128
    ) -> (i32, Size) {
        let mut high_score: i32 = i32::MIN;
        let mut best_move = moves[0].position;

        for m in moves {
            let score = Self::get_score(
                &m,
                grid.clone(),
                id,
                player_list,
                win_length,
                current_turn,
                depth - 1,
                move_counter,
                total
            );
            if score > high_score {
                high_score = score;
                best_move = m.position;
            }
            *move_counter += 1;
        }
        (high_score, best_move)
    }

    fn get_complexity(num: u128, depth: u128) -> u128 {
        if depth <= 0 {
            return 1;
        }
        match num {
            0 => 1,
            1 => 1,
            _ => num * Self::get_complexity(num - 1, depth - 1),
        }
    }
}
