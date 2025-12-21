use crate::prelude_internal::*;
use ratatui::widgets::Widget;
use uuid::Uuid;

pub struct Spinner {
    chars: Vec<char>,
    index: usize,
    id: Uuid,
}

impl Model for Spinner {
    fn init(&mut self) -> Message {
        let (id, tik) = Message::tick(tokio::time::Duration::from_millis(100));
        self.id = id;
        tik
    }

    fn update(&mut self, msg: Message) -> Message {
        match msg {
            Message::Tok(id) => {
                if id == self.id {
                    self.index = (self.index + 1) % self.chars.len();
                    let (new_id, tik) = Message::tick(tokio::time::Duration::from_millis(100));
                    self.id = new_id;
                    Message::Redraw.and(tik)
                } else {
                    Message::Noop
                }
            }
            _ => Message::Noop,
        }
    }

    fn view(&mut self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) -> Result<()> {
        let c = self.chars[self.index];
        frame.render_widget(c.to_string(), area);
        Ok(())
    }
}

impl Widget for Spinner {
    fn render(
        self,
        area: ratatui::layout::Rect,
        buf: &mut ratatui::buffer::Buffer,
    ) {
        let c = self.chars[self.index];
        buf.set_string(area.x, area.y, c.to_string(), ratatui::style::Style::default());
    }
}

impl Spinner {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn new_from_iter(chars: impl IntoIterator<Item = char>) -> Self {
        Spinner {
            chars: chars.into_iter().collect(),
            index: 0,
            id: Uuid::new_v4(),
        }
    }
    pub fn circles() -> Self {
        Spinner::new_from_iter(vec!['◐', '◓', '◑', '◒'])
    }
    pub fn dots() -> Self {
        Spinner::new_from_iter(vec!['⠁', '⠂', '⠄', '⡀', '⢀', '⠠', '⠐', '⠈'])
    }
    pub fn arrows() -> Self {
        Spinner::new_from_iter(vec!['←', '↖', '↑', '↗', '→', '↘', '↓', '↙'])
    }
}

impl Default for Spinner {
    fn default() -> Self {
        Spinner {
            chars: vec!['|', '/', '-', '\\'],
            index: 0,
            id: Uuid::new_v4(),
        }
    }
}

impl std::fmt::Display for Spinner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.chars[self.index])
    }
}
