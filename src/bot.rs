use std::sync::{ mpsc::{ self, Receiver }, Arc, Mutex };

use crate::{ net::MessageEvent, player::Player };

pub(crate) struct Bot {
    player: Arc<Mutex<Player>>,
}
impl Bot {
    pub fn new(id: i32) -> Self {
        let (tx, rx) = mpsc::channel();
        Self { player: Arc::new(Mutex::new(Player::new(id, tx))) }
    }
    fn run(bot: Arc<Mutex<Bot>>, rx: Receiver<MessageEvent>) {
        for msg in rx.iter() {
            println!("{}", msg.to_message());
            todo!("Implement message handling for bot");
        }
    }
}
