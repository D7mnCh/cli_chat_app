use crate::{
    client::{
        client_messages::ClientMessages,
        network::{Client, ServerState},
        ui::{InputMode, InputState, Ui, TERMINAL_HEIGHT, TERMINAL_WIDTH},
    },
    shared_utils::{LockClean, NameValidation},
};
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

pub fn send_msg_to_local_history(
    messages: &mut Vec<String>,
    client_name: &mut String,
    input_buffer: &mut String,
) {
    let detailed_msg = format!("{}: {}", client_name, input_buffer);
    messages.push(detailed_msg);
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

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<(), Error> {
        let _ = self.client.connect();
        let (server_state_tx, server_state_rx) = mpsc::channel::<ServerState>();
        let (name_validation_tx, name_validation_rx) = mpsc::channel::<NameValidation>();
        let (new_message_tx, new_message_rx) = mpsc::channel::<bool>();
        let _ = self.client.handle_msgs(
            Arc::clone(&self.messages),
            server_state_tx,
            name_validation_tx,
            new_message_tx,
        );

        loop {
            match &self.client.networking.server_state {
                ServerState::Connected(_) => {
                    // if you used other terminla.draw method it will make like another buffer to draw on
                    terminal.draw(|frame| {
                        if frame.area().width < TERMINAL_WIDTH
                            || frame.area().height < TERMINAL_HEIGHT
                        {
                            let warning = self.ui.window_warning_msgs(&frame);
                            let warning_area = self.ui.get_window_warning_area(&frame);

                            frame.render_widget(warning, warning_area);
                        } else {
                            self.ui.render(frame, &mut self.messages.lock_mutex());
                        }
                    })?;

                    // check if server disconnected
                    if let Ok(_) = server_state_rx.try_recv() {
                        self.client.networking.server_state = ServerState::Disconnected;
                        continue;
                    }

                    // check for a new message
                    if let Ok(true) = new_message_rx.try_recv() {
                        self.ui.vertical_scrolling.last();
                        continue;
                    }

                    // 200millis to prevent 100% CPU core usage
                    if event::poll(Duration::from_millis(200))? {
                        if let Some(key) = event::read()?.as_key_press_event() {
                            match self.ui.input.mode {
                                InputMode::Normal => match key.code {
                                    KeyCode::Char('i') => self.ui.input.mode = InputMode::Editing,
                                    KeyCode::Char('q') => {
                                        self.client.disconnected();
                                        return Ok(());
                                    }
                                    KeyCode::Char('k') => self.ui.vertical_scrolling.prev(),
                                    KeyCode::Char('j') => self.ui.vertical_scrolling.next(),
                                    _ => {}
                                },

                                InputMode::Editing => match key.code {
                                    KeyCode::Enter => match self.ui.input_state {
                                        InputState::EnterName => {
                                            self.client.name = self.ui.input.buffer.clone();

                                            let serialized_msg =
                                                ClientMessages::CheckName(self.client.name.clone())
                                                    .serialize();
                                            self.client.networking.send_to_server(&serialized_msg);

                                            let prgh: Option<Paragraph> = match name_validation_rx
                                                .recv()
                                                .expect("[Error]:the reader thread get killed")
                                            {
                                                NameValidation::Empty => {
                                                    Ui::name_err_msg(&NameValidation::Empty)
                                                }
                                                NameValidation::Reserved => {
                                                    self.ui.input.buffer.clear();
                                                    self.ui.input.reset_cursor();
                                                    Ui::name_err_msg(&NameValidation::Reserved)
                                                }
                                                NameValidation::IllegalChar(c) => {
                                                    self.ui.input.buffer.clear();
                                                    self.ui.input.reset_cursor();
                                                    Ui::name_err_msg(&NameValidation::IllegalChar(
                                                        c,
                                                    ))
                                                }

                                                NameValidation::Used => {
                                                    self.ui.input.buffer.clear();
                                                    self.ui.input.reset_cursor();
                                                    Ui::name_err_msg(&NameValidation::Used)
                                                }
                                                NameValidation::Valid(received_name) => {
                                                    self.client.name = received_name.clone();
                                                    self.ui.input_state = InputState::Chatting;

                                                    self.ui.input.buffer.clear();
                                                    self.ui.input.reset_cursor();
                                                    Ui::name_err_msg(&NameValidation::Valid(
                                                        received_name,
                                                    ))
                                                }
                                            };

                                            if let Some(error_msg) = prgh {
                                                terminal.draw(|frame| {
                                                    frame.render_widget(error_msg, frame.area());
                                                })?;
                                                thread::sleep(Duration::from_millis(1700));
                                            }
                                            continue;
                                        }
                                        InputState::Chatting => {
                                            if self.ui.input.buffer == "/quit" {
                                                self.client.disconnected();
                                                return Ok(());
                                            }
                                            if self.ui.input.buffer.is_empty()
                                                || self.ui.input.buffer.trim() == String::new()
                                            {
                                                self.ui.input.buffer.clear();
                                                self.ui.input.reset_cursor();
                                                continue;
                                            }

                                            send_msg_to_local_history(
                                                &mut self.messages.lock_mutex(),
                                                &mut self.client.name,
                                                &mut self.ui.input.buffer,
                                            );

                                            // last method gonna put me on last message that is based on
                                            //the prev max pos, so i need to update it
                                            self.ui.updating_max_scroll_pos();
                                            self.ui.vertical_scrolling.last();

                                            let serialized_msg = ClientMessages::Chat {
                                                sender: self.client.name.clone(),
                                                content: self.ui.input.buffer.clone(),
                                            }
                                            .serialize();
                                            self.client.networking.send_to_server(&serialized_msg);

                                            self.ui.input.buffer.clear();
                                            self.ui.input.reset_cursor();
                                        }
                                    },

                                    KeyCode::Char(to_insert) => {
                                        self.ui.input.enter_char(to_insert);
                                    }
                                    KeyCode::Esc => self.ui.input.mode = InputMode::Normal,
                                    KeyCode::Backspace => self.ui.input.delete_char(),
                                    KeyCode::Right => {
                                        self.ui.input.move_cursor_right();
                                    }
                                    KeyCode::Left => {
                                        self.ui.input.move_cursor_left();
                                    }
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
