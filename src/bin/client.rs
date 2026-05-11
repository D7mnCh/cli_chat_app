use cli_chat_app::client::app::App;

fn main() {
    let mut app = App::new();
    let _ = ratatui::run(|terminal| app.run(terminal));
}
#[cfg(test)]
mod testing {}
