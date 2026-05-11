use cli_chat_app::server::network::Server;

fn main() {
    let mut server = Server::new();
    let _ = server.bind_addr();
    let _ = server.run();
}
#[cfg(test)]
mod test {}
