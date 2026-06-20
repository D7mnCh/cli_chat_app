use crate::shared_utils::{ClientMessage, LockClean, NameValidation, ServerMessage};

use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Result, Write},
    net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread::{self},
};

pub struct Server {
    addr: SocketAddr,
    listener: Option<TcpListener>,
    messages: Arc<Mutex<Vec<String>>>,
    clients: Arc<Mutex<HashMap<String, TcpStream>>>,
}

impl Server {
    pub fn new() -> Self {
        Self {
            addr: SocketAddr::from((Ipv4Addr::new(0, 0, 0, 0), 8080)),
            listener: None,
            messages: Default::default(),
            clients: Default::default(),
        }
    }

    pub fn broadcast(clients: &mut HashMap<String, TcpStream>, server_msg: &ServerMessage) {
        match server_msg {
            server_msg @ ServerMessage::Chat { sender, .. } => {
                for (client_name, client_stream) in clients.iter_mut() {
                    if sender != client_name {
                        let _ = client_stream.write_all(server_msg.serialize().as_bytes());
                    }
                }
            }

            server_msg @ _ => {
                for (_client_name, client_stream) in clients.iter_mut() {
                    let _ = client_stream.write_all(server_msg.serialize().as_bytes());
                }
            }
        }
    }

    pub fn handle_check_client_name(
        requested_name: &str,
        stream: &mut TcpStream,
        clients_names: &[String],
    ) -> ServerMessage {
        let mut is_client_name_reserved = false;
        let mut server_msg: ServerMessage = ServerMessage::InvalidName(NameValidation::Empty);

        for other_client_name in clients_names.iter() {
            if requested_name == other_client_name {
                server_msg = ServerMessage::InvalidName(NameValidation::Used);
                is_client_name_reserved = true;
            }
        }

        if !is_client_name_reserved {
            if requested_name.eq_ignore_ascii_case("server") {
                server_msg = ServerMessage::InvalidName(NameValidation::Reserved)
            } else if requested_name.is_empty() {
                server_msg = ServerMessage::InvalidName(NameValidation::Empty)
            } else if requested_name.contains(':') {
                server_msg = ServerMessage::InvalidName(NameValidation::IllegalChar(':'))
            } else {
                server_msg = ServerMessage::ValidName(requested_name.to_string())
            };
        }

        Server::send_to_one_client(stream, &server_msg.serialize());

        return server_msg;
    }

    fn handle_valid_name(
        client_name: &str,
        messages: &mut Vec<String>,
        clients: &mut HashMap<String, TcpStream>,
        stream: &mut TcpStream,
    ) -> Result<()> {
        let new_connection_msg = ServerMessage::ClientConnected(client_name.to_owned());
        Server::broadcast(clients, &new_connection_msg);
        messages.push(new_connection_msg.serialize());

        // sending sample of msgs
        //Server::_sending_sample_msgs(
        //    &mut messages.lock_mutex(),
        //);

        // append this new client to clients collection
        clients.insert(client_name.to_owned(), stream.try_clone()?);

        Server::send_to_one_client(
            stream,
            &ServerMessage::History(messages.to_owned()).serialize(),
        );

        Ok(())
    }

    fn handle_client_disconnection(
        client_name: &Option<String>,
        messages: &mut Vec<String>,
        clients: &mut HashMap<String, TcpStream>,
    ) {
        if let Some(client_name) = client_name {
            // NOTE BUG: if clients.len() == 0, any new client
            // can use any name from any disconnected client
            //clients.remove(&client_name.clone());

            messages.push(ServerMessage::ClientDisconnected(client_name.to_owned()).serialize());

            Server::broadcast(
                clients,
                &ServerMessage::ClientDisconnected(client_name.to_owned()),
            );
        }
    }

    // didn't use self cuz i'll move self into a thread that have static life time
    pub fn send_to_one_client(client: &mut TcpStream, msg: &String) {
        let _ = client.write_all(msg.as_bytes());
    }

