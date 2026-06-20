use super::name_validation::check_name_validity;
use crate::shared_utils::{LockClean, NameValidation, ServerMessage};

use std::{
    collections::HashMap,
    io::{Read, Result, Write},
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

// TODO intoduce some method
impl Server {
    pub fn new() -> Self {
        Self {
            addr: SocketAddr::from((Ipv4Addr::new(0, 0, 0, 0), 8080)),
            listener: None,
            messages: Default::default(),
            clients: Default::default(),
        }
    }

    // didn't use self cuz i'll move self into a thread that have static life time
    pub fn broadcast(clients: &mut HashMap<String, TcpStream>, msg: &String) {
        for trimed_msg in msg.lines() {
            {
                let suffix_chat = trimed_msg.strip_prefix("client:chat:");
                if let Some(prefix_chat) = suffix_chat {
                    match prefix_chat.splitn(2, ":").collect::<Vec<&str>>()[..] {
                        [chat_sender, ..] => {
                            for (client_name, client_stream) in clients.iter_mut() {
                                if chat_sender != client_name {
                                    let _ = client_stream
                                        .write_all((trimed_msg.to_owned() + "\n").as_bytes());
                                }
                            }
                        }
                        _ => todo!(),
                    }
                    continue;
                }
            }

            for (_client_name, client_stream) in clients.iter_mut() {
                let _ = client_stream.write_all((trimed_msg.to_owned() + "\n").as_bytes());
            }
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
                    Ok(mut s) => {
                        let cloned_messages = Arc::clone(&self.messages);
                        let cloned_clients = Arc::clone(&self.clients);
                        let mut cloned_stream = s.try_clone()?;
                        let mut client_name: Option<String> = None;

                        thread::spawn(move || -> Result<()> {
                            loop {
                                let mut full_serialized_msg: String = String::new();

                                let mut raw_message = [0; 1024];
                                match s.read(&mut raw_message) {
                                    // if client program crush or disconnected (it will send 0 byte as a result)
                                    Ok(0) => {
                                        if let Some(ref client_name) = client_name {
                                            // didn't understand how put this parameter and not just &client_name
                                            cloned_clients
                                                .lock_mutex()
                                                .remove(&client_name.clone());

                                            full_serialized_msg.push_str(
                                                &ServerMessage::ClientDisconnected(
                                                    client_name.to_owned(),
                                                )
                                                .serialize(),
                                            );

                                            cloned_messages.lock_mutex().push(format!(
                                                "server:event:client_disconnected:{client_name}"
                                            ));

                                            Server::broadcast(
                                                &mut cloned_clients.lock_mutex(),
                                                &full_serialized_msg,
                                            );
                                        }

                                        break Ok(());
                                    }

                                    // on Windows, instead of reaturning Ok(0) like in linux when a stream is
                                    //dead it will return Error with (Os error 10054) message
                                    Err(e) if e.raw_os_error() == Some(10054) => {
                                        if let Some(ref client_name) = client_name {
                                            cloned_clients
                                                .lock_mutex()
                                                .remove(&client_name.clone());

                                            full_serialized_msg.push_str(
                                                &ServerMessage::ClientDisconnected(
                                                    client_name.to_owned(),
                                                )
                                                .serialize(),
                                            );

                                            // NOTE why i didn't use ServerMessage ?
                                            cloned_messages.lock_mutex().push(format!(
                                                "server:event:client_disconnected:{client_name}"
                                            ));

                                            Server::broadcast(
                                                &mut cloned_clients.lock_mutex(),
                                                &full_serialized_msg,
                                            );
                                        }

                                        break Ok(());
                                    }

                                    Ok(bytes_read) => {
                                        let message_buffer =
                                            str::from_utf8(&raw_message[..bytes_read])
                                                .unwrap_or_default();

                                        let messages_lines: Vec<&str> =
                                            message_buffer.lines().collect();

                                        for line in messages_lines.iter() {
                                            let received_name_fields: Vec<&str> =
                                                line.splitn(3, ':').collect();

                                            if let ["client", "name", received_client_name] =
                                                received_name_fields[..]
                                            {
                                                client_name =
                                                    Some(received_client_name.to_string());

                                                let name_validation = check_name_validity(
                                                    client_name.as_deref(),
                                                    cloned_clients
                                                        .lock_mutex()
                                                        .iter()
                                                        .map(|(client_names, _)| {
                                                            client_names.clone()
                                                        })
                                                        .collect(),
                                                );

                                                let server_msg = match name_validation {
                                                    NameValidation::Reserved => {
                                                        ServerMessage::InvalidName(
                                                            NameValidation::Reserved,
                                                        )
                                                    }
                                                    NameValidation::Used => {
                                                        ServerMessage::InvalidName(
                                                            NameValidation::Used,
                                                        )
                                                    }
                                                    // when client quit before sending his name,
                                                    //it will push empty string
                                                    NameValidation::Empty => {
                                                        ServerMessage::InvalidName(
                                                            NameValidation::Empty,
                                                        )
                                                    }
                                                    NameValidation::IllegalChar(c) => {
                                                        ServerMessage::InvalidName(
                                                            NameValidation::IllegalChar(c),
                                                        )
                                                    }
                                                    NameValidation::Valid(client_name) => {
                                                        ServerMessage::ValidName(client_name)
                                                    }
                                                };

                                                match server_msg {
                                                    ref msg @ ServerMessage::InvalidName(_) => {
                                                        full_serialized_msg
                                                            .push_str(&msg.serialize());
                                                    }
                                                    ref msg @ ServerMessage::ValidName(
                                                        ref client_name,
                                                    ) => {
                                                        // NOTE why i didn't use ServerMessage serialization here?
                                                        let new_connection_msg = format!(
                                                            "server:event:client_connected:{client_name}"
                                                        );
                                                        cloned_messages
                                                            .lock_mutex()
                                                            .push(new_connection_msg.clone());

                                                        // sending sample of msgs
                                                        //Server::_sending_sample_msgs(
                                                        //    &mut cloned_messages.lock_mutex(),
                                                        //);

                                                        Server::broadcast(
                                                            &mut cloned_clients.lock_mutex(),
                                                            &new_connection_msg,
                                                        );

                                                        // append this new client to clients collection
                                                        cloned_clients.lock_mutex().insert(
                                                            client_name.to_string(),
                                                            cloned_stream.try_clone()?,
                                                        );

                                                        full_serialized_msg
                                                            .push_str(&msg.serialize());

                                                        full_serialized_msg.push_str(
                                                            &ServerMessage::History(
                                                                cloned_messages
                                                                    .lock_mutex()
                                                                    .to_vec(),
                                                            )
                                                            .serialize(),
                                                        );
                                                    }
                                                    _ => {}
                                                }

                                                Server::send_to_one_client(
                                                    &mut cloned_stream,
                                                    &full_serialized_msg,
                                                );

                                                continue;
                                            }

                                            let client_message_fields: Vec<&str> =
                                                line.splitn(4, ':').collect();

                                            if let ["client", "chat", sender, msg] =
                                                client_message_fields[..]
                                            {
                                                let client_message = ServerMessage::Chat {
                                                    sender: sender.to_string(),
                                                    content: msg.to_string(),
                                                };

                                                full_serialized_msg
                                                    .push_str(&client_message.serialize());

                                                let detailed_message =
                                                    format!("client:chat:{sender}:{msg}");
                                                cloned_messages.lock_mutex().push(detailed_message);

                                                Server::broadcast(
                                                    &mut cloned_clients.lock_mutex(),
                                                    &full_serialized_msg,
                                                );

                                                dbg!(&cloned_messages);
                                            }
                                        }
                                    }

                                    Err(e) => {
                                        println!("READING ERROR: {e}");
                                        // if client on different device disconnected
                                        //it will keep logging the error
                                        // NOTE which error exaclty ?
                                        break Ok(());
                                    }
                                }
                            }
                        });
                    }

                    Err(e) => {
                        println!("STREAM ERROR: {e}");
                        break;
                    }
                }
            }
        }
        Ok(())
    }
}
