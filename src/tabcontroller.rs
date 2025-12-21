use crate::prelude_internal::*;

#[derive(Default)]
pub struct TabController {
    size: usize,
    current: usize,
}

impl TabController {
    pub fn new(size: usize) -> Self {
        Self {
            size,
            current: 0,
        }
    }

    pub fn add(&mut self, new: usize) -> usize {
        self.size += new;
        self.size
    }

    #[inline]
    pub fn get_current_index(&self) -> usize {
        self.current
    }

    pub fn next(&mut self) -> usize {
        self.current += 1;
        if self.current >= self.size {
            self.current = 0;
        }
        self.current
    }

    pub fn previous(&mut self) -> usize {
        if self.current == 0 {
            self.current = self.size;
        }
        self.current -= 1;
        self.current
    }

    pub fn is_focused(&self, idx: usize) -> bool {
        idx == self.current
    }

    pub fn update(&mut self, msg: &Message) -> Message {
        use ratatui::crossterm::event::{
            KeyEvent,
            KeyCode,
        };
        match *msg {
            Message::KeyPress(KeyEvent{code: KeyCode::Tab, ..}) => {
                self.next();
                log::info!("in tab_contoller::update with {msg:?}; current index: {}", self.current);
                Message::Redraw
            }
            Message::KeyPress(KeyEvent{code: KeyCode::BackTab, ..}) => {
                self.previous();
                log::info!("in tab_contoller::update with {msg:?}; current index: {}", self.current);
                Message::Redraw
            }
            _ => Message::Noop
        }
    }

    pub fn iter(&mut self) -> impl std::iter::Iterator<Item=(usize, FocusState)> {
        let current = self.current;
        (0..self.size).into_iter().map(move |i| (i, if i==current { FocusState::Focus } else { FocusState::Blur }))
    }
}

