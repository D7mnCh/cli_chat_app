use cli_chat_app::utils::parsing;
use std::{
    io::{self, BufRead, BufReader, Error, Read, Stdout, Write},
    net::TcpStream,
    ops::Add,
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
    if let Ok(mut stream) = TcpStream::connect("127.0.0.1:7878") {
        let mut cloned_stream = stream.try_clone().unwrap();
        let name = get_client_name(&mut stream);
        let mut threads: Vec<JoinHandle<_>> = Vec::new();

        let received_client_msgs_thread_handler = thread::spawn(move || {
            loop {
                let mut raw_message = [0; 1024];
                // NOTE this will block because i can't send to myself so it keep wataing server to send him some msg
                let bytes_readed = cloned_stream.read(&mut raw_message).unwrap();
                let message: String = str::from_utf8(&raw_message[..bytes_readed])
                    .unwrap()
                    .to_string();

                // parse msg and show which one send the msg
                let (name, msg) = parsing(&message);
                print!("{name}: {msg}");
                //dbg!(&message);
            }
        });
        threads.push(received_client_msgs_thread_handler);
        // send messages to server
        // if server send message, don't make reading from stdin suspend this program
        let send_msg_to_server_thread_handler = thread::spawn(move || {
            loop {
                let mut raw_message = [0; 1024];
                let separator = ":";
                let sufx_msg = format!("{}{}", separator, name);

                // NOTE it will get suspend because of this read from stdin, i am using scope threads
                let bytes_readed = io::stdin().read(&mut raw_message).unwrap();
                let detailed_message: String = str::from_utf8(&raw_message[..bytes_readed])
                    .unwrap_or("")
                    .to_string()
                    .add(&sufx_msg);
                let _ = stream.write_all(detailed_message.as_bytes());

                //dbg!(&detailed_message);
                //dbg!(&message);
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
