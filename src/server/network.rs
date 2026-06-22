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
            server_msg @ ServerMessage::Chat { sender, content } => {
                println!("[Info] {sender} message is : \"{content}\"");
                for (client_name, client_stream) in clients.iter_mut() {
                    if sender != client_name {
                        let _ = client_stream.write_all(server_msg.serialize().as_bytes());
                    }
                }
                println!("[Info] broadcasting \"{sender}\" message\n");
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
        let mut server_msg: Option<ServerMessage> = None;

        for other_client_name in clients_names.iter() {
            if requested_name == other_client_name {
                server_msg = Some(ServerMessage::InvalidName(NameValidation::Used));
                is_client_name_reserved = true;
                println!("[Error] client name is been used by other client");
            }
        }

        if !is_client_name_reserved {
            if requested_name.eq_ignore_ascii_case("server") {
                println!("[Error] client name used a reserved name: {requested_name}");
                server_msg = Some(ServerMessage::InvalidName(NameValidation::Reserved))
            } else if requested_name.contains(':') {
                println!("[Error] client name have IllegalChar \":\"");
                server_msg = Some(ServerMessage::InvalidName(NameValidation::IllegalChar(':')))
            } else {
                println!("[Info] name accepted");
                server_msg = Some(ServerMessage::ValidName(requested_name.to_string()))
            };
        }

        if let Some(server_msg) = server_msg {
            Server::send_to_one_client(stream, &server_msg.clone().serialize());
            println!("[Info] sending to {requested_name} name accepted\n");

            return server_msg;
        }

        return ServerMessage::InvalidName(NameValidation::Unknown);
    }

    fn handle_valid_name(
        client_name: &str,
        messages: &mut Vec<String>,
        clients: &mut HashMap<String, TcpStream>,
        stream: &mut TcpStream,
    ) -> Result<()> {
        let new_connection_msg = ServerMessage::ClientConnected(client_name.to_owned());
        println!("[Info] new connection event from \"{client_name}\"");
        Server::broadcast(clients, &new_connection_msg);
        println!("[Info] broadcasting new connection event\n");

        messages.push(new_connection_msg.serialize());

        // sending sample of msgs
        //Server::_sending_sample_msgs(
        //    &mut messages.lock_mutex(),
        //);

        // append this new client to clients collection
        clients.insert(client_name.to_owned(), stream.try_clone()?);
        println!("[Info] appending \"{client_name}\" to clients collection\n");

        Server::send_to_one_client(
            stream,
            &ServerMessage::History(messages.to_owned()).serialize(),
        );
        println!("[Info] sending to \"{client_name}\" messages history");

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

            println!("[Info] disconnection event from \"{client_name}\"");
            Server::broadcast(
                clients,
                &ServerMessage::ClientDisconnected(client_name.to_owned()),
            );
            println!("[Info] broadcasting disconnection event message\n");
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
        println!("[Info] samples messages have being sent to all clients succesfully",);
    }

    pub fn bind_addr(&mut self) -> Result<()> {
        let listener = TcpListener::bind(self.addr)?;
        self.listener = Some(listener);
        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        if let Some(listener) = &mut self.listener {
            for stream in listener.incoming() {
                println!("[Info] new connection\n");
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
                                        println!(
                                            "[Info] get detailed message from client: {client_msg}"
                                        );
                                        // check client name validation
                                        let client_msg = ClientMessage::deserialize(&client_msg);
                                        println!(
                                            "[Info] detailed message after deserialization : {client_msg:?}"
                                        );

                                        match client_msg {
                                            ClientMessage::CheckName(ref requested_client_name) => {
                                                println!("[Info] checking if the requested name is valid");
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
                                            }

                                            ClientMessage::Unknown(unknown_msg) => {
                                                if let Some(ref client_name) = client_name {
                                                    println!(
                                                        "[Info] Unknown msg from \"{client_name}\" : {unknown_msg}"
                                                    );
                                                } else {
                                                    println!("[Info] Unknown msg : {unknown_msg}");
                                                }
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
