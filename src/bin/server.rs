#![allow(unused)]
use std::collections::HashMap;
use std::io::{Error, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

fn main() -> Result<(), Error> {
    const IP_ADDR: &str = "127.0.0.1";
    const PORT: &str = "7878";

    let listener = TcpListener::bind([IP_ADDR, PORT].join(":"))?;
    let messages: Arc<Mutex<Vec<String>>> = Default::default();
    let clients: Arc<Mutex<Vec<TcpStream>>> = Default::default();

    let mut connecters = 0;
    // this for loop is wierd, only iterate on the first element once
    // NOTE i might need other thread for writing to clients
    //let _broadcaster = thread::spawn(|| {});
    for stream in listener.incoming() {
        let mut stream = stream?;
        let cloned_messages = Arc::clone(&messages);
        let cloned_clients = Arc::clone(&clients);
        clients
            .try_lock()
            .unwrap()
            .push(stream.try_clone().unwrap());

        thread::spawn(move || {
            loop {
                let mut raw_message = [0; 1024];
                let bytes_readed = stream.read(&mut raw_message).unwrap();
                let message: String = str::from_utf8(&raw_message[..bytes_readed])
                    .unwrap_or("")
                    .to_string();
                cloned_messages.lock().unwrap().push(message.clone());

                for client in cloned_clients.try_lock().unwrap().iter_mut() {
                    let _ = client.write_all(message.as_bytes());
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
clients can receive messages from each other
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
