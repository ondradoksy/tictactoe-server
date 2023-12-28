use std::{ net::TcpStream, process::Child, thread, time };

use tungstenite::{ client::connect, WebSocket, stream::MaybeTlsStream };

pub fn get_connection(addr: &str) -> Result<WebSocket<MaybeTlsStream<TcpStream>>, &'static str> {
    let ip = "ws://".to_owned() + addr + "/";
    let result = connect(ip);
    if result.is_err() {
        return Err("Connection failed");
    }
    let (socket, _response) = result.unwrap();
    Ok(socket)
}

pub fn start_server(addr: &str) -> Child {
    let child = std::process::Command
        ::new("cargo")
        .arg("run")
        .arg("-q")
        .arg("--")
        .arg(addr)
        .spawn()
        .expect("Failed to start the server");

    // Wait for server to start
    thread::sleep(time::Duration::from_millis(100));

    child
}

pub fn stop_server(child: &mut Child) {
    // Send a SIGINT to the child process to terminate it
    if child.kill().is_err() {
        panic!("Failed to send SIGINT to child process");
    }

    child.wait().unwrap();
}
