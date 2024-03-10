mod botlogic;
mod random;
mod minmax;

use std::{ sync::{ mpsc::{ self, Receiver }, Arc, Mutex }, thread::Builder };

use crate::{ game::Game, net::{ broadcast_players, MessageEvent }, player::Player };

use self::{ minmax::MinMaxBot, random::RandomBot, botlogic::BotLogic };

pub(crate) struct Bot {
    player: Arc<Mutex<Player>>,
    game: Arc<Mutex<Game>>,
    pub bot_type: String,
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

        let s = Self {
            player: p_arc,
            game: game.clone(),
            bot_type: bot_type.unwrap_or("".to_string()),
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
                Self::run(s_clone, rx);
            })
            .expect("Could not create thread");
        s_arc
    }
    fn get_bot_logic(s: &str) -> Box<dyn BotLogic> {
        match s {
            "minmax" => { Box::new(MinMaxBot::new()) }
            _ => { Box::new(RandomBot::new()) }
        }
    }
    fn run(bot: Arc<Mutex<Bot>>, rx: Receiver<MessageEvent>) {
        let bot_logic = Self::get_bot_logic(bot.lock().unwrap().bot_type.as_str());
        bot.lock().unwrap().bot_type = bot_logic.get_name();

        for msg in rx.iter() {
            match msg.event.as_str() {
                "turn" => {
                    bot.lock().unwrap().process_turn(msg.content.as_str(), &bot_logic);
                }
                _ => {
                    println!(
                        "Bot {}: Ignoring event: {}",
                        bot.lock().unwrap().player.lock().unwrap().id,
                        msg.event
                    );
                }
            }
        }
    }
    fn process_turn(&self, content: &str, bot_logic: &Box<dyn BotLogic>) {
        let cur_result: Result<i32, serde_json::Error> = serde_json::from_str(content);

        if cur_result.is_err() {
            println!("Failed to parse turn: {}", cur_result.err().unwrap());
            return;
        }

        let cur = cur_result.unwrap();
        if cur != self.player.lock().unwrap().id {
            // Not my turn
            return;
        }

        self.try_make_move(bot_logic);
    }
    fn try_make_move(&self, bot_logic: &Box<dyn BotLogic>) {
        println!("Processing move of bot type: {}", self.bot_type);

        let m = bot_logic.generate_move(self.player.lock().unwrap().id, &self.game.lock().unwrap());

        println!("Sending move {:?}", m);

        if !self.game.lock().unwrap().add_move(&self.player.clone(), m) {
            println!("Illegal move, something went wrong!");
        }
    }
}
