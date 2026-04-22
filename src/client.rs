#![allow(unused)]
use crate::utils::NameHandling;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
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
}

impl Networking {
    pub fn new() -> Self {
        Self {
            addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 217, 211)), 7878),
            server_state: ServerState::Disconnected,
        }
    }
}

// NOTE what if i make Clients instead, and add more fields to it ?
impl Client {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            networking: Networking::new(),
        }
    }

    pub fn connect(&mut self) {
        // TODO maybe add dots increaseing or make it to ratatui, (i want the latter)
        print!("Checking server if connected...");
        io::stdout().flush();

        let _checking_connection =
            match TcpStream::connect_timeout(&self.networking.addr, Duration::from_secs(3)) {
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
            let _ = stream.write_all(self.name.as_bytes());
        }
    }

    // NOTE i can change the logic to be more simple ?
    pub fn send_message_to_server(&mut self, client_message: &String) {
        let separator = ":";
        let suffix_msg = format!("{}{}\n", separator, self.name);

        if let ServerState::Connected(stream) = &mut self.networking.server_state {
            if !client_message.is_empty() {
                let detailed_message: String = client_message.clone().add(&suffix_msg).to_string();
                let _ = stream.write_all(detailed_message.as_bytes());
            }
        }
    }
    // if not use shutdown method and just "close the ratatui context", it will sent an error of
    //client program crushes (os error 104)
    pub fn disconnected(&mut self) {
        if let ServerState::Connected(stream) = &mut self.networking.server_state {
            let _ = stream.shutdown(std::net::Shutdown::Both);
        }
    }

    // NOTE the other deive will not received the full messsages, it will make it empty,
    pub fn received_client_msgs(&mut self, messages: Arc<Mutex<Vec<String>>>) -> io::Result<()> {
        if let ServerState::Connected(stream) = &mut self.networking.server_state {
            let mut cloned_stream = stream.try_clone()?;
            let is_server_disconnected: Arc<Mutex<bool>> = Default::default();

            let cloned_is_server_disconnected = Arc::clone(&is_server_disconnected);
            let _checked_server_connection = thread::spawn(move || loop {
                // to prevent cpu 100% usage
                thread::sleep(Duration::from_secs(1));
                if *cloned_is_server_disconnected
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    == true
                {
                    // TODO exit ratatui, and the whole program
                    eprintln!("ERROR: Server disconnected");

                    // NOTE what int parameter means inside this exit function ?
                    std::process::exit(0);
                }
            });

            let cloned_is_server_disconnected = Arc::clone(&is_server_disconnected);
            let cloned_messages = Arc::clone(&messages);

            let _received_client_msgs_thread_handler = thread::spawn(move || loop {
                let mut raw_message = [0; 1024];
                match cloned_stream.read(&mut raw_message) {
                    Ok(0) => {
                        *cloned_is_server_disconnected
                            .lock()
                            .unwrap_or_else(|e| e.into_inner()) = true;
                        break;
                    }
                    Ok(bytes_readed) => {
                        let message: String = str::from_utf8(&raw_message[..bytes_readed])
                            .unwrap_or("")
                            .to_string();

                        let detailed_messages: Vec<&str> = message.split('\n').collect();
                        for detailed_msg in detailed_messages.iter() {
                            let detailed_msg: Vec<&str> = detailed_msg.split(':').collect();
                            if let [msg, name] = detailed_msg[..] {
                                if !msg.is_empty() {
                                    let other_clients_messages = format!("{name}: {msg}");
                                    cloned_messages
                                        .lock()
                                        .unwrap_or_else(|e| e.into_inner())
                                        .push(other_clients_messages.to_string());
                                }
                            }
                        }
                    }
                    _ => todo!(),
                }
            });
        }
        Ok(())
    }
}
