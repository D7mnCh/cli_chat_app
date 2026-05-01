//#![allow(unused)]
use crate::{app::NameValidation, utils::parsing_name_server};
use std::{
    collections::HashMap,
    io::{self, Read, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream},
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
            addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 7878),
            listener: None,
            messages: Default::default(),
            clients: Default::default(),
        }
    }

    pub fn bind_addr(&mut self) -> io::Result<()> {
        let listener = TcpListener::bind(self.addr)?;
        self.listener = Some(listener);
        Ok(())
    }

    fn send_message_history(
        stream: &mut TcpStream,
        messages: &mut Vec<String>,
        client_name: &String,
    ) {
        // sending sample of messages
        //let mut vec_of_messages = Vec::new();
        //for i in 0..=20 {
        //    let sample_message = i.to_string() + ":Server";
        //    vec_of_messages.push(sample_message);
        //}
        //for sample_message in vec_of_messages.iter() {
        //    let _ = stream.write_all((sample_message.to_owned() + "\n").as_bytes());
        //}

        if messages.is_empty() {
            return;
        } else {
            for detailed_msg in messages.iter() {
                let _ = stream.write_all((detailed_msg.to_owned() + "\n").as_bytes());
            }
            println!(
                "[Log]: messages history have been sent succesfully to {}",
                client_name
            );
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        if let Some(listener) = &mut self.listener {
            for stream in listener.incoming() {
                println!("[Log]: new connection");
                // using match and not using propagation because if one client get me an error,
                //the whole server is gonna crush, and that's bad
                match stream {
                    Ok(mut s) => {
                        let cloned_messages = Arc::clone(&self.messages);
                        let cloned_clients = Arc::clone(&self.clients);
                        let mut cloned_stream = s.try_clone()?;
                        let mut client_name: Option<String> = None;

                        thread::spawn(move || -> io::Result<()> {
                            loop {
                                let mut raw_message = [0; 1024];
                                match s.read(&mut raw_message) {
                                    // if client program crush (it will send 0 byte as a result), if
                                    // yes then break his stream loop
                                    Ok(0) => {
                                        if let Some(client_name) = client_name {
                                            println!("{client_name} has disconnected");
                                        }
                                        break Ok(());
                                    }

                                    Ok(bytes_readed) => {
                                        let detailed_messages: String =
                                            str::from_utf8(&raw_message[..bytes_readed])
                                                .unwrap_or_default()
                                                .to_string();

                                        let mut detailed_messages: Vec<&str> =
                                            detailed_messages.split('\n').collect();
                                        detailed_messages.pop();

                                        for detailed_msg in detailed_messages.iter() {
                                            let detailed_msg_cloned = detailed_msg;
                                            let detailed_msg: Vec<&str> =
                                                detailed_msg.split(':').collect();

                                            if let [recieved_name, msg] = detailed_msg[..] {
                                                client_name = Some(msg.to_string());
                                                if recieved_name == "name" {
                                                    let checked_name = parsing_name_server(
                                                        client_name.clone(),
                                                        cloned_clients
                                                            .clone()
                                                            .lock()
                                                            .unwrap()
                                                            .iter()
                                                            .map(|e| e.0)
                                                            .collect(),
                                                    );

                                                    match checked_name {
                                                        NameValidation::Reserved => {
                                                            let _ = cloned_stream.write_all(
                                                                ("server:reserved\n").as_bytes(),
                                                            );
                                                            println!(
                                                                "[Error]: sending name to client"
                                                            );
                                                            continue;
                                                        }
                                                        NameValidation::Used => {
                                                            let _ = cloned_stream.write_all(
                                                                ("server:used\n").as_bytes(),
                                                            );
                                                            println!(
                                                                "[Error]: sending name to client"
                                                            );
                                                            continue;
                                                        }
                                                        // when client quit before sending his name,
                                                        //it will push empty string
                                                        NameValidation::Empty => {
                                                            let _ = cloned_stream.write_all(
                                                                ("server:empty\n").as_bytes(),
                                                            );
                                                            println!(
                                                                "[Error]: sending name to client"
                                                            );
                                                            continue;
                                                        }
                                                        NameValidation::Valid(name) => {
                                                            let _ = cloned_stream.write_all(
                                                                ("server:valid\n").as_bytes(),
                                                            );

                                                            // NOTE the stay connected one will not receive a message that
                                                            //new memebr enter the chat!
                                                            Server::send_message_history(
                                                                &mut s,
                                                                &mut cloned_messages
                                                                    .lock()
                                                                    .unwrap_or_else(|e| {
                                                                        e.into_inner()
                                                                    }),
                                                                &name,
                                                            );
                                                            let new_connection_msg = format!(
                                                                "server:{name} entered the chat"
                                                            );
                                                            cloned_messages
                                                                .lock()
                                                                .unwrap_or_else(|e| e.into_inner())
                                                                .push(new_connection_msg.clone());
                                                            let _ = cloned_stream.write_all(
                                                                (new_connection_msg + "\n")
                                                                    .as_bytes(),
                                                            );

                                                            println!("[Log]: sending name to client: name -> {name}");
                                                            cloned_clients
                                                                .lock()
                                                                // i can get poisoning data (not complete), for now
                                                                //return data always even though she might be currepted
                                                                .unwrap_or_else(|e| e.into_inner())
                                                                .insert(
                                                                    name.clone(),
                                                                    cloned_stream.try_clone()?,
                                                                );
                                                            println!("[Log]: {name} has connected");

                                                            continue;
                                                        }
                                                    };
                                                }

                                                if !msg.is_empty() {
                                                    cloned_messages
                                                        .lock()
                                                        .unwrap_or_else(|e| e.into_inner())
                                                        .push(detailed_msg_cloned.to_string());

                                                    for client in cloned_clients
                                                        .lock()
                                                        .unwrap_or_else(|e| e.into_inner())
                                                        .iter_mut()
                                                    {
                                                        if recieved_name != client.0 {
                                                            let _ = client.1.write_all(
                                                                ((*detailed_msg_cloned).to_owned()
                                                                    + "\n")
                                                                    .as_bytes(),
                                                            );
                                                        }
                                                    }

                                                    //dbg!(&cloned_clients);
                                                    //dbg!(&s);
                                                    //dbg!(&message);
                                                    dbg!(&cloned_messages);
                                                }
                                            }
                                        }
                                    }

                                    Err(e) => {
                                        println!("ERROR: {e}");
                                        // on if client on different device disconnected
                                        //it will keep logging the error
                                        // NOTE Maybe it will be a bug later
                                        break Ok(());
                                    }
                                }
                            }
                        });
                    }

                    Err(e) => {
                        println!("ERROR: {e}");
                        continue;
                    }
                }
            }
        }
        Ok(())
    }
}
/*
TODO
- make only one read
- if clients connected at the same time, it can be multiple clients have the same name
- sometimes getting blocked when i enter a name
- os error 5 search about it, and fix it ofc
- display to others that the client is disconnected or connected
- make server in oop stye
- use rust features to increase readablity
- do error handling (i don't know about that), i think i'll make llm  review my code

- the big boss for now:
    - bind server to wifi, and let client on the same wifi connect to that server,
    need search on how to do that (safely, for now ?)

NOTE
- i think it's better to make if let instead of spamming unwrap()
*/
