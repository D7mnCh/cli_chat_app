use ratatui::layout::{Constraint, Layout, Position};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

#[derive(Default)]
pub struct Ui {
    pub input: String,
    pub input_mode: InputMode,
    /// Position of cursor in the editor area.
    character_index: usize,
    messages_state: ListState,
}
#[derive(Default)]
pub enum InputMode {
    #[default]
    Normal,
    Editing,
}

impl Ui {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            input_mode: InputMode::Normal,
            character_index: 0,
            messages_state: ListState::default(),
        }
    }

    // NOTE saturating methods to prevent overflow

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
            self.input.insert(index, new_char);
            self.move_cursor_right();
        }
    }

    // returns the index of a current cursor pos
    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            // need if n is greater or equal string
            .unwrap_or(self.input.len())
    }

    // NOTE didn't read this
    pub fn delete_char(&mut self) {
        let cursor_not_left_most = self.character_index != 0;
        if cursor_not_left_most {
            // NOTE i waanna try to use remove's string method instead of iterators

            let current_index = self.character_index;
            let chars_before_del_char: usize = current_index - 1;
            let chars_after_del_char: usize = current_index;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.input.chars().take(chars_before_del_char);
            // getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(chars_after_del_char);

            // Put all characters together except the selected one.
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    // NOTE this will actually making a bound from 0 to max input character, to prevent cursor overpassed
    //the input string
    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    const fn reset_cursor(&mut self) {
        self.character_index = 0;
    }

    pub fn scroll_down(&mut self, messages_len: usize) {
        let i = match self.messages_state.selected() {
            Some(i) => {
                if i >= messages_len.saturating_sub(1) {
                    messages_len.saturating_sub(1)
                } else {
                    i + 1
                }
            }
            None => 0,
        };

        self.messages_state.select(Some(i));
    }

    pub fn scroll_up(&mut self) {
        let i = match self.messages_state.selected() {
            Some(i) => i.saturating_sub(1),
            None => 0,
        };

        self.messages_state.select(Some(i));
    }
    pub fn select_last_message(&mut self, messages: &mut Vec<String>) {
        self.messages_state
            .select(Some(messages.len().saturating_sub(1)));
    }

    pub fn submit_message(&mut self, client_name: &String, messages: &mut Vec<String>) {
        let detailed_msg = format!("{}: {}", client_name, self.input);
        messages.push(detailed_msg);
        self.reset_cursor();
    }

    // NOTE i should leanrn about ratatui
    pub fn render(&self, frame: &mut Frame, messages: &[String]) {
        let layout = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(1),
        ]);
        let [help_area, input_area, messages_area] = frame.area().layout(&layout);

        // helping area things
        let (msg, style) = match self.input_mode {
            InputMode::Normal => (
                vec!["Press q to exit, e to start editing.".into()],
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
        let input = Paragraph::new(self.input.as_str())
            .style(match self.input_mode {
                InputMode::Normal => Style::default(),
                InputMode::Editing => Style::default().fg(Color::Yellow),
            })
            .block(Block::bordered().title("Input"));
        frame.render_widget(input, input_area);
        match self.input_mode {
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            InputMode::Normal => {}

            // Make the cursor visible and ask ratatui to put it at the specified coordinates after
            // rendering
            #[expect(clippy::cast_possible_truncation)]
            InputMode::Editing => frame.set_cursor_position(Position::new(
                // Draw the cursor at the current position in the input field.
                // This position can be controlled via the left and right arrow key
                input_area.x + self.character_index as u16 + 1,
                // Move one line down, from the border to the input line
                input_area.y + 1,
            )),
        }

        // messages area
        let messages: Vec<ListItem> = messages
            .iter()
            .map(|msgs| {
                let content = Line::from(Span::raw(format!("{}", msgs)));
                ListItem::new(content)
            })
            .collect();
        let messages = List::new(messages).block(Block::bordered().title("Messages"));
        frame.render_stateful_widget(messages, messages_area, &mut self.messages_state.clone());
    }
}
#[cfg(test)]
mod test {}
