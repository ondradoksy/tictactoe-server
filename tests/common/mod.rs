use std::{
    net::TcpStream,
    process::{ Child, Stdio, ChildStdout },
    thread,
    time,
    io::{ BufReader, BufRead, Lines },
};

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

pub fn start_server(addr: &str) -> (Child, Lines<BufReader<ChildStdout>>) {
    let mut child = std::process::Command
        ::new("cargo")
        .arg("run")
        .arg("-q")
        .arg("--")
        .arg(addr)
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start the server");

    // Create a reader for the child's stdout
    let reader = child.stdout.take().expect("Failed to get stdout");
    let buf_reader = BufReader::new(reader);
    let mut lines = buf_reader.lines();

    // Lopp until child outputs some data
    loop {
        match lines.next() {
            Some(Ok(line)) => {
                // Server has likely started
                println!("{}", line);
                break;
            }
            None => {
                // Wait for server to start
                thread::sleep(time::Duration::from_millis(100));
            }
            Some(Err(e)) => panic!("Error trying to read output {}", e.to_string()),
        }
    }

    match child.try_wait() {
        Ok(Some(status)) => {
            panic!("Child process should not have exited yet! Status: {}", status.to_string());
        }
        Ok(None) => {
            // Child is running, all is good
            println!("Server started succesfully.");
        }
        Err(e) => panic!("Failed to wait for exit status. {}", e.to_string()),
    }

    (child, lines)
}

pub fn stop_server(child: &mut (Child, Lines<BufReader<ChildStdout>>)) {
    // Display output

    // Send a SIGINT to the child process to terminate it
    if child.0.kill().is_err() {
        panic!("Failed to send SIGINT to child process");
    }

    child.0.wait().unwrap();

    loop {
        match child.1.next() {
            Some(Ok(line)) => {
                println!("{}", line);
            }
            None => {
                break;
            }
            Some(Err(e)) => panic!("Error trying to read output {}", e.to_string()),
        }
    }
}
