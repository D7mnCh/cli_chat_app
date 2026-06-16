use super::{
    channels::{create_channels, ChannelReceivers},
    client_messages::ClientMessages,
    network::{Client, ServerState},
    ui::{InputMode, InputState, RenderingEvents, Ui},
};

use crate::{
    client::ui::{TERMINAL_HEIGHT, TERMINAL_WIDTH},
    shared_utils::{LockClean, NameValidation},
};

use std::{
    io::Error,
    sync::{Arc, Mutex},
    thread::sleep,
    time::Duration,
};

use crossterm::event::{self, KeyCode, KeyEvent};
use ratatui::DefaultTerminal;

pub struct App {
    ui: Ui,
    client: Client,
    messages: Arc<Mutex<Vec<String>>>,
    // Option cuz i can't build Receiver<T> with Sender<T> on app's new method
    channel_receivers: Option<ChannelReceivers>,
}

impl App {
    pub fn new() -> Self {
        let ui = Ui::new();
        let client = Client::new();

        Self {
            ui,
            client,
            channel_receivers: None,
            messages: Default::default(),
        }
    }

    pub fn init_networking(&mut self) {
        let _ = self.client.connect();

        let (sender, receiver) = create_channels();
        self.channel_receivers = Some(receiver);

        let _ = self.client.handle_msgs(Arc::clone(&self.messages), sender);
    }

    fn send_msg_to_local_history(&self) {
        let detailed_msg = format!("{}: {}", self.client.name, self.ui.input.buffer);
        self.messages.lock_mutex().push(detailed_msg);
    }

    fn handle_enter_name(&mut self) {
        self.client.name = self.ui.input.buffer.clone();

        let serialized_msg = ClientMessages::CheckName(self.client.name.clone()).serialize();
        self.client.networking.send_to_server(&serialized_msg);

        match self
            .channel_receivers
            .as_ref()
            .unwrap()
            .name_validation_rx
            .recv()
            .expect("[Error]:the reader thread get killed")
        {
            NameValidation::Empty => {
                self.ui.rendering_events =
                    Some(RenderingEvents::NameValidationError(NameValidation::Empty))
            }
            NameValidation::Reserved => {
                self.ui.input.clear();
                self.ui.rendering_events = Some(RenderingEvents::NameValidationError(
                    NameValidation::Reserved,
                ))
            }
            NameValidation::IllegalChar(c) => {
                self.ui.input.clear();
                self.ui.rendering_events = Some(RenderingEvents::NameValidationError(
                    NameValidation::IllegalChar(c),
                ))
            }

            NameValidation::Used => {
                self.ui.input.clear();
                self.ui.rendering_events =
                    Some(RenderingEvents::NameValidationError(NameValidation::Used))
            }
            NameValidation::Valid(received_name) => {
                self.client.name = received_name.clone();
                self.ui.input_state = InputState::Chatting;

                self.ui.input.clear();
                self.ui.rendering_events = Some(RenderingEvents::NameValidationError(
                    NameValidation::Valid(received_name),
                ))
            }
        };
    }

    fn handle_chat(&mut self) {
        if self.ui.input.buffer.is_empty() || self.ui.input.buffer.trim().is_empty() {
            self.ui.input.clear();
            return;
        }

        self.send_msg_to_local_history();

        // last method gonna put me on the last message that is based on
        //the prev max scrolling pos, so i need to update it
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

    fn handle_pressing_enter(&mut self) {
        match self.ui.input_state {
            InputState::EnterName => self.handle_enter_name(),
            InputState::Chatting => self.handle_chat(),
        }
    }

    fn render_app(&mut self, terminal: &mut DefaultTerminal) -> Result<(), Error> {
        terminal.draw(|frame| {
            if frame.area().width < TERMINAL_WIDTH || frame.area().height < TERMINAL_HEIGHT {
                self.ui.rendering_events = Some(RenderingEvents::MustResizingWarrning);
            }

            match &self.ui.rendering_events {
                None => self.ui.render_chat(frame, &mut self.messages.lock_mutex()),
                Some(RenderingEvents::ServerDisconnected) => {
                    self.ui.render_server_is_disconnected_window(frame);
                    self.client.disconnected();
                    return;
                }
                Some(RenderingEvents::MustResizingWarrning) => {
                    self.ui.render_must_resize_window(frame)
                }
                Some(RenderingEvents::NameValidationError(name_validation)) => {
                    self.ui.input.clear();
                    if let NameValidation::Valid(received_name) = name_validation {
                        self.client.name = received_name.clone();
                        self.ui.input_state = InputState::Chatting;
                    } else {
                        // NOTE there's a bug, popout window error of name not valid not showing
                        self.ui
                            .render_name_not_valid_error_window(name_validation, frame);
                        //sleep(Duration::from_millis(1200));
                    }
                }
            }

            if self.ui.rendering_events.is_some() {
                self.ui.rendering_events = None;
            };
        })?;
        Ok(())
    }

    fn _handle_input(&mut self) {}

    fn check_if_server_disconnected(&mut self) {
        if let Some(ref channel_receivers) = self.channel_receivers {
            if channel_receivers.server_state_rx.try_recv().is_ok() {
                self.client.networking.server_state = ServerState::Disconnected;
                return;
            }
        }
    }

    fn check_if_new_msg_arrived(&mut self) {
        // TODO i think you can redisgn this
        if let Some(ref channel_receivers) = self.channel_receivers {
            if let Ok(true) = channel_receivers.new_message_rx.try_recv() {
                self.ui.vertical_scrolling.last();
                return;
            }
        }
    }

    fn handle_input_normal_mode(&mut self, key: &KeyEvent) -> Result<(), Error> {
        match key.code {
            KeyCode::Char('i') => self.ui.input.mode = InputMode::Editing,
            KeyCode::Char('q') => {
                self.client.disconnected();
                return Ok(());
            }
            KeyCode::Char('k') => self.ui.vertical_scrolling.prev(),
            KeyCode::Char('j') => self.ui.vertical_scrolling.next(),
            _ => {}
        }
        Ok(())
    }

    fn handle_input_edit_mode(&mut self, key: &KeyEvent) -> Result<(), Error> {
        match key.code {
            KeyCode::Enter => self.handle_pressing_enter(),
            KeyCode::Char(to_insert) => self.ui.input.enter_char(to_insert, &self.ui.input_state),
            KeyCode::Esc => self.ui.input.mode = InputMode::Normal,
            KeyCode::Backspace => self.ui.input.delete_char(),
            KeyCode::Right => {
                self.ui.input.move_cursor_right();
            }
            KeyCode::Left => {
                self.ui.input.move_cursor_left();
            }
            _ => {}
        }
        Ok(())
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<(), Error> {
        loop {
            self.render_app(terminal)?;
            match &self.client.networking.server_state {
                ServerState::Connected(_) => {
                    self.check_if_server_disconnected();
                    self.check_if_new_msg_arrived();

                    // 200millis to prevent 100% CPU core usage
                    if event::poll(Duration::from_millis(200))? {
                        if let Some(key) = event::read()?.as_key_press_event() {
                            match self.ui.input.mode {
                                InputMode::Normal => self.handle_input_normal_mode(&key)?,
                                InputMode::Editing => self.handle_input_edit_mode(&key)?,
                            }
                        }
                    }
                }

                ServerState::Disconnected => {
                    if self.ui.rendering_events == Some(RenderingEvents::ServerDisconnected) {
                        sleep(Duration::from_millis(1700));
                        return Ok(());
                    }
                    self.ui.rendering_events = Some(RenderingEvents::ServerDisconnected);
                }
            }
        }
    }
}