    // NOTE BUG: i get a bug on client side, i can't scroll up, only in a few seconds
    fn _sending_sample_msgs(msgs: &mut Vec<String>) {
        let mut vec_of_messages = Vec::new();
        let num_of_samples = 100;

        for i in 0..=num_of_samples {
            let sample_message = format!("client:chat:server:{i} a message");
            vec_of_messages.push(sample_message);
        }

        for sample_message in vec_of_messages {
            let msg_to_broadcast = format!("{}", sample_message);
            msgs.push(msg_to_broadcast);
        }
        println!("[Log]:samples messages have being sent to all clients succesfully",);
    }

    pub fn bind_addr(&mut self) -> Result<()> {
        let listener = TcpListener::bind(self.addr)?;
        self.listener = Some(listener);
        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        if let Some(listener) = &mut self.listener {
            for stream in listener.incoming() {
                println!("[Log]:new connection");
                // using match and not using propagation because if one client get me an error and propagate it,
                //the whole server will shutdown, and that's bad
                match stream {
                    Ok(s) => {
                        let cloned_messages = Arc::clone(&self.messages);
                        let cloned_clients = Arc::clone(&self.clients);

                        let mut cloned_stream = s.try_clone()?;
                        let buffer = BufReader::new(s.try_clone()?);

                        thread::spawn(move || -> Result<()> {
                            // i need client_name outside of the for loop, if name is valid
                            // who disonnected
                            let mut client_name: Option<String> = None;

                            for line in buffer.lines() {
                                match line {
                                    Ok(client_msg) => {
                                        // check client name validation
                                        let client_msg = ClientMessage::deserialize(&client_msg);

                                        match client_msg {
                                            ClientMessage::CheckName(ref requested_client_name) => {
                                                // check name validity
                                                let clients_names: Vec<String> = cloned_clients
                                                    .lock_mutex()
                                                    .iter()
                                                    .map(|(client_names, _)| client_names.clone())
                                                    .collect();

                                                let server_msg = Server::handle_check_client_name(
                                                    requested_client_name,
                                                    &mut cloned_stream,
                                                    &clients_names,
                                                );

                                                // handle if name is valid
                                                if matches!(server_msg, ServerMessage::ValidName(_))
                                                {
                                                    client_name =
                                                        Some(requested_client_name.to_owned());

                                                    Server::handle_valid_name(
                                                        &requested_client_name,
                                                        &mut cloned_messages.lock_mutex(),
                                                        &mut cloned_clients.lock_mutex(),
                                                        &mut cloned_stream,
                                                    )?;
                                                }

                                                continue;
                                            }

                                            // NOTE didn't undersatnd why i need to make both to have ref
                                            ref msg @ ClientMessage::Chat {
                                                ref sender,
                                                ref content,
                                            } => {
                                                cloned_messages.lock_mutex().push(msg.serialize());

                                                let server_msg = ServerMessage::Chat {
                                                    sender: sender.to_owned(),
                                                    content: content.to_owned(),
                                                };

                                                Server::broadcast(
                                                    &mut cloned_clients.lock_mutex(),
                                                    &server_msg,
                                                );

                                                dbg!(&cloned_messages);
                                            }

                                            ClientMessage::Unknown(unknown_msg) => {
                                                println!("[Info] Unknown_msg: {unknown_msg}");
                                            }
                                        }
                                    }

                                    // on windows platform, if i shutdown or client crash, it will send to
                                    //server "os error 10054"
                                    Err(e) if e.raw_os_error() == Some(10054) => {
                                        Server::handle_client_disconnection(
                                            &client_name,
                                            &mut cloned_messages.lock_mutex(),
                                            &mut cloned_clients.lock_mutex(),
                                        );

                                        break;
                                    }

                                    Err(e) => {
                                        println!("Read Error: {e}");
                                        break;
                                    }
                                }
                            }

                            // if break then EOF
                            Server::handle_client_disconnection(
                                &client_name,
                                &mut cloned_messages.lock_mutex(),
                                &mut cloned_clients.lock_mutex(),
                            );
                            Ok(())
                        });
                    }

                    Err(e) => {
                        println!("Stream Error: {e}");
                    }
                }
            }
        }

        Ok(())
    }
}
