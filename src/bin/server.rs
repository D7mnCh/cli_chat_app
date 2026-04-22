use cli_chat_app::server::Server;

fn main() {
    let mut server = Server::new();
    let _ = server.bind_addr();
    let _ = server.run();
}
#[cfg(test)]
mod test {
    #[test]
    fn testing_loops() {
        let mut vec_of_messages = Vec::new();
        for i in 0..=500 {
            let sample_message = i.to_string() + ":Server";
            println!("{sample_message}");
            vec_of_messages.push(sample_message);
        }
    }
}
