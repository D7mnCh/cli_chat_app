use crate::client::{Client, ServerState};
use crate::ui::{InputMode, InputState, Ui};
use crate::utils::{parsing_name, NameHandling};
use std::{
    io::Error,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crossterm::event::{self, KeyCode};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};
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
        let _ = client.connect();

        Self {
            ui,
            client,
            messages: Default::default(),
        }
    }
    // sent to local messages history
    pub fn submit_message(&mut self) {
        let detailed_msg = format!("{}: {}", self.client.name, self.ui.input.buffer);
        self.messages
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(detailed_msg);
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<(), Error> {
        //TODO to make it here you must use ratatui widget (next)
        //let _ = self.client.connect();
        match self.client.networking.server_state {
            ServerState::Connected(_) => {
                let _ = self.client.received_client_msgs(Arc::clone(&self.messages));
                loop {
                    // NOTE if you used other terminla.draw method it will make like another buffer
                    terminal.draw(|frame| {
                        self.ui.render(
                            frame,
                            &mut self.messages.lock().unwrap_or_else(|e| e.into_inner()),
                        )
                    })?;

                    // 200 to prevent 100% CPU usage
                    if event::poll(Duration::from_millis(200))? {
                        if let Some(key) = event::read()?.as_key_press_event() {
                            match self.ui.input.mode {
                                InputMode::Normal => match key.code {
                                    KeyCode::Char('e') => self.ui.input.mode = InputMode::Editing,
                                    KeyCode::Char('q') => {
                                        self.client.disconnected();
                                        return Ok(());
                                    }
                                    KeyCode::Char('k') => self.ui.scroll_up(),
                                    KeyCode::Char('j') => self.ui.scroll_down(
                                        self.messages
                                            .lock()
                                            .unwrap_or_else(|e| e.into_inner())
                                            .len(),
                                    ),
                                    _ => {}
                                },

                                InputMode::Editing => match key.code {
                                    KeyCode::Enter => match self.ui.input_state {
                                        InputState::EnterName => {
                                            let prgh: Option<Paragraph> =
                                                match parsing_name(&self.ui.input.buffer) {
                                                    NameHandling::Empty => {
                                                        Ui::name_err_msg(NameHandling::Empty)
                                                    }
                                                    NameHandling::Reserved => {
                                                        self.ui.input.buffer.clear();
                                                        self.ui.input.reset_cursor();
                                                        Ui::name_err_msg(NameHandling::Reserved)
                                                    }
                                                    NameHandling::Valid => {
                                                        self.client.name =
                                                            self.ui.input.buffer.clone();
                                                        self.client.send_client_name_to_server();

                                                        self.ui.input.buffer.clear();
                                                        self.ui.input.reset_cursor();
                                                        self.ui.select_last_message(
                                                            &self
                                                                .messages
                                                                .lock()
                                                                .unwrap_or_else(|e| e.into_inner()),
                                                        );
                                                        self.ui.input_state = InputState::Chatting;
                                                        Ui::name_err_msg(NameHandling::Valid)
                                                    }
                                                };

                                            if let Some(error_msg) = prgh {
                                                terminal.draw(|frame| {
                                                    frame.render_widget(error_msg, frame.area());
                                                })?;
                                                thread::sleep(Duration::from_millis(1700));
                                            }
                                            // TODO later, i guess make user input in order to retry,
                                            //if not then can you make invalid have 1 sec else more ?
                                            continue;
                                        }
                                        InputState::Chatting => {
                                            if self.ui.input.buffer == "/quit" {
                                                self.client.disconnected();
                                                return Ok(());
                                            }
                                            // NOTE display in ratatui terminal context
                                            if self.ui.input.buffer == "/msgs" {
                                                dbg!(&self
                                                    .messages
                                                    .lock()
                                                    .unwrap_or_else(|e| e.into_inner()));
                                                self.ui.input.buffer.clear();
                                                continue;
                                            }
                                            if self.ui.input.buffer.is_empty() {
                                                continue;
                                            }
                                            self.submit_message();
                                            self.ui.select_last_message(
                                                &mut self
                                                    .messages
                                                    .lock()
                                                    .unwrap_or_else(|e| e.into_inner()),
                                            );

                                            self.client
                                                .send_message_to_server(&mut self.ui.input.buffer);
                                            self.ui.input.buffer.clear();
                                            self.ui.input.reset_cursor();
                                        }
                                    },
                                    KeyCode::Char(to_insert) => {
                                        self.ui.input.enter_char(to_insert);
                                    }
                                    KeyCode::Esc => self.ui.input.mode = InputMode::Normal,
                                    KeyCode::Backspace => self.ui.input.delete_char(),
                                    KeyCode::Left => self.ui.input.move_cursor_left(),
                                    KeyCode::Right => self.ui.input.move_cursor_right(),
                                    _ => {}
                                },
                            }
                        }
                    }
                }
            }

            ServerState::Disconnected => loop {
                let paragraph = Paragraph::new("server is not running at the moment")
                    .centered()
                    .block(Block::bordered().title_top(Line::from("Error").centered()));
                terminal.draw(|frame| {
                    frame.render_widget(paragraph, frame.area());
                })?;

                if event::poll(Duration::from_millis(200))? {
                    if let Some(key) = event::read()?.as_key_press_event() {
                        match key.code {
                            KeyCode::Char(_) | KeyCode::Enter => {
                                return Ok(());
                            }
                            _ => {}
                        }
                    }
                }
            },
        }
    }
}
#[cfg(test)]
mod test {}
/*
- no more features, organize your project, and try understand the ratatui library
TODO
- when other clients send message, for now i want to select last message
- try to understand how scrolling work and also the other things in ratatui
- what to do if name have being used by other clients
- on the first messages, it take why so long to scroll down, but with scroll_up it doesn't
(when hitting the ground) -> cause you are literally selecting the messages
- make input expand when hits edge, like either expand in width or height
    - or just make a limit of chars
- when run server let user input ip address
- switch to using async
*/
