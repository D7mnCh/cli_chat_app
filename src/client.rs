#![allow(unused)]
use crate::app::NameValidation;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::sync::mpsc::{self, Sender, TryRecvError};
use std::{
    io::{self, BufRead, BufReader, Error, Read, Stdout, Write},
    ops::Add,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
};

pub struct Client {
    pub name: String,
    pub networking: Networking,
}

pub enum ServerState {
    Connected(TcpStream),
    Disconnected,
}

pub struct Networking {
    addr: SocketAddr,
    pub server_state: ServerState,
    pub server_disconned: Arc<Mutex<bool>>,
}

impl Networking {
    pub fn new() -> Self {
        Self {
            server_disconned: Default::default(),
            addr: SocketAddr::from((Ipv4Addr::new(192, 168, 190, 211), 7878)),
            server_state: ServerState::Disconnected,
        }
    }
}

impl Client {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            networking: Networking::new(),
        }
    }

    pub fn connect(&mut self) {
        let _checking_connection =
            match TcpStream::connect_timeout(&self.networking.addr, Duration::from_secs(2)) {
                Ok(connection) => {
                    self.networking.server_state = ServerState::Connected(connection);
                }
                Err(_) => {
                    self.networking.server_state = ServerState::Disconnected;
                }
            };
    }

    pub fn send_client_name_to_server(&mut self) {
        if let ServerState::Connected(stream) = &mut self.networking.server_state {
            let name_msg = format!("name:{}\n", self.name);
            let _ = stream.write_all(name_msg.as_bytes());
        }
    }

    pub fn send_message_to_server(&mut self, client_message: &String) {
        if let ServerState::Connected(stream) = &mut self.networking.server_state {
            if !client_message.is_empty() {
                let separator = ":";
                let suffix_msg = format!("{}{}\n", separator, client_message);
                let detailed_message: String = self.name.clone().add(&suffix_msg).to_string();
                let _ = stream.write_all(detailed_message.as_bytes());
            }
        }
    }

    // if not use shutdown method and just "close the ratatui context", it will sent an error of
    //client program crushes (os error 104)
    pub fn disconnected(&self) {
        if let ServerState::Connected(stream) = &self.networking.server_state {
            let _ = stream.shutdown(std::net::Shutdown::Both);
        }
    }

    pub fn handle_msgs(
        &mut self,
        messages: Arc<Mutex<Vec<String>>>,
        server_state_tx: Sender<ServerState>,
        name_validation_tx: Sender<NameValidation>,
    ) -> io::Result<()> {
        if let ServerState::Connected(stream) = &mut self.networking.server_state {
            let mut cloned_stream = stream.try_clone()?;
            let cloned_messages = Arc::clone(&messages);

            let _received_client_msgs_thread_handler = thread::spawn(move || loop {
                let mut raw_message = [0; 1024];
                match cloned_stream.read(&mut raw_message) {
                    Ok(0) => {
                        server_state_tx.send(ServerState::Disconnected);
                        break;
                    }

                    Ok(bytes_read) => {
                        let message_buffer: String = str::from_utf8(&raw_message[..bytes_read])
                            .unwrap_or("")
                            .to_string();

                        let messages_lines: Vec<&str> = message_buffer.split('\n').collect();
                        for line in messages_lines.iter() {
                            let fields: Vec<&str> = line.split(':').collect();

                            if let [name, msg] = fields[..] {
                                match (name, msg) {
                                    ("server", "reserved") => {
                                        name_validation_tx.send(NameValidation::Reserved);
                                    }
                                    ("server", "used") => {
                                        name_validation_tx.send(NameValidation::Used);
                                    }
                                    ("server", "empty") => {
                                        name_validation_tx.send(NameValidation::Empty);
                                    }
                                    ("server", "valid") => {
                                        name_validation_tx
                                            .send(NameValidation::Valid(String::new()));
                                    }
                                    (_, msg) if !msg.is_empty() || msg.trim() != "" => {
                                        let chat_message = format!("{name}: {msg}");

                                        cloned_messages
                                            .lock()
                                            .unwrap_or_else(|e| e.into_inner())
                                            .push(chat_message);
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    Err(e) => println!("[Error]: {e}"),
                }
            });
        }
        Ok(())
    }
}
