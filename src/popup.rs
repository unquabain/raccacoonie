use crate::prelude_internal::*;
use ratatui::{
    widgets::Borders,
    layout::Margin,
};

pub struct Popup(String);
impl Popup {
    pub async fn show<M: std::fmt::Display>(message: M) -> Result<()> {
        let mut p = Popup(format!("{message}"));
        p.run().await
    }
}

impl Runner for Popup {}

impl Model for Popup {
    fn update(&mut self, msg: Message) -> Message {
        match msg {
        Message::KeyPress(_)
            => Message::Quit,
            _ =>Message::Noop,
        }
    }
    fn view(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        frame.render_widget(Clear, area);
        let rows: [Rect; 3] = Layout::vertical([
            Constraint::Min(0),
            Constraint::Ratio(3,5),
            Constraint::Min(0),
        ]).areas(area);
        let cols: [Rect; 3] = Layout::horizontal([
            Constraint::Min(0),
            Constraint::Ratio(3,5),
            Constraint::Min(0),
        ]).areas(rows[1]);
        frame.render_widget(STYLES.focus.block.clone(), cols[1]);

        let areas: [Rect; 2] = Layout::vertical([
            Constraint::Min(0),
            Constraint::Length(3),
        ]).areas(cols[1].inner(Margin::new(1, 1)));
        frame.render_widget(
            Paragraph::new(self.0.as_str()).block(STYLES.focus.block.clone().borders(Borders::NONE)).centered(),
            areas[0]
        );
        frame.render_widget(
            Paragraph::new("Press any key to continue").block(STYLES.focus.block.clone().title("Help")).centered(),
            areas[1]
        );
        Ok(())
    }
}
