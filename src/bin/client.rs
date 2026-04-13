use cli_chat_app::app::App;

fn main() {
    let mut app = App::new();
    let _ = ratatui::run(|terminal| app.run(terminal));
}
