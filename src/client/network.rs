use super::channels::ChannelSenders;
use crate::shared_utils::{LockClean, NameValidation, ServerMessage};
use std::io::{BufRead, BufReader};
use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::{
    io::{self, Write},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

pub struct Client {
    // didn't use &'static str, cuz when creating client instatnce, i won't have client name (empty &str)
    //and i can't mutate &'static str after that
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
            addr: SocketAddr::from((Ipv4Addr::new(0, 0, 0, 0), 8080)),
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
    //client program crashes (os error 104), don't know if same error for both linux and windows
    pub fn disconnected(&mut self) {
        self.networking.server_state = ServerState::Disconnected;
        if let ServerState::Connected(stream) = &self.networking.server_state {
            let _ = stream.shutdown(std::net::Shutdown::Both);
        }
    }

    pub fn handle_msgs(
        &mut self,
        messages: Arc<Mutex<Vec<String>>>,
        // the reader thread gonna use channel senders, not network struct, so take ownership
        channel_senders: ChannelSenders,
    ) -> io::Result<()> {
        if let ServerState::Connected(stream) = &self.networking.server_state {
            let cloned_stream = stream.try_clone()?;
            // Bufreader gonna store what it reads into a buffer, with that buffer you can perform mutiple operations
            // the std libray said that Bufreader can prove the speed of my prgoram if i have small(socket content)
            //and repeated read calls
            let buffer = BufReader::new(cloned_stream);
            let cloned_messages = Arc::clone(&messages);

            let _received_client_msgs_thread_handler = thread::spawn(move || {
                // this for loop will keep iterating until it founds EOF
                // explanation: for loops gonna call next on every iteration, if calling next yeilds to None
                //then break. in our case, it'll appear if EOF happen
                for line in buffer.lines() {
                    match line {
                        Ok(line) => {
                            handle_server_msgs(
                                line,
                                &mut cloned_messages.lock_mutex(),
                                &channel_senders,
                            );
                        }

                        // on Windows, instead of reaturning Ok(0) like in linux when a stream is
                        //dead it will return Error with (Os error 10054) message
                        Err(e) if e.raw_os_error() == Some(10054) => {
                            let _ = channel_senders
                                .server_state_tx
                                .send(ServerState::Disconnected);
                            break;
                        }

                        Err(e) => {
                            eprintln!("[READER ERROR]: {e}");
                            std::process::exit(1);
                        }
                    }
                }

                // if calling next on buffer.lines() yields to None (means EOF), then the for loop will break
                let _ = channel_senders
                    .server_state_tx
                    .send(ServerState::Disconnected);
            });
        }
        Ok(())
    }
}

fn handle_server_msgs(msg: String, messages: &mut Vec<String>, channel_senders: &ChannelSenders) {
    match ServerMessage::deserialize(msg.clone()) {
        // handling invalid name
        ServerMessage::InvalidName(NameValidation::Empty) => {
            let _ = channel_senders
                .name_validation_tx
                .send(NameValidation::Empty);
        }
        ServerMessage::InvalidName(NameValidation::Used) => {
            let _ = channel_senders
                .name_validation_tx
                .send(NameValidation::Used);
        }
        ServerMessage::InvalidName(NameValidation::Reserved) => {
            let _ = channel_senders
                .name_validation_tx
                .send(NameValidation::Reserved);
        }
        ServerMessage::InvalidName(NameValidation::IllegalChar(c)) => {
            let _ = channel_senders
                .name_validation_tx
                .send(NameValidation::IllegalChar(c));
        }

        ServerMessage::ValidName(name) => {
            let _ = channel_senders
                .name_validation_tx
                .send(NameValidation::Valid(name));
        }

        ServerMessage::Chat { sender, content } => {
            let chat_message = format!("{sender}: {content}");

            messages.push(chat_message);
            let _ = channel_senders.new_message_tx.send(true);
        }

        // handling events messages
        ServerMessage::ClientConnected(client_name) => {
            let _ = channel_senders.new_message_tx.send(true);
            let msg = format!("server: {client_name} connected!");
            messages.push(msg);
        }
        // NOTE for now i don't know where i am handling other client disconnected msg
        ServerMessage::ClientDisconnected(client_name) => {
            let msg = format!("server: {client_name} disconnected!");
            messages.push(msg);
        }

        ServerMessage::Unknown(_msg) => {
            //dbg!(&_msg);
        }
        _ => unreachable!(
            "caller should not invoke ServerMessage::History, it doesn't
            have any deserialization"
        ),
    }
}
