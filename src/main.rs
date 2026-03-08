
use std::io::{Error, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
fn main() -> Result<(), Error> {
    let listener = TcpListener::bind("127.0.0.1:7878")?;
    for stream in listener.incoming() {
        let _client = thread::spawn(move|| -> Result<(), Error>{
            let mut stream = stream?;
            _ = handle_request(&stream);
            _ = handle_response(&mut stream);
            Ok(())
        });
    }
    Ok(())
}
fn handle_request(stream: &TcpStream) -> Result<(), Error> {
    println!("new connection");
    let mut reader = stream;
    loop {
        let mut raw_request = [0; 1024];
        _ = reader.read(&mut raw_request)?;
        let request = str::from_utf8(&raw_request)
            .unwrap_or("")
            .trim_end_matches(['\n', '\0']);
        dbg!(&request);
        if request == "/quit" {
            break Ok(());
        }
    }
}
fn handle_response(stream: &mut TcpStream) {
    let response = "do you just connect ?\r\n";
    _ = stream.write_all(response.as_bytes());
}
/*
give clients a name
display messages from other clients
make the app with beautiful cli -> https://ratatui.rs/tutorials/
*/

