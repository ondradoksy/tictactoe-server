use std::process::Child;

use tungstenite::Message;

use crate::common::stop_server;

mod common;

#[test]
fn can_connect() {
    let ip = "127.0.0.2:9001";
    let mut child = common::start_server(ip);
    let conn = common::get_connection(ip);

    clean_assert(conn.is_ok(), &mut child);

    common::stop_server(&mut child);
}

#[test]
fn player_list() {
    let ip = "127.0.0.3:9001";

    let mut child = common::start_server(ip);
    let conn1 = common::get_connection(ip);
    let conn2 = common::get_connection(ip);

    clean_assert(conn1.is_ok(), &mut child);
    clean_assert(conn2.is_ok(), &mut child);

    let mut player1 = conn1.unwrap();
    let mut _player2 = conn2.unwrap();

    player1.send(Message::text("{\"event\":\"players\",\"content\":\"\"}")).unwrap();

    clean_assert(player1.read().unwrap().len() > 0, &mut child);

    common::stop_server(&mut child);
}

fn clean_assert(value: bool, child: &mut Child) {
    if value {
        return;
    }
    stop_server(child);
    panic!();
}
