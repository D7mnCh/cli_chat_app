use cli_chat_app::utils::parsing;
use std::{
    io::{self, BufRead, BufReader, Error, Read, Stdout, Write},
    net::TcpStream,
    ops::Add,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
};

fn get_client_name() -> String {
    print!("enter an name: ");
    let _ = io::stdout().flush();

    let mut name = String::new();
    io::stdin().read_line(&mut name).unwrap();

    String::from(name.trim())
}
fn send_client_name_to_server(name: &String, stream: &mut TcpStream) {
    let _ = stream.write_all(name.as_bytes());
}

fn main() {
    // NOTE when i send msg and only one client is connected, msgs don't get to server ?( it will
    //not display it
    // NOTE what is an IP addr exactly
    // NOTE give me an IP_ADDR of a device when trying to connect, don't make it fixed
    //to accept connections from different devices
    const IP_ADDR: &str = "192.168.100.3";
    const PORT: &str = "7878";
    let client_name = get_client_name();

    if let Ok(mut stream) = TcpStream::connect([IP_ADDR, PORT].join(":")) {
        let mut cloned_stream = stream.try_clone().unwrap();
        send_client_name_to_server(&client_name, &mut stream);
        let mut threads: Vec<JoinHandle<_>> = Vec::new();
        let is_server_disconnected: Arc<Mutex<bool>> = Default::default();

        let cloned_is_server_disconnected = Arc::clone(&is_server_disconnected);
        let checked_server_connection = thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(1));
            if *cloned_is_server_disconnected.lock().unwrap() == true {
                // TODO don't display the msg below when '/quit'
                eprintln!("ERROR: Server disconnected");
                // NOTE what int parameter means inside exit function ?
                std::process::exit(0);
            }
        });
        threads.push(checked_server_connection);

        let cloned_is_server_disconnected = Arc::clone(&is_server_disconnected);
        let received_client_msgs_thread_handler = thread::spawn(move || {
            loop {
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

                        // parse msg and show which one send the msg
                        let (client_name, msg) = parsing(&message);
                        print!("{client_name}: {msg}");
                        //dbg!(&message);
                    }
                    _ => todo!(),
                }
            }
        });
        threads.push(received_client_msgs_thread_handler);

        let send_msg_to_server_thread_handler = thread::spawn(move || {
            loop {
                let mut raw_message = [0; 1024];
                let separator = ":";
                let suffix_msg = format!("{}{}", separator, client_name);

                let bytes_readed = io::stdin().read(&mut raw_message).unwrap();
                let client_message = str::from_utf8(&raw_message[..bytes_readed])
                    .unwrap()
                    .to_string();

                if !client_message.trim().is_empty() {
                    let detailed_message = client_message.add(&suffix_msg);
                    let _ = stream.write_all(detailed_message.as_bytes());
                }

                //dbg!(&detailed_message);
                //dbg!(&client_message);
            }
        });
        threads.push(send_msg_to_server_thread_handler);
        for thread in threads {
            thread.join().unwrap();
        }
    } else {
        eprintln!("ERROR: server shutdown at the moment");
    }
}
#[test]
fn print_array() {
    let array = &[10, 5, 3, 3][..3];
    assert_eq!(array, [10, 5, 3]);
}
/*
NOTE if the connection lost then break immediately
*/
