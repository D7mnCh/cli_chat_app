use crate::{
    client::{
        channels::{create_channels, ChannelReceivers},
        client_messages::ClientMessages,
        network::{Client, ServerState},
        ui::{InputMode, InputState, Ui, TERMINAL_HEIGHT, TERMINAL_WIDTH},
    },
    shared_utils::{LockClean, NameValidation},
};
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
    client: Client,
    messages: Arc<Mutex<Vec<String>>>,
    // Option cuz i can't build Receiver<T> with Sender<T> on new method
    channel_receivers: Option<ChannelReceivers>,
}

impl App {
    pub fn new() -> Self {
        let ui = Ui::new();
        let client = Client::new();

        Self {
            ui,
            client,
            messages: Default::default(),
            channel_receivers: None,
        }
    }

    fn send_msg_to_local_history(&self) {
        let detailed_msg = format!("{}: {}", self.client.name, self.ui.input.buffer);
        self.messages.lock_mutex().push(detailed_msg);
    }

    pub fn init_networking(&mut self) {
        let _ = self.client.connect();

        let (sender, receiver) = create_channels();
        self.channel_receivers = Some(receiver);

        let _ = self.client.handle_msgs(Arc::clone(&self.messages), sender);
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<(), Error> {
        loop {
            match &self.client.networking.server_state {
                ServerState::Connected(_) => {
                    // if you used other terminla.draw method it will make like another buffer to draw on
                    terminal.draw(|frame| {
                        if frame.area().width < TERMINAL_WIDTH
                            || frame.area().height < TERMINAL_HEIGHT
                        {
                            let warning = self.ui.window_warning_msgs(&frame);
                            let warning_area = self.ui.get_window_center_area(&frame);

                            frame.render_widget(warning, warning_area);
                        } else {
                            self.ui.render(frame, &mut self.messages.lock_mutex());
                        }
                    })?;

                    // check if server disconnected
                    if let Some(ref channel_receivers) = self.channel_receivers {
                        if channel_receivers.server_state_rx.try_recv().is_ok() {
                            self.client.networking.server_state = ServerState::Disconnected;
                            continue;
                        }
                    }

                    // check for a new message
                    if let Some(ref channel_receivers) = self.channel_receivers {
                        if let Ok(true) = channel_receivers.new_message_rx.try_recv() {
                            self.ui.vertical_scrolling.last();
                            continue;
                        }
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

                                            let prgh: Option<Paragraph> = match self
                                                .channel_receivers
                                                .as_ref()
                                                .unwrap()
                                                .name_validation_rx
                                                .recv()
                                                .expect("[Error]:the reader thread get killed")
                                            {
                                                NameValidation::Empty => {
                                                    Ui::name_err_msg(&NameValidation::Empty)
                                                }
                                                NameValidation::Reserved => {
                                                    self.ui.input.clear();
                                                    Ui::name_err_msg(&NameValidation::Reserved)
                                                }
                                                NameValidation::IllegalChar(c) => {
                                                    self.ui.input.clear();
                                                    Ui::name_err_msg(&NameValidation::IllegalChar(
                                                        c,
                                                    ))
                                                }

                                                NameValidation::Used => {
                                                    self.ui.input.clear();
                                                    Ui::name_err_msg(&NameValidation::Used)
                                                }
                                                NameValidation::Valid(received_name) => {
                                                    self.client.name = received_name.clone();
                                                    self.ui.input_state = InputState::Chatting;

                                                    self.ui.input.clear();
                                                    Ui::name_err_msg(&NameValidation::Valid(
                                                        received_name,
                                                    ))
                                                }
                                            };

                                            if let Some(error_msg) = prgh {
                                                terminal.draw(|frame| {
                                                    let error_area =
                                                        self.ui.get_window_center_area(frame);
                                                    frame.render_widget(error_msg, error_area);
                                                })?;
                                                thread::sleep(Duration::from_millis(1700));
                                            }
                                            continue;
                                        }
                                        InputState::Chatting => {
                                            if self.ui.input.buffer.is_empty()
                                                || self.ui.input.buffer.trim().is_empty()
                                            {
                                                self.ui.input.clear();
                                                continue;
                                            }

                                            self.send_msg_to_local_history();

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

                                            self.ui.input.clear();
                                        }
                                    },

                                    KeyCode::Char(to_insert) => {
                                        self.ui.input.enter_char(to_insert, &self.ui.input_state);
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
