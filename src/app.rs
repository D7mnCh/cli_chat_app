use crate::client::Client;
use crate::ui::{InputMode, Ui};
use std::thread;
use std::{
    io::Error,
    sync::{Arc, Mutex},
    time::Duration,
};

use crossterm::event::{self, KeyCode};
use ratatui::DefaultTerminal;

pub struct App {
    ui: Ui,
    pub client: Client,
    pub messages: Arc<Mutex<Vec<String>>>,
}

impl App {
    pub fn new() -> Self {
        let ui = Ui::new();
        let mut client = Client::new();

        client.connect();
        client.get_client_name();
        client.send_client_name_to_server();

        Self {
            ui,
            client,
            messages: Default::default(),
        }
    }
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<(), Error> {
        self.client.received_client_msgs(Arc::clone(&self.messages));
        // NOTE i need something else other then sleeping
        // sleep to load every message so i can select the last one
        thread::sleep(Duration::from_secs(1));
        self.ui
            .select_last_message(&mut self.messages.lock().unwrap());

        loop {
            match self.client.networking.stream {
                Some(_) => {
                    terminal.draw(|frame| self.ui.render(frame, &self.messages.lock().unwrap()))?;

                    // 200 to prevent 100% CPU usage
                    if event::poll(Duration::from_millis(200))? {
                        if let Some(key) = event::read()?.as_key_press_event() {
                            match self.ui.input_mode {
                                InputMode::Normal => match key.code {
                                    KeyCode::Char('e') => {
                                        self.ui.input_mode = InputMode::Editing;
                                    }
                                    KeyCode::Char('q') => {
                                        return Ok(());
                                    }
                                    // NOTE i can't scroll when client connect with large
                                    //msg history
                                    KeyCode::Char('k') => self.ui.scroll_up(),
                                    KeyCode::Char('j') => {
                                        self.ui.scroll_down(self.messages.lock().unwrap().len())
                                    }
                                    _ => {}
                                },
                                InputMode::Editing => match key.code {
                                    KeyCode::Enter => {
                                        if self.ui.input == "/quit" {
                                            return Ok(());
                                        }
                                        if self.ui.input.is_empty() {
                                            continue;
                                        }
                                        // NOTE i think there's better methods then this
                                        self.ui.submit_message(
                                            &mut self.client.name,
                                            &mut self.messages.lock().unwrap(),
                                        );
                                        self.ui.select_last_message(
                                            &mut self.messages.lock().unwrap(),
                                        );

                                        self.client.send_message_to_server(&mut self.ui.input);
                                        self.ui.input.clear();
                                    }
                                    KeyCode::Char(to_insert) => {
                                        self.ui.enter_char(to_insert);
                                    }
                                    KeyCode::Backspace => self.ui.delete_char(),
                                    KeyCode::Left => self.ui.move_cursor_left(),
                                    KeyCode::Right => self.ui.move_cursor_right(),
                                    KeyCode::Esc => self.ui.input_mode = InputMode::Normal,
                                    _ => {}
                                },
                            }
                        }
                    }
                }
                None => {
                    eprintln!("ERROR: server shutdown at the moment");
                    return Ok(());
                }
            }
        }
    }
}
#[cfg(test)]
mod test {}
/*
- no more features, organize your project, and try understand the ratatui library
TODO
- remove unwrap :)
- on the first messages, it take why so long to scroll down, but with scroll_up it doesn't
(when hitting the ground)
- make input expand when hits edge, like either expand in width or height
    - or just make a limit of chars
- when run server let user input ip address
- switch to using async
- send me the mesgs histroy just once when client connect
*/
