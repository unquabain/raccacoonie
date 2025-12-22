use lazy_static::lazy_static;
use ratatui::style::palette::tailwind::{SKY, SLATE, ORANGE};
use ratatui::widgets::{
    Block, Borders, List, ListItem, ListState,
    HighlightSpacing
};
use ratatui::Frame;
use ratatui::layout::Rect;

#[derive(Debug, Default, Clone)]
pub struct Style{
    pub block: Block<'static>,
    pub highlight: ratatui::style::Style,
    pub highlight_symbol: &'static str,
    pub highlight_spacing: HighlightSpacing,
}

#[derive(Debug, Clone)]
pub struct Styles {
    pub blur: Style,
    pub focus: Style,
    pub error: Style,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusState {
    #[default]
    Blur,
    Focus,
}

impl From<bool> for FocusState {
    fn from(b: bool) -> Self {
        if b {
            Self::Focus
        } else {
            Self::Blur
        }
    }
}

impl From<FocusState> for bool {
    fn from(fs: FocusState) -> Self {
        fs == FocusState::Focus
    }
}

impl Default for Styles {
    fn default() -> Self {
        Self {
            blur: Style {
                block: Block::default()
                    .borders(Borders::ALL)
                    .border_style(ratatui::style::Style::default().fg(SLATE.c800))
                    .border_type(ratatui::widgets::BorderType::Plain)
                    .style(ratatui::style::Style::default().bg(SKY.c200).fg(SKY.c800)),
                    highlight: ratatui::style::Style::default().bg(SKY.c700).fg(SKY.c950),
                    highlight_symbol: "➡︎ ",
                    highlight_spacing: HighlightSpacing::Always,
            },
            focus: Style {
                block: Block::default()
                    .borders(Borders::ALL)
                    .border_style(ratatui::style::Style::default().fg(SKY.c900).fg(ORANGE.c600))
                    .border_type(ratatui::widgets::BorderType::Double)
                    .style(ratatui::style::Style::default().bg(SLATE.c100).fg(SKY.c900)),
                    highlight: ratatui::style::Style::default().bg(ORANGE.c600).fg(SKY.c50),
                    highlight_symbol: "➡︎ ",
                    highlight_spacing: HighlightSpacing::Always,
            },
            error: Style {
                block: Block::default()
                    .borders(Borders::ALL)
                    .border_style(ratatui::style::Style::default().fg(ratatui::style::palette::tailwind::RED.c700))
                    .border_type(ratatui::widgets::BorderType::Double)
                    .style(ratatui::style::Style::default().bg(ratatui::style::palette::tailwind::RED.c950).fg(ratatui::style::Color::White)),
                    highlight: ratatui::style::Style::default().bg(ratatui::style::Color::Red).fg(ratatui::style::Color::White),
                    highlight_symbol: "‼ ",
                    highlight_spacing: HighlightSpacing::Always,
            },
        }
    }
}
impl Styles {
    pub fn render_list<'item, Item, Items>(
        &self,
        title: &str,
        style: FocusState,
        items: Items,
        state: &mut ListState,
        area: Rect,
        frame: &mut Frame,
    )
        where Item: Into<ListItem<'item>>,
              Items: IntoIterator<Item = Item>,
    {
        let style = match style {
            FocusState::Blur => &self.blur,
            FocusState::Focus => &self.focus,
        };
        let list = List::new(items)
            .block(style.block.clone().title(title))
            .highlight_style(style.highlight)
            .highlight_symbol(style.highlight_symbol)
            .highlight_spacing(style.highlight_spacing.clone());
        frame.render_stateful_widget(list, area, state);
    }

    #[inline]
    pub fn style_for(&self, focus: FocusState) -> Style {
        match focus {
            FocusState::Blur => &self.blur,
            FocusState::Focus => &self.focus,
        }.clone()
    }
}

lazy_static! {
    pub static ref STYLES: Styles = Styles::default();
}
