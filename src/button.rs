use crate::prelude_internal::*;
use ratatui::crossterm::event::KeyCode;

#[derive(Debug,Clone)]
pub struct Button {
    label: String,
    focus: FocusState,
    on_press: Message,
}

impl Button {
    pub fn new(label: &str, msg: Message) -> Self {
        Self {
            label: label.to_owned(),
            focus: Default::default(),
            on_press: msg,
        }
    }
}

impl Model for Button {
    fn set_focus(&mut self, focus: FocusState) {
        self.focus = focus;
    }
    fn update(&mut self, msg: Message) -> Message {
        match msg {
            Message::KeyPress(key_event) => {
                match key_event.code {
                    KeyCode::Enter | KeyCode::Char(' ') => self.on_press.clone(),
                    _ => Message::Noop,
                }
            }
            _ => Message::Noop,
        }
    }
    fn view(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let style = match self.focus {
            FocusState::Blur => &STYLES.blur,
            FocusState::Focus => &STYLES.focus,
        };
        let paragraph = Paragraph::new(self.label.as_str())
            .block(style.block.clone())
            .centered()
        ;
        frame.render_widget(paragraph, area);
        Ok(())
    }
    fn help(&self) -> Option<String> {
        Some(format!("Press ENTER or SPACE to activate the {} button", self.label))
    }
}

impl From<(&str, Message)> for Button {
    fn from((label, msg): (&str, Message)) -> Self {
        Self::new(label, msg)
    }
}

#[derive(Debug,Clone)]
pub struct ButtonBar {
    buttons: Vec<Button>,
    focus_index: usize,
}

impl ButtonBar {
    pub fn new<I: Into<Button>, V: IntoIterator<Item=I>>(buttons: V) -> Self {
        Self {
            buttons: buttons.into_iter().map(|b| b.into()).collect(),
            focus_index: 0,
        }
    }
    pub fn yes_no() -> Self {
        Self::new([("Yes", Message::Yes), ("No", Message::No)])
    }
    pub fn ok_cancel() -> Self {
        Self::new([("OK", Message::Yes), ("Cancel", Message::No)])
    }
    pub fn ok_quit() -> Self {
        Self::new([("OK", Message::Yes), ("Quit", Message::Quit)])
    }
    pub fn set_focus(&mut self, index: usize) {
        if index < self.buttons.len() {
            self.focus_index = index;
        }
    }
    pub fn blur(&mut self) {
        self.focus_index = usize::MAX;
    }
}

impl Model for ButtonBar {
    fn update(&mut self, msg: Message) -> Message {
        if let Some(button) = self.buttons.get_mut(self.focus_index) {
            button.update(msg)
        } else {
            Default::default()
        }
    }
    fn view(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let button_width = area.width / self.buttons.len() as u16;
        for (i, button) in self.buttons.iter_mut().enumerate() {
            let x = area.x + i as u16 * button_width;
            let button_area = Rect {
                x,
                y: area.y,
                width: button_width,
                height: area.height,
            };
            if i == self.focus_index {
                button.set_focus(FocusState::Focus);
            } else {
                button.set_focus(FocusState::Blur);
            }
            button.view(frame, button_area)?;
        }
        Ok(())
    }

    fn help(&self) -> Option<String> {
        match self.buttons.get(self.focus_index) {
            Some(button) => button.help(),
            None => Some("No button active".to_owned()),
        }
    }
}
