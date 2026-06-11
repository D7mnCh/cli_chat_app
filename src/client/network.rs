use crate::shared_utils::{LockClean, NameValidation};
use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::sync::mpsc::Sender;
use std::{
    io::{self, Read, Write},
    sync::{Arc, Mutex},
    thread,
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
            addr: SocketAddr::from((Ipv4Addr::new(0, 0, 0, 0), 7878)),
            server_state: ServerState::Disconnected,
        }
    }

    pub fn send_to_server(&mut self, msg: &String) {
        if let ServerState::Connected(stream) = &mut self.server_state {
            let _ = stream.write_all(msg.as_bytes());
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
        new_message_tx: Sender<bool>,
    ) -> io::Result<()> {
        if let ServerState::Connected(stream) = &mut self.networking.server_state {
            let mut cloned_stream = stream.try_clone()?;
            let cloned_messages = Arc::clone(&messages);

            let _received_client_msgs_thread_handler = thread::spawn(move || loop {
                let mut raw_message = [0; 1024];
                match cloned_stream.read(&mut raw_message) {
                    Ok(0) => {
                        let _ = server_state_tx.send(ServerState::Disconnected);
                        break;
                    }

                    Ok(bytes_read) => {
                        let message_buffer: String = str::from_utf8(&raw_message[..bytes_read])
                            .unwrap_or("")
                            .to_string();

                        let messages_lines: Vec<&str> = message_buffer.split('\n').collect();
                        for line in messages_lines.iter() {
                            let server_msg: Vec<&str> = line.splitn(4, ':').collect();
                            match server_msg[..] {
                                ["server", "success", "valid_name", content] => {
                                    let _ = name_validation_tx
                                        .send(NameValidation::Valid(content.to_string()));
                                }
                                ["server", "event", content] => {
                                    let _ = new_message_tx.send(true);
                                    let msg = format!("server: {content}");
                                    cloned_messages.lock_mutex().push(msg);
                                }
                                ["server", "error", "reserved_name"] => {
                                    let _ = name_validation_tx.send(NameValidation::Reserved);
                                }
                                ["server", "error", "used_name"] => {
                                    let _ = name_validation_tx.send(NameValidation::Used);
                                }
                                ["server", "error", "empty_name"] => {
                                    let _ = name_validation_tx.send(NameValidation::Empty);
                                }
                                ["server", "error", "illegalchar", character] => {
                                    if let Some(character) = character.chars().next() {
                                        let _ = name_validation_tx
                                            .send(NameValidation::IllegalChar(character));
                                    }
                                }
                                ["client", "chat", sender, content] if !content.is_empty() => {
                                    let chat_message = format!("{sender}: {content}");

                                    cloned_messages
                                        .lock()
                                        .unwrap_or_else(|e| e.into_inner())
                                        .push(chat_message);
                                    let _ = new_message_tx.send(true);
                                }
                                // ignore this case that could cuz from split('\n') method
                                [""] => {}
                                ref uknown => {
                                    println!("[Warn]: unkown msg: {uknown:?}")
                                }
                            }
                        }
                        let _ = new_message_tx.send(false);
                    }
                    Err(e) => println!("[Error]: {e}"),
                }
            });
        }
        Ok(())
    }
}
