//! Provides a struct and methods for representing and managing bots in the game.

mod botlogic;
mod random;
mod minmax;
mod mcts;

use std::{ sync::{ mpsc::{ self, Receiver }, Arc, Mutex }, thread::Builder };

use crate::{ game::Game, net::{ broadcast_players, MessageEvent }, player::Player };

use self::{ botlogic::BotLogic, mcts::MCTSBot, minmax::MinMaxBot, random::RandomBot };

/// A struct representing a bot player in the game.
pub(crate) struct Bot {
    /// The player object associated with the bot.
    player: Arc<Mutex<Player>>,
    /// The game instance the bot is playing in.
    game: Arc<Mutex<Game>>,
    /// The type of bot algorithm used (e.g., "minmax", "mcts").
    pub bot_type: String,
}
impl Bot {
    /// Creates a new `Bot` instance with the specified details.
    ///
    /// The bot automatically adds itself to the players in the game.
    ///
    /// # Arguments
    ///
    /// * `id`: The unique identifier for the bot.
    /// * `bot_type`: An optional string specifying the type of bot algorithm to use ("minmax", "mcts", etc.). Defaults to an empty string.
    /// * `players`: An `Arc<Mutex<Vec<Arc<Mutex<Player>>>>>` containing all players in the game.
    /// * `game`: An `Arc<Mutex<Game>>` representing the game instance.
    ///
    /// # Returns
    ///
    /// An `Arc<Mutex<Self>>` representing the newly created bot.
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

    /// Selects the appropriate bot logic implementation based on the bot type string.
    ///
    /// # Arguments
    ///
    /// * `s`: A string representing the bot type ("minmax", "mcts", etc.).
    ///
    /// # Returns
    ///
    /// A `Box<dyn BotLogic>` containing the chosen bot logic implementation. Defaults to an instance of [`RandomBot`] if the string is not matched with any other algorithm name.
    fn get_bot_logic(s: &str) -> Box<dyn BotLogic> {
        match s {
            "minmax" => { Box::new(MinMaxBot::new()) }
            "mcts" => { Box::new(MCTSBot::new()) }
            _ => { Box::new(RandomBot::new()) }
        }
    }

    /// The main loop of the bot thread, handling received messages and processing turns.
    ///
    /// Does the same functionality that [`crate::handle_connection`] would if the player was not a bot.
    ///
    /// # Arguments
    ///
    /// * `bot`: An `Arc<Mutex<Bot>>` representing the bot instance.
    /// * `rx`: A `Receiver<MessageEvent>` used to receive messages from the game thread.
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

    /// Processes a "turn" message, using the assigned bot logic to generate a move.
    ///
    /// # Arguments
    ///
    /// * `content`: A string representing the content of the received "turn" message.
    /// * `bot_logic`: A reference to a `Box<dyn BotLogic>` containing the chosen bot logic implementation.
    fn process_turn(&self, content: &str, bot_logic: &Box<dyn BotLogic>) {
        if !self.game.lock().unwrap().is_running() {
            return;
        }
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

    /// Attempts to make a move in the game using the provided bot logic.
    ///
    /// # Arguments
    ///
    /// * `bot_logic`: A reference to a `Box<dyn BotLogic>` containing the chosen bot logic implementation.
    fn try_make_move(&self, bot_logic: &Box<dyn BotLogic>) {
        println!("Processing move of bot type: {}", self.bot_type);

        let m = bot_logic.generate_move(self.player.lock().unwrap().id, &self.game.lock().unwrap());

        println!("Sending move {:?}", m);

        if !self.game.lock().unwrap().add_move(&self.player.clone(), m) {
            println!("Illegal move, something went wrong!");
        }
    }
}
