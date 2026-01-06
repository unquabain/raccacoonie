use crate::prelude_internal::*;
use ratatui::widgets::ListState;
use ratatui::crossterm::event::KeyCode;
use crate::input_control::InputControl;
use std::fmt::Display;
use std::iter::IntoIterator;

#[derive(Debug, PartialEq, Eq, Default)]
enum ListViewMode {
    #[default]
    Normal,
    Searching,
    Filtered,
}

#[derive(Debug, Default)]
pub struct ListView<Item: Display + Clone> {
    title: String,
    items: Vec<Item>,
    state: ListState,
    search: InputControl,
    mode: ListViewMode,
    focus: FocusState,
    pub chosen: Option<Item>,
}

impl<Item: Display + Clone> ListView<Item> {
    pub fn new(title: &str, items: impl IntoIterator<Item=Item>) -> Self {
        let items = items.into_iter().collect();
        Self {
            title: title.to_string(),
            items,
            state: ListState::default(),
            search: InputControl::from_value(""),
            mode: Default::default(),
            focus: FocusState::Blur,
            chosen: None,
        }
    }
    pub fn with_items(&self, items: impl IntoIterator<Item=Item>) -> Self {
        Self::new(&self.title, items)
    }
    fn filtered_items(&self) -> impl Iterator<Item=&Item> {
        let query = self.search.value().to_lowercase();
        self.items.iter().filter(move |item| {
            item.to_string().to_lowercase().contains(&query)
        })
    }
    pub fn filtered_get(&self, index: usize) -> Option<&Item> {
        self.filtered_items().nth(index)
    }
    pub fn selected(&self) -> Option<&Item> {
        let idx = match self.state.selected() {
            None => return None,
            Some(idx) => idx,
        };
        self.filtered_get(idx)
    }
    fn filtered_len(&self) -> usize {
        self.filtered_items().count()
    }
    fn filtered_is_empty(&self) -> bool {
        self.filtered_len() == 0
    }
}

impl<Item: Display + Clone> Model for ListView<Item> {
    fn init(&mut self) -> Message {
        if self.items.is_empty() {
            return Message::Noop
        }
        self.state.select(Some(0));
        Message::Redraw
    }
    fn set_focus(&mut self, focus: FocusState) {
        self.focus = focus;
    }

    fn update(&mut self, msg: Message) -> Message {
        if let Message::KeyPress(key) = msg {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('w') => {
                    match self.mode {
                        ListViewMode::Searching => {
                            self.search.update(msg)
                        }
                        _ => {
                            let fl = self.filtered_len();
                            let i = match self.state.selected() {
                                Some(i) => if i == 0 {
                                    Some(fl - 1)
                                } else {
                                    Some(i - 1)
                                }
                                None => if self.filtered_is_empty() {
                                    None
                                } else {
                                    Some(fl - 1)
                                }
                            };
                            self.state.select(i);
                            Message::Redraw
                        }
                    }
                }
                KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('s') => {
                    match self.mode {
                        ListViewMode::Searching =>
                            if key.code == KeyCode::Down {
                                self.mode = if self.search.value().is_empty() {
                                    ListViewMode::Normal
                                } else {
                                    ListViewMode::Filtered
                                };
                                self.state.select(if self.filtered_len() == 0 { None } else { Some(0) });
                                Message::Redraw
                            } else {
                                self.search.update(msg)
                            }
                        _ => {
                            let fl = self.filtered_len();
                            let i = match self.state.selected() {
                                Some(i) => if i >= fl - 1 {
                                    Some(0)
                                } else {
                                    Some(i + 1)
                                }
                                None => if self.filtered_is_empty() {
                                    None
                                } else {
                                    Some(0)
                                }
                            };
                            self.state.select(i);
                            Message::Redraw
                        }
                    }
                }
                KeyCode::Enter => {
                    match self.mode {
                        ListViewMode::Searching => {
                            self.mode = if self.search.value().is_empty() {
                                ListViewMode::Normal
                            } else {
                                ListViewMode::Filtered
                            };
                            self.state.select(if self.filtered_len() == 0 { None } else { Some(0) });
                            Message::Redraw
                        }
                        _ => if self.filtered_is_empty() {
                                Message::errorf("No items match the search query.")
                            } else if let Some(i) = self.state.selected() {
                                self.chosen = self.filtered_get(i).cloned();
                                Message::choose(i)
                            } else {
                                Message::Noop
                            }
                    }
                }
                KeyCode::Char('/') => {
                    self.mode = ListViewMode::Searching;
                    self.search.set_value("");
                    Message::Redraw
                }
                _ => {
                    if self.mode == ListViewMode::Searching {
                        self.search.update(msg)
                    } else {
                        Message::Noop
                    }
                }
            }
        } else { Message::Noop }
    }

    fn view(&mut self, f: &mut Frame, area: Rect) -> Result<()> {
        if self.mode != ListViewMode::Normal {
            self.search.set_focus(
                if self.mode == ListViewMode::Searching {
                    FocusState::Focus
                } else {
                    FocusState::Blur
                }
            );
            self.search.view(f, Rect {
                x: area.x,
                y: area.y,
                width: area.width,
                height: 3,
            })?;
        }
        let list_area = match self.mode {
            ListViewMode::Normal => area,
            _ => {
                Rect {
                    x: area.x,
                    y: area.y + 3,
                    width: area.width,
                    height: area.height - 3,
                }
            }
        };
        let items = self.filtered_items().map(|item| ratatui::widgets::ListItem::new(item.to_string())).collect::<Vec<_>>();
        STYLES.render_list(
            &self.title,
            self.focus,
            items,
            &mut self.state,
            list_area,
            f,
        );

        Ok(())
    }

    fn help(&self) -> Option<String> {
        Some("Use Up/Down arrows or 'j'/'k' to navigate, Enter to select, / to toggle filtering".to_string())
    }
}
