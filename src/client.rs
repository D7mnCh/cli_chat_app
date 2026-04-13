#![allow(unused)]
use crate::utils::parsing;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::{
    io::{self, BufRead, BufReader, Error, Read, Stdout, Write},
    ops::Add,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
};

pub struct Client {
    // NOTE i never used them, i need to put the recieved message from server to this
    pub name: String,
    pub networking: Networking,
}
pub struct Networking {
    addr: SocketAddr,
    pub stream: Option<TcpStream>,
}
impl Networking {
    pub fn new() -> Self {
        Self {
            addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 100, 3)), 7878),
            stream: None,
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
        // TODO check first if the addr is available
        self.networking.stream = Some(
            TcpStream::connect(self.networking.addr).expect("ERROR: server shutdown at the moment"),
        );
    }

    pub fn get_client_name(&mut self) {
        print!("enter an name: ");
        let _ = io::stdout().flush();

        let mut name = String::new();
        io::stdin().read_line(&mut name).unwrap();

        self.name = String::from(name.trim())
    }

    pub fn send_client_name_to_server(&mut self) {
        if let Some(stream) = &mut self.networking.stream {
            let _ = stream.write_all(self.name.as_bytes());
        }
    }

    pub fn send_message_to_server(&mut self, client_message: &String) {
        let separator = ":";
        let suffix_msg = format!("{}{}", separator, self.name);

        if let Some(stream) = &mut self.networking.stream {
            if !client_message.trim().is_empty() {
                let detailed_message: String = client_message.clone().add(&suffix_msg).to_string();
                let _ = stream.write_all(detailed_message.as_bytes());
            }
        }
    }

    // NOTE what the thing that blocks my program in this method
    pub fn received_client_msgs(&mut self, messages: Arc<Mutex<Vec<String>>>) {
        match &self.networking.stream {
            Some(stream) => {
                let mut cloned_stream = stream.try_clone().unwrap();
                let is_server_disconnected: Arc<Mutex<bool>> = Default::default();

                let cloned_is_server_disconnected = Arc::clone(&is_server_disconnected);
                let _checked_server_connection = thread::spawn(move || loop {
                    // to prevent cpu 100% usage
                    thread::sleep(Duration::from_secs(1));
                    if *cloned_is_server_disconnected.lock().unwrap() == true {
                        // TODO don't display the msg below when '/quit'
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
                            *cloned_is_server_disconnected.lock().unwrap() = true;
                            break;
                        }
                        Ok(bytes_readed) => {
                            let message: String = str::from_utf8(&raw_message[..bytes_readed])
                                .unwrap()
                                .trim()
                                .to_string();

                            let (client_name, msg) = parsing(&message);

                            let other_clients_messages = format!("{client_name}: {msg}");
                            cloned_messages.lock().unwrap().push(other_clients_messages);
                        }
                        _ => unreachable!(),
                    }
                });
            }
            None => {
                eprintln!("ERROR: server shutdown at the moment");
            }
        }
    }
}
