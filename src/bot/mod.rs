mod botlogic;
mod random;

use std::{ sync::{ mpsc::{ self, Receiver }, Arc, Mutex }, thread::Builder };

use crate::{
    bot::{ botlogic::BotLogic, random::RandomBot },
    game::Game,
    grid::Grid,
    net::{ broadcast_players, MessageEvent },
    player::Player,
    player_move::PlayerMove,
};

pub(crate) struct Bot {
    player: Arc<Mutex<Player>>,
    grid: Option<Grid>,
    game: Arc<Mutex<Game>>,
    bot_type: String,
}
impl Bot {
    pub fn new(
        id: i32,
        bot_type: Option<String>,
        players: &Arc<Mutex<Vec<Arc<Mutex<Player>>>>>,
        game: &Arc<Mutex<Game>>
    ) -> Arc<Mutex<Self>> {
        let (tx, rx) = mpsc::channel();
        let p_arc = Arc::new(Mutex::new(Player::new_bot(id, tx)));

        let bot_logic = Self::get_bot_logic(bot_type);

        let s = Self {
            player: p_arc,
            grid: None,
            game: game.clone(),
            bot_type: bot_logic.get_name(),
        };

        println!("Created new bot [{}]", s.bot_type);

        players.lock().unwrap().push(s.player.clone());
        game.lock().unwrap().join_player_forced(&s.player);
        broadcast_players(players);

        let s_arc = Arc::new(Mutex::new(s));
        let s_clone = s_arc.clone();
        Builder::new()
            .name(format!("Bot {} Thread", id))
            .spawn(move || {
                Self::run(s_clone, rx, bot_logic);
            })
            .expect("Could not create thread");
        s_arc
    }
    fn get_bot_logic(s: Option<String>) -> impl BotLogic {
        match s {
            _ => { RandomBot::new() }
        }
    }
    fn run(bot: Arc<Mutex<Bot>>, rx: Receiver<MessageEvent>, bot_logic: impl BotLogic) {
        let mut on_turn = false;
        for msg in rx.iter() {
            match msg.event.as_str() {
                "current_state" => {
                    bot.lock().unwrap().process_current_state(msg.content.as_str());
                }
                "move" => {
                    bot.lock().unwrap().process_move(msg.content.as_str());
                }
                "turn" => {
                    bot.lock()
                        .unwrap()
                        .process_turn(msg.content.as_str(), &mut on_turn, &bot_logic);
                }
                _ => {
                    println!(
                        "Bot {}: Ignoring event: {}",
                        bot.lock().unwrap().player.lock().unwrap().id,
                        msg.event
                    );
                }
            }

            if
                on_turn &&
                msg.event.as_str() != "turn" &&
                bot.lock().unwrap().game.lock().unwrap().is_running()
            {
                bot.lock().unwrap().try_make_move(&bot_logic);
            }
        }
    }
    fn process_current_state(&mut self, content: &str) {
        let cs_result = Grid::from_string(content);
        if cs_result.is_err() {
            println!("Failed to parse current_state: {}", cs_result.err().unwrap());
            return;
        }

        self.grid = Some(cs_result.unwrap());
    }
    fn process_move(&mut self, content: &str) {
        let m_result = PlayerMove::from_string(content);
        if m_result.is_err() {
            println!("Failed to parse move: {}", m_result.err().unwrap());
            return;
        }
        let m = m_result.unwrap();

        if self.grid.is_none() {
            return;
        }

        self.grid.as_mut().unwrap().add(m);
    }
    fn process_turn(&self, content: &str, on_turn: &mut bool, bot_logic: &impl BotLogic) {
        let cur_result: Result<i32, serde_json::Error> = serde_json::from_str(content);

        if cur_result.is_err() {
            println!("Failed to parse turn: {}", cur_result.err().unwrap());
            return;
        }

        let cur = cur_result.unwrap();
        if cur != self.player.lock().unwrap().id {
            // Not my turn
            *on_turn = false;
            return;
        }

        *on_turn = true;

        self.try_make_move(bot_logic);
    }
    fn try_make_move(&self, bot_logic: &impl BotLogic) {
        if self.grid.is_none() {
            self.request_current_state();
            return;
        }

        println!("Processing move of bot type: {}", self.bot_type);

        let m = bot_logic.generate_move(
            self.player.lock().unwrap().id,
            self.grid.as_ref().unwrap()
        );
        if !self.game.lock().unwrap().add_move(&self.player.clone(), m) {
            println!("Illegal move, board is likely out-of-date!");
            self.request_current_state();
        }
    }
    fn request_current_state(&self) {
        self.game.lock().unwrap().request_current_state(&self.player.clone());
    }
}
