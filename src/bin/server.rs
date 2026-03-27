#![allow(unused)]
use cli_chat_app::utils::parsing;
use std::{
    collections::HashMap,
    io::{Error, Read, Write},
    net::{Shutdown::Both, TcpListener, TcpStream},
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
    const IP_ADDR: &str = "192.168.100.3";
    const PORT: &str = "7878";

    let listener = TcpListener::bind([IP_ADDR, PORT].join(":"))?;
    let messages: Arc<Mutex<Vec<String>>> = Default::default();
    let clients: Arc<Mutex<HashMap<String, TcpStream>>> = Default::default();

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

        // NOTE maybe it's time to introduce some of enums
        thread::spawn(move || {
            loop {
                let mut raw_message = [0; 1024];
                match stream.read(&mut raw_message) {
                    // if client program crush (it will send 0 bytes as a result), then break loop
                    Ok(0) => break,
                    Ok(bytes_readed) => {
                        let detailed_message: String = str::from_utf8(&raw_message[..bytes_readed])
                            .unwrap()
                            .to_string();

                        let (name, mut msg) = parsing(&detailed_message.clone());

                        //TODO maybe make make parsing return enums of commands ?
                        if msg.trim() == String::from("/quit") {
                            // TODO didn't fix if client's program just crush, the stream didn't get shutdown !
                            stream.shutdown(Both).unwrap();
                            break;
                        }

                        if !msg.is_empty() {
                            cloned_messages
                                .lock()
                                .unwrap()
                                .push(detailed_message.clone());

                            for client in cloned_clients.try_lock().unwrap().iter_mut() {
                                if name != *client.0 {
                                    dbg!(&msg);
                                    let _ = client.1.write_all(detailed_message.as_bytes());
                                }
                            }

                            //dbg!(&stream);
                            //dbg!(&message);
                            dbg!(&cloned_messages);
                        }
                    }
                    _ => todo!(),
                }
            }
        });
    }
    Ok(())
}

/*
TODO
use rust features to increase readablity
do error handling

the big boss for now:
    - bind server to wifi, and let client on the same wifi connect to that server
    - need search on how to do that (safely, for now ?)
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
