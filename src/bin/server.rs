#![allow(unused)]
use cli_chat_app::utils::parsing;
use std::{
    collections::HashMap,
    io::{Error, Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

fn get_client_name(stream: &mut TcpStream) -> String {
    let mut raw_message = [0; 1024];
    let bytes_readed = stream.read(&mut raw_message).unwrap();
    let message: String = str::from_utf8(&raw_message[..bytes_readed])
        .unwrap()
        .trim()
        .to_string();
    let name = message;
    name
}

fn main() -> Result<(), Error> {
    const IP_ADDR: &str = "127.0.0.1";
    const PORT: &str = "7878";

    let listener = TcpListener::bind([IP_ADDR, PORT].join(":"))?;
    let messages: Arc<Mutex<Vec<String>>> = Default::default();
    let clients: Arc<Mutex<HashMap<String, TcpStream>>> = Default::default();

    let mut connecters = 0;
    // this for loop is wierd, only iterate on the first element once
    //let _broadcaster = thread::spawn(|| {});
    for stream in listener.incoming() {
        let mut stream = stream?;
        let cloned_messages = Arc::clone(&messages);
        let cloned_clients = Arc::clone(&clients);

        let name = get_client_name(&mut stream);
        clients
            .try_lock()
            .unwrap()
            .insert(name, stream.try_clone().unwrap());

        thread::spawn(move || {
            loop {
                let mut raw_message = [0; 1024];
                let bytes_readed = stream.read(&mut raw_message).unwrap();
                let detailed_message: String = str::from_utf8(&raw_message[..bytes_readed])
                    .unwrap()
                    .to_string();
                cloned_messages
                    .lock()
                    .unwrap()
                    .push(detailed_message.clone());
                // parse msg here ?

                for client in cloned_clients.try_lock().unwrap().iter_mut() {
                    let (name, mut msg) = parsing(&detailed_message.clone());
                    if name != *client.0 {
                        dbg!(&msg);
                        let _ = client.1.write_all(detailed_message.as_bytes());
                    }
                }

                //dbg!(&stream);
                //dbg!(&message);
                dbg!(&cloned_messages);
            }
        });

        connecters += 1;
        println!("number of connected clients: {connecters}");
    }
    Ok(())
}

/*
TODO
bind server to wifi, and bind clients to
make the app with cli using ratatui -> https://ratatui.rs/tutorials/

NOTE
- i think it's better to make if let instead of spaming unwrap()
*/
#[cfg(test)]
mod test {
    #[test]
    #[allow(non_snake_case)]
    fn concatenate_Strings() {
        let mut data = String::new();
        let data_2 = &data;
        let data_3 = &mut data;
        *data_3 = String::from("hello world");
        //dbg!(&data_2);
        let s1 = String::from("Hello, ");
        let s2 = String::from("World");
        let s3 = s1 + &s2;
        assert_eq!(String::from("Hello, World"), s3);
    }
}
