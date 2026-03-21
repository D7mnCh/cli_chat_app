use std::{
    io::{self, BufRead, BufReader, Error, Read, Stdout, Write},
    net::TcpStream,
    ops::Add,
    thread,
    time::Duration,
};

fn get_client_name() -> String {
    print!("enter an name: ");
    let _ = io::stdout().flush();

    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).unwrap();
    buffer
}

fn main() {
    if let Ok(mut stream) = TcpStream::connect("127.0.0.1:7878") {
        let mut cloned_stream = stream.try_clone().unwrap();
        let name = get_client_name();
        loop {
            thread::scope(|service| {
                // received messages from other clients
                service.spawn(|| {
                    //thread::sleep(Duration::from_secs(3));
                    //let mut raw_message = [0; 1024];
                    //let bytes_readed = cloned_stream.read(&mut raw_message).unwrap();
                    //let message: String = str::from_utf8(&raw_message[..bytes_readed])
                    //    .unwrap()
                    //    .to_string();
                    //print!("server send : {message}");
                });

                // send messages to server
                service.spawn(|| {
                    let mut raw_message = [0; 1024];
                    let separator = ":";
                    let sufx_msg = format!("{}{}", separator, name);

                    let bytes_readed = io::stdin().read(&mut raw_message).unwrap();
                    let message: String = str::from_utf8(&raw_message[..bytes_readed])
                        .unwrap_or("")
                        .trim()
                        .to_string()
                        .add(&sufx_msg);
                    dbg!(&message);
                    let _ = stream.write_all(message.as_bytes());
                });
            });
        }
    } else {
        eprintln!("connection lost to host");
    }
}
/*
NOTE if the connection lost then break immediately
*/
