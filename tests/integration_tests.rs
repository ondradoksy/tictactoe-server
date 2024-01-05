use jzon::{ self, object, JsonValue::Null };
use tungstenite::{ Message, WebSocket, stream::MaybeTlsStream };

use crate::common::stop_server;

mod common;

macro_rules! clean_assert {
    ($condition:expr, $child:ident) => {
        if !$condition {
            stop_server(&mut $child);
            panic!();
        }
    };
}

#[test]
fn can_connect() {
    let ip = "127.0.0.2:9002";
    let mut child = common::start_server(ip);
    let conn = common::get_connection(ip);

    clean_assert!(conn.is_ok(), child);

    common::stop_server(&mut child);
}

#[test]
fn player_list() {
    let ip = "127.0.0.3:9003";

    let mut child = common::start_server(ip);

    let player_num = 10;

    let mut conns: Vec<WebSocket<MaybeTlsStream<std::net::TcpStream>>> = Vec::new();

    for i in 0..player_num {
        let result = common::get_connection(ip);

        // Check connection
        clean_assert!(result.is_ok(), child);

        let mut conn = result.unwrap();

        // Send request
        conn.send(
            Message::text(
                jzon::stringify(
                    object! {
                        event: "players",
                        content: ""
                    }
                )
            )
        ).unwrap();

        // Check response
        let response = conn.read();
        clean_assert!(response.is_ok(), child);

        // Parse response
        let parsed_result = jzon::parse(response.unwrap().to_text().unwrap());
        clean_assert!(parsed_result.is_ok(), child);

        let parsed = parsed_result.unwrap();

        // Check response
        clean_assert!(parsed["event"].is_string(), child);
        clean_assert!(parsed["content"].is_string(), child);
        clean_assert!(parsed["event"] == "players", child);

        // Parse content
        let parsed_content_result = jzon::parse(parsed["content"].as_str().unwrap());
        clean_assert!(parsed_content_result.is_ok(), child);

        let parsed_content = parsed_content_result.unwrap();

        // Check content
        clean_assert!(parsed_content.is_array(), child);
        clean_assert!(parsed_content.as_array().unwrap().len() == i + 1, child);
        clean_assert!(parsed_content.as_array().unwrap()[i].is_object(), child);
        clean_assert!(
            parsed_content
                .as_array()
                .unwrap()
                .contains(
                    &(object! {
                    id: i,
                    joined_game: Null,
                    ready: false
        })
                ),
            child
        );

        conns.push(conn);
    }

    for mut conn in conns {
        conn.close(None).unwrap();
        conn.flush().unwrap();
        conn.read().unwrap();
        clean_assert!(!conn.can_read(), child);
    }

    common::stop_server(&mut child);
}

#[test]
fn games_list() {
    let ip = "127.0.0.4:9004";

    let mut child = common::start_server(ip);

    let game_num = 10;

    let result = common::get_connection(ip);

    // Check connection
    clean_assert!(result.is_ok(), child);

    let mut conn = result.unwrap();

    for i in 0..game_num {
        // Send request
        conn.send(
            Message::Text(
                jzon::stringify(
                    object! {
                        event: "create_game",
                        content: jzon::stringify(object! {
                            size: {
                                x: i + 3,
                                y: i + 3
                            },
                            hotjoin: true,
                            player_limit: 10,
                        })
                    }
                )
            )
        ).unwrap();

        // Check response
        let response = conn.read();
        clean_assert!(response.is_ok(), child);

        // Parse response
        let parsed_result = jzon::parse(response.unwrap().to_text().unwrap());
        clean_assert!(parsed_result.is_ok(), child);

        let parsed = parsed_result.unwrap();

        // Check response
        clean_assert!(parsed["event"].is_string(), child);
        clean_assert!(parsed["content"].is_string(), child);
        clean_assert!(parsed["event"] == "create_game", child);

        // Parse content
        let parsed_content_result = jzon::parse(parsed["content"].as_str().unwrap());
        clean_assert!(parsed_content_result.is_ok(), child);

        let parsed_content = parsed_content_result.unwrap();

        clean_assert!(parsed_content["status"].is_string(), child);
        clean_assert!(parsed_content["status"] == "ok", child);

        // Send request
        conn.send(
            Message::text(
                jzon::stringify(
                    object! {
                        event: "games",
                        content: ""
                    }
                )
            )
        ).unwrap();

        // Check response
        let response = conn.read();
        clean_assert!(response.is_ok(), child);

        // Parse response
        let parsed_result = jzon::parse(response.unwrap().to_text().unwrap());
        clean_assert!(parsed_result.is_ok(), child);

        let parsed = parsed_result.unwrap();

        // Check response
        clean_assert!(parsed["event"].is_string(), child);
        clean_assert!(parsed["content"].is_string(), child);
        clean_assert!(parsed["event"] == "games", child);

        // Parse content
        let parsed_content_result = jzon::parse(parsed["content"].as_str().unwrap());
        clean_assert!(parsed_content_result.is_ok(), child);

        let parsed_content = parsed_content_result.unwrap();

        // Check content
        clean_assert!(parsed_content.is_array(), child);
        clean_assert!(parsed_content.as_array().unwrap().len() == i + 1, child);
        clean_assert!(parsed_content.as_array().unwrap()[i].is_object(), child);
        clean_assert!(
            parsed_content
                .as_array()
                .unwrap()
                .contains(
                    &(object! {
                        id: i,
                        player_list: [],
                        creator: {
                            id: 0,
                            joined_game: Null,
                            ready: false
                        },
                        current_turn: 0,
                        hotjoin: true,
                        player_limit: 10,
                        running: false,
                    })
                ),
            child
        );
    }

    common::stop_server(&mut child);
}
