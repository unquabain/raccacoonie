use crate::prelude_internal::*;
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

#[derive(Debug,Default,Clone)]
pub struct InputControl {
    label: Option<String>,
    focus: FocusState,
    input: Input,
}

impl InputControl {
    pub fn from_label(label: &str) -> Self {
        Self {
            label: Some(label.to_owned()),
            focus: Default::default(),
            input: Default::default(),
        }
    }

    pub fn from_value(value: &str) -> Self {
        Self {
            label: None,
            focus: Default::default(),
            input: Input::new(value.to_string()),
        }
    }
    pub fn from_label_and_value(label: &str, value: &str) -> Self {
        Self {
            label: Some(label.to_owned()),
            focus: Default::default(),
            input: Input::new(value.to_string()),
        }
    }
    pub fn set_focus(&mut self, focus: FocusState) {
        self.focus = focus;
    }
    pub fn value(&self) -> String {
        self.input.value().to_owned()
    }
    pub fn set_value(&mut self, value: &str) {
        self.input = Input::new(value.to_string());
    }
}

impl Model for InputControl {
    fn update(&mut self, msg: Message) -> Message {
        match msg {
            Message::KeyPress(key_event) => {
                let event = ratatui::crossterm::event::Event::Key(key_event);
                self.input.handle_event(&event);
                Message::Redraw
            }
            _ => Message::Noop,
        }
    }
    fn view(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
                // keep 2 for borders and 1 for cursor
        let width = area.width.max(3) - 3;
        let scroll = self.input.visual_scroll(width as usize) as u16;

        let style = match self.focus {
            FocusState::Blur => STYLES.blur.clone(),
            FocusState::Focus => STYLES.focus.clone(),
        };
        let par = Paragraph::new(self.input.value())
            .scroll((0, scroll))
            .block(match &self.label {
                Some(label) => style.block.clone().title(label.clone()),
                None => style.block,
            });
        frame.render_widget(par, area);
        if self.focus == FocusState::Focus {
            let cursor_x = area.x + self.input.visual_cursor().max(scroll as usize) as u16 - scroll + 1;
            let cursor_y = area.y + 1;
            frame.set_cursor_position((cursor_x, cursor_y));
        }

        Ok(())
    }
}


