use crate::client::{Client, ServerState};
use crate::ui::{InputMode, InputState, Ui};
use std::sync::mpsc::{self};
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

pub enum NameValidation {
    Empty,
    Reserved,
    Valid(String),
    Used,
}

pub struct App {
    ui: Ui,
    pub client: Client,
    pub messages: Arc<Mutex<Vec<String>>>,
}

impl App {
    pub fn new() -> Self {
        let ui = Ui::new();
        let client = Client::new();

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
        let _ = self.client.connect();
        let (server_state_tx, server_state_rx) = mpsc::channel::<ServerState>();
        let (name_validation_tx, name_validation_rx) = mpsc::channel::<NameValidation>();
        let _ = self.client.handle_msgs(
            Arc::clone(&self.messages),
            server_state_tx,
            name_validation_tx,
        );

        loop {
            match self.client.networking.server_state {
                ServerState::Connected(_) => {
                    // NOTE if you used other terminla.draw method it will make like another buffer
                    terminal.draw(|frame| {
                        self.ui.render(
                            frame,
                            &mut self.messages.lock().unwrap_or_else(|e| e.into_inner()),
                        );
                    })?;
                    // check if server disconnected
                    if let Ok(_) = server_state_rx.try_recv() {
                        self.client.networking.server_state = ServerState::Disconnected;
                        continue;
                    }

                    // 200 to prevent 100% CPU usage
                    if event::poll(Duration::from_millis(200))? {
                        if let Some(key) = event::read()?.as_key_press_event() {
                            match self.ui.input.mode {
                                InputMode::Normal => match key.code {
                                    KeyCode::Char('i') => self.ui.input.mode = InputMode::Editing,
                                    KeyCode::Char('q') => {
                                        self.client.disconnected();
                                        return Ok(());
                                    }
                                    // NOTE scrolling didn't work, i think some things is resseting it
                                    KeyCode::Char('k') => self.ui.vertical_scrolling.prev(),
                                    KeyCode::Char('j') => self.ui.vertical_scrolling.next(),
                                    _ => {}
                                },

                                InputMode::Editing => match key.code {
                                    KeyCode::Enter => match self.ui.input_state {
                                        InputState::EnterName => {
                                            self.client.name = self.ui.input.buffer.clone();
                                            self.client.send_client_name_to_server();

                                            let prgh: Option<Paragraph> =
                                                match name_validation_rx.recv().unwrap() {
                                                    NameValidation::Empty => {
                                                        Ui::name_err_msg(NameValidation::Empty)
                                                    }
                                                    NameValidation::Reserved => {
                                                        self.ui.input.buffer.clear();
                                                        self.ui.input.reset_cursor();
                                                        Ui::name_err_msg(NameValidation::Reserved)
                                                    }
                                                    NameValidation::Used => {
                                                        self.ui.input.buffer.clear();
                                                        self.ui.input.reset_cursor();
                                                        Ui::name_err_msg(NameValidation::Used)
                                                    }
                                                    NameValidation::Valid(_) => {
                                                        self.client.name =
                                                            self.ui.input.buffer.clone();
                                                        self.ui.input_state = InputState::Chatting;

                                                        self.ui.input.buffer.clear();
                                                        self.ui.input.reset_cursor();
                                                        Ui::name_err_msg(NameValidation::Valid(
                                                            String::new(),
                                                        ))
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
                                            // NOTE you need window height for adjusment
                                            self.ui.vertical_scrolling.last();

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
                ServerState::Disconnected => {
                    let paragraph = Paragraph::new("server is not running at the moment")
                        .centered()
                        .block(Block::bordered().title_top(Line::from("Error").centered()));
                    terminal.draw(|frame| {
                        frame.render_widget(paragraph, frame.area());
                    })?;
                    self.client.connect();

                    if let Some(key) = event::read()?.as_key_press_event() {
                        match key.code {
                            KeyCode::Char(_) | KeyCode::Enter => {
                                return Ok(());
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}
#[cfg(test)]
mod test {}
/*
 - no more features, organize your project, and try understand the ratatui library
 - ratatui examples is your friends for ui

 TODO Features:
 - Ui:
 - i need logging in ratatui context, my project will not scale well if i not did that
 - messages pop out from bottom to top
 - adjust scrolling with messages height
 - when other clients sned nessage, for now i want to select last message
 - make input expand when hits edge, like either expand in width or height
    - or just make a limit of chars
- make logs in popout window (see ratatui examples)

 - not Ui:
 - make the code more readble
 - retry connection
 - what to do if name have being used by other clients
 - don't allow clients have same names (test it, without)
 - when run server let user input ip address
 - sometimes, the timer freeze for 1sec
 - when i suspend server, what will happen if another device connected

TODO
- next (adding horizontal moving on input area(i think it's should be easy))
- select the last message when entering the app
- switch to using async (later)
- i wanna know when to use channels and smart pointer like Arc + Mutex
*/
