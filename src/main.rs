//#![allow(unused)]
use std::collections::HashMap;
use std::io::{Error, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{
    mpsc::{self, Sender},
    Arc, Mutex,
};
use std::thread;

fn main() -> Result<(), Error> {
    const IP_ADDR: &str = "127.0.0.1";
    const PORT: &str = "7878";

    let listener = TcpListener::bind([IP_ADDR, PORT].join(":"))?;
    let mut _messages: Arc<Mutex<Vec<String>>> = Default::default();
    let clients: Arc<Mutex<HashMap<u8, TcpStream>>> = Default::default();
    // TODO when making the client into a different program, i can't use mpsc
    // so push to messages variable ?
    // let's try that (don't depend on mpsc)
    let (tx, rx) = mpsc::channel::<HashMap<u8, String>>();

    let cloned_clients = Arc::clone(&clients);
    let _cloned_messages = Arc::clone(&_messages);
    let _server = thread::spawn(move || {
        loop {
            let hashed_msg = rx.recv().expect("the sender should always be connected");

            for client in cloned_clients.lock().unwrap().iter_mut() {
                for client_name in hashed_msg.keys() {
                    if client_name != client.0 {
                        // NOTE if the user didn't complete his text, the receive message gonna append on that message
                        //(i think tui gonna fix that)
                        let _ = client
                            .1
                            .write_all(hashed_msg.get(client_name).unwrap().as_bytes());
                    }
                }
            }
        }
    });
    let mut id: u8 = 0;
    for stream in listener.incoming() {
        let mut stream = stream?;
        id += 1;

        clients
            .lock()
            .unwrap()
            .insert(id, stream.try_clone().unwrap());
        let tx = tx.clone();

        let _client = thread::spawn(move || -> Result<(), Error> {
            let _ = handle_client(id, &mut stream, tx);
            Ok(())
        });
    }
    Ok(())
}
fn handle_client(
    id: u8,
    stream: &mut TcpStream,
    tx: Sender<HashMap<u8, String>>,
) -> Result<(), Error> {
    loop {
        let mut raw_message = [0; 1024];
        let bytes_readed = stream.read(&mut raw_message)?;
        let message: String = str::from_utf8(&raw_message[..bytes_readed])
            .unwrap_or("")
            .to_string();

        // NOTE /quit will not disable reading from input, but i can't send messaages (i can receive them),
        //this happend because i am storing streams on a container and didn't not free them
        //TODO do parsing, like when the string is empty, i don't want to append it to my HashMap
        if message.trim() == "/quit" {
            break Ok(());
        } else {
            let hashed_message = HashMap::from([(id, message)]);
            tx.send(hashed_message.clone())
                .expect("the receiver should always be connected");
        }
    }
}

/*
TODO
add an Id beside the name with a touple to fix when clients have the same name
make clients program instead of having one program
make the app with cli using ratatui -> https://ratatui.rs/tutorials/

NOTE
- i think it's better to make if let instead of spaming unwrap()
*/
#[cfg(test)]
mod test {
    #[test]
    #[allow(non_snake_case)]
    fn concatenate_Strings() {
        let s1 = String::from("Hello, ");
        let s2 = String::from("World");
        let s3 = s1 + &s2;
        assert_eq!(String::from("Hello, World"), s3);
    }
}
