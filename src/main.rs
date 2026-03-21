#![allow(unused)]
use std::collections::HashMap;
use std::io::{Error, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;

fn main() -> Result<(), Error> {
    const IP_ADDR: &str = "127.0.0.1";
    const PORT: &str = "7878";

    let listener = TcpListener::bind([IP_ADDR, PORT].join(":"))?;
    let messages: Arc<Mutex<Vec<String>>> = Default::default();

    let mut connecters = 0;
    // i don't know exaclty how this for loop is working
    // lazy for loop ?, only happend once ?
    for stream in listener.incoming() {
        let mut stream = stream?;
        let cloned_messages = Arc::clone(&messages);

        thread::spawn(move || {
            loop {
                dbg!(&stream);
                let mut raw_message = [0; 1024];
                let bytes_readed = stream.read(&mut raw_message).unwrap();
                let message: String = str::from_utf8(&raw_message[..bytes_readed])
                    .unwrap_or("")
                    .to_string();
                dbg!(&message);
                //TODO this gonna write to the clinet that write that message ?, i don't want that
                cloned_messages.lock().unwrap().push(message.clone());
                dbg!(&cloned_messages);
                //let _ = stream.write_all(message.as_bytes());
            }
        });

        connecters += 1;
        println!("number of connected clients: {connecters}");
    }
    Ok(())
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
