#![allow(unused)]
use cli_chat_app::utils::parsing;
use std::{
    collections::HashMap,
    default,
    io::{Error, Read, Write},
    net::{IpAddr, Ipv4Addr, Shutdown::Both, SocketAddr, TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

struct Server {
    addr: SocketAddr,
    listener: Option<TcpListener>,
    messages: Arc<Mutex<Vec<String>>>,
    clients: Arc<Mutex<HashMap<String, TcpStream>>>,
}
impl Server {
    fn new() -> Self {
        Self {
            addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 7878),
            listener: None,
            messages: Default::default(),
            clients: Default::default(),
        }
    }

    fn bind_addr(&mut self) {
        self.listener = Some(TcpListener::bind(self.addr).unwrap());
    }

    fn run(&mut self) {
        if let Some(listener) = &mut self.listener {
            for stream in listener.incoming() {
                let mut stream = stream.unwrap();
                let cloned_messages = Arc::clone(&self.messages);
                let cloned_clients = Arc::clone(&self.clients);

                let name = Server::get_client_name(&mut stream);
                self.clients
                    .try_lock()
                    .unwrap()
                    .insert(name.clone(), stream.try_clone().unwrap());
                println!("{name} has connected");

                Server::send_message_history(&mut stream, &mut cloned_messages.lock().unwrap());

                // NOTE maybe it's time to introduce some of enums
                thread::spawn(move || {
                    loop {
                        let mut raw_message = [0; 1024];
                        match stream.read(&mut raw_message) {
                            // if client program crush (it will send 0 byte as a result), if
                            // yes then break his stream loop
                            Ok(0) => {
                                println!("{name} has disconnected");
                                break;
                            }
                            Ok(bytes_readed) => {
                                let detailed_message: String =
                                    str::from_utf8(&raw_message[..bytes_readed])
                                        .unwrap()
                                        .to_string();

                                let (name, mut msg) = parsing(&detailed_message.clone());

                                if !msg.is_empty() {
                                    cloned_messages
                                        .lock()
                                        .unwrap()
                                        .push(detailed_message.clone());

                                    for client in cloned_clients.try_lock().unwrap().iter_mut() {
                                        if name != *client.0 {
                                            //dbg!(&cloned_messages);
                                            let _ = client.1.write_all(detailed_message.as_bytes());
                                        }
                                    }

                                    //dbg!(&cloned_clients);
                                    //dbg!(&stream);
                                    //dbg!(&message);
                                    dbg!(&cloned_messages);
                                }
                            }
                            Err(e) => println!("{e}"),
                        }
                    }
                });
            }
        }
    }
    fn get_client_name(stream: &mut TcpStream) -> String {
        let mut raw_message = [0; 1024];
        let bytes_readed = stream.read(&mut raw_message).unwrap();
        let detailed_message: String = str::from_utf8(&raw_message[..bytes_readed])
            .unwrap()
            .trim()
            .to_string();

        let name = detailed_message;
        name
    }

    fn send_message_history(stream: &mut TcpStream, messages: &mut Vec<String>) {
        if messages.is_empty() {
            return;
        } else {
            for detailed_msg in messages.iter() {
                // if not sleep the msg will get connected into large string on client side while parsing
                thread::sleep(Duration::from_millis(10));
                let _ = stream.write_all(detailed_msg.as_bytes());
            }
        }
    }
}

fn main() {
    let mut server = Server::new();
    server.bind_addr();
    server.run();
}

/*
TODO
- make server in oop stye
- use rust features to increase readablity
- do error handling (i don't know about that), i think i'll make llm  review my code

- the big boss for now:
    - bind server to wifi, and let client on the same wifi connect to that server,
    need search on how to do that (safely, for now ?)

NOTE
- i think it's better to make if let instead of spamming unwrap()
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
