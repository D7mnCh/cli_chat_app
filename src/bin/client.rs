use cli_chat_app::utils::parsing;
use std::{
    io::{self, BufRead, BufReader, Error, Read, Stdout, Write},
    net::TcpStream,
    ops::Add,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
};

fn get_client_name(stream: &mut TcpStream) -> String {
    print!("enter an name: ");
    let _ = io::stdout().flush();

    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).unwrap();
    let _ = stream.write_all(buffer.as_bytes());

    String::from(buffer.trim())
}

fn main() {
    // what is an IP addr exactly
    const IP_ADDR: &str = "192.168.100.3";
    const PORT: &str = "7878";

    if let Ok(mut stream) = TcpStream::connect([IP_ADDR, PORT].join(":")) {
        let mut cloned_stream = stream.try_clone().unwrap();
        let name = get_client_name(&mut stream);
        let mut threads: Vec<JoinHandle<_>> = Vec::new();

        // NOTE i think i will make third thread to check if server is disconnected on every sec, and i will use
        //process::exit(0) or something
        let is_server_disconnected: Arc<Mutex<bool>> = Default::default();
        let cloned_is_server_disconnected = Arc::clone(&is_server_disconnected);

        let received_client_msgs_thread_handler = thread::spawn(move || {
            loop {
                let mut raw_message = [0; 1024];
                // NOTE this will block because i can't send to myself so it keep wataing server to send him some msg
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

                        // parse msg and show which one send the msg
                        let (name, msg) = parsing(&message);
                        print!("{name}: {msg}");
                        //dbg!(&message);
                    }
                    _ => todo!(),
                }
            }
        });
        // NOTE if the server crash, i need also to end reading from stdin
        threads.push(received_client_msgs_thread_handler);
        let send_msg_to_server_thread_handler = thread::spawn(move || {
            loop {
                if *is_server_disconnected.lock().unwrap() == false {
                    let mut raw_message = [0; 1024];
                    let separator = ":";
                    let sufx_msg = format!("{}{}", separator, name);

                    let bytes_readed = io::stdin().read(&mut raw_message).unwrap();
                    let client_message = str::from_utf8(&raw_message[..bytes_readed])
                        .unwrap()
                        .to_string();

                    if !client_message.trim().is_empty() {
                        let detailed_message = client_message.add(&sufx_msg);
                        let _ = stream.write_all(detailed_message.as_bytes());
                    }

                    //dbg!(&detailed_message);
                    //dbg!(&message);
                } else {
                    break;
                }
            }
        });
        threads.push(send_msg_to_server_thread_handler);
        for thread in threads {
            thread.join().unwrap();
        }
    } else {
        eprintln!("connection lost to host");
    }
}
/*
NOTE if the connection lost then break immediately
*/
