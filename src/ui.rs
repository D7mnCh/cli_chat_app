use crate::app::NameValidation;
use ratatui::layout::{Constraint, Layout, Margin, Position, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState};
use ratatui::Frame;

pub enum InputMode {
    Normal,
    Editing,
}

pub enum InputState {
    EnterName,
    Chatting,
}

pub enum ServerError {
    ServerNotRunning,
    ServerDisconneted,
}

pub enum Logging {
    MessagesHistory,
}

pub struct Ui {
    pub input: Input,
    pub input_state: InputState,
    pub vertical_scrolling: ScrollbarState,
}
pub struct Input {
    pub buffer: String,
    pub mode: InputMode,
    character_index: usize,
}

impl Input {
    pub fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    pub fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    // NOTE the pos of the new character is based on the string
    pub fn enter_char(&mut self, new_char: char) {
        if self.character_index < 30 {
            let index = self.byte_index();
            self.buffer.insert(index, new_char);
            self.move_cursor_right();
        }
    }

    // returns the index of a current cursor pos
    fn byte_index(&self) -> usize {
        self.buffer
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            // need if n is greater or equal string
            .unwrap_or(self.buffer.len())
    }

    pub fn delete_char(&mut self) {
        let cursor_not_left_most = self.character_index != 0;
        if cursor_not_left_most {
            // NOTE i waanna try to use remove's string method instead of iterators

            let current_index = self.character_index;
            let chars_before_del_char: usize = current_index - 1;
            let chars_after_del_char: usize = current_index;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.buffer.chars().take(chars_before_del_char);
            // getting all characters after selected character.
            let after_char_to_delete = self.buffer.chars().skip(chars_after_del_char);

            // Put all characters together except the selected one.
            self.buffer = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    // NOTE this will actually making a bound from 0 to max input character, to prevent cursor
    //overpassed the input string
    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.buffer.chars().count())
    }

    pub const fn reset_cursor(&mut self) {
        self.character_index = 0;
    }
}

// NOTE make nested struct
impl Ui {
    pub fn new() -> Self {
        Self {
            input: Input {
                buffer: String::new(),
                mode: InputMode::Normal,
                character_index: 0,
            },
            input_state: InputState::EnterName,
            vertical_scrolling: ScrollbarState::new(0),
        }
    }

    // saturating methods to prevent overflow

    fn render_vertical_scrollbar(&mut self, frame: &mut Frame, area: Rect, messages: &Vec<String>) {
        // store last pos cuz when creating a new scrollbarState it will reset the pos
        let last_pos = self.vertical_scrolling.get_position();
        self.vertical_scrolling = ScrollbarState::new(messages.len());
        self.vertical_scrolling = self.vertical_scrolling.position(last_pos);

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
        frame.render_stateful_widget(
            scrollbar,
            area.inner(Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut self.vertical_scrolling,
        );
    }

    pub fn name_err_msg<'a>(state: NameValidation) -> Option<Paragraph<'a>> {
        match state {
            NameValidation::Reserved => Some(
                Paragraph::new("name used by server")
                    .centered()
                    .block(Block::bordered().title_top(Line::from("Reserved name").centered())),
            ),
            NameValidation::Empty => Some(
                Paragraph::new("No name entered")
                    .block(Block::bordered().title_top(Line::from("Invalid name").centered()))
                    .centered(),
            ),
            NameValidation::Used => Some(
                Paragraph::new("other user is using this name")
                    .block(Block::bordered().title_top(Line::from("Used name").centered()))
                    .centered(),
            ),
            NameValidation::Valid(_) => None,
        }
    }

    // NOTE i should learn about ratatui
    pub fn render(&mut self, frame: &mut Frame, messages: &mut Vec<String>) {
        // NOTE maybe do match here ?
        let (help_area, input_area, messages_area) = match self.input_state {
            InputState::EnterName => {
                let layout = Layout::vertical([Constraint::Length(1), Constraint::Length(3)]);
                let [help, input] = frame.area().layout(&layout);
                (help, input, None)
            }
            InputState::Chatting => {
                let layout = Layout::vertical([
                    Constraint::Length(1),
                    Constraint::Length(3),
                    Constraint::Min(1),
                ]);
                let [help, input, messages] = frame.area().layout(&layout);
                (help, input, Some(messages))
            }
        };

        // helping area things
        let (msg, style) = match self.input.mode {
            InputMode::Normal => (
                vec!["Press q to exit, i to start editing.".into()],
                Style::default(),
            ),
            InputMode::Editing => (
                vec!["Press Esc to stop editing, Enter to record the message".into()],
                Style::default(),
            ),
        };
        let text = Text::from(Line::from(msg)).patch_style(style);
        let help_message = Paragraph::new(text);
        frame.render_widget(help_message, help_area);

        // input area
        let input = Paragraph::new(self.input.buffer.as_str())
            .style(match self.input.mode {
                InputMode::Normal => Style::default(),
                InputMode::Editing => Style::default().fg(Color::Yellow),
            })
            .block(Block::bordered().title(match self.input_state {
                InputState::Chatting => "Input",
                InputState::EnterName => "Enter Your Name",
            }));
        frame.render_widget(input, input_area);
        match self.input.mode {
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            InputMode::Normal => {}

            // Make the cursor visible and ask ratatui to put it at the specified coordinates after
            // rendering
            InputMode::Editing => frame.set_cursor_position(Position::new(
                // Draw the cursor at the current position in the input field.
                // This position can be controlled via the left and right arrow key
                input_area.x + self.input.character_index as u16 + 1,
                // Move one line down, from the border to the input line
                input_area.y + 1,
            )),
        }
        // messages area
        if let Some(messages_area) = messages_area {
            let msgs = messages
                .iter()
                .map(|msgs| {
                    let content = Line::from(Span::raw(format!("{}", msgs)));
                    content
                })
                .collect::<Vec<Line>>();
            // NOTE ratatui should make vertical alignemnt as they do with horizontal alignemnt
            let messages_block = Paragraph::new(msgs)
                .scroll((
                    (self.vertical_scrolling.get_position().saturating_sub(10)) as u16,
                    0 as u16,
                ))
                .cyan()
                .block(Block::bordered().title("Messages"));

            frame.render_widget(messages_block, messages_area);
            // NOTE fix the scroll bar cutting when scrolling (want to be one piece)
            self.render_vertical_scrollbar(frame, messages_area, messages);
        }
    }
}

#[cfg(test)]
mod test {}
