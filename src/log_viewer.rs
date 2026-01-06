use crate::prelude_internal::*;
use ratatui::crossterm::event::{KeyEvent, KeyCode, KeyModifiers};
use tui_logger::{TuiLoggerWidget, TuiWidgetState};

#[derive(Debug,Clone,PartialEq,Eq,Default)]
enum LogViewState {
    #[default]
    Hidden,
    Shown,
    Listening,
}

pub struct LogViewer<M: Model> {
    model: M,
    state: TuiWidgetState,
    view_state: LogViewState,
}

impl<M: Model> LogViewer<M> {
    pub fn new(model: M) -> Self {
        Self{
            model,
            state: TuiWidgetState::new(),
            view_state: Default::default(),
        }
    }
    pub fn into_model(self) -> M {
        self.model
    }
    pub fn handle_keypress(&mut self, msg: Message) -> Message {
        match self.view_state {
            LogViewState::Hidden => {
                match msg {
                    Message::KeyPress(KeyEvent{code: KeyCode::Char('?'), .. }) => {
                        self.view_state = LogViewState::Shown;
                        Message::Redraw
                    }
                    _ => Message::Noop,
                }
            },
            LogViewState::Shown => {
                match msg {
                    Message::KeyPress(KeyEvent{code: KeyCode::Esc, ..}) 
                    | Message::KeyPress(KeyEvent{code: KeyCode::Char('?'), .. }) => {
                        self.view_state = LogViewState::Hidden;
                        Message::Redraw
                    }
                    Message::KeyPress(KeyEvent{code: KeyCode::Char('l'), modifiers, ..})
                        if modifiers.contains(KeyModifiers::CONTROL) => {
                            self.view_state = LogViewState::Listening;
                            Message::Redraw
                        }
                    _ => Message::Noop
                }
            }
            LogViewState::Listening => {
                match msg {
                    Message::KeyPress(KeyEvent{code: KeyCode::Esc, ..}) 
                    | Message::KeyPress(KeyEvent{code: KeyCode::Char('?'), .. }) => {
                        self.view_state = LogViewState::Hidden;
                        Message::Redraw
                    }
                    Message::KeyPress(KeyEvent{code: KeyCode::Char('l'), modifiers, ..})
                        if modifiers.contains(KeyModifiers::CONTROL) => {
                            self.view_state = LogViewState::Shown;
                            Message::Redraw
                        }
                    _ => self.model.update(msg)
                }
            }
        }
    }

}
pub fn init_logging(level: log::LevelFilter) -> Result<()> {
    tui_logger::init_logger(level)?;
    Ok(())
}

impl<M: Model> Model for LogViewer<M> {
    fn view(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        use ratatui::style::{
            Style,
            palette::tailwind::*
        };
        use tui_logger::{
            TuiLoggerLevelOutput::*
        };
        use ratatui::widgets::{
            Block,
            Borders,
            BorderType
        };
        self.model.view(frame, area)?;
        let help_area = Rect {
            x: area.x,
            width: area.width,
            y: area.y + area.height-3,
            height: 3,
        };
        frame.render_widget(Clear, help_area);
        frame.render_widget(
            Paragraph::new(self.help().unwrap()).block(STYLES.blur.block.clone().title("Help")),
            help_area,
        );
        if self.view_state == LogViewState::Hidden {
            return Ok(())
        }
        let rows: [Rect; 3] = Layout::vertical([
            Constraint::Min(0),
            Constraint::Ratio(3, 5),
            Constraint::Min(0),
        ]).areas(area);
        let cols: [Rect; 3] = Layout::horizontal([
            Constraint::Min(0),
            Constraint::Ratio(3, 5),
            Constraint::Min(0),
        ]).areas(rows[1]);

        frame.render_widget(Clear, cols[1]);
        let widget: TuiLoggerWidget = TuiLoggerWidget::default()
            .state(&self.state)
            .output_level(Some(Abbreviated))
            .output_file(false)
            .output_timestamp(None)
            .output_target(false)
            .output_line(false)
            .block(Block::default().title("Log").borders(Borders::ALL).border_type(BorderType::Thick))
            .style_error(Style::default().fg(RED.c900).bg(RED.c300))
            .style_warn(Style::default().fg(AMBER.c500).bg(AMBER.c100))
            .style_info(Style::default().fg(SKY.c200).bg(SKY.c500))
            .style_debug(Style::default().fg(SLATE.c800).bg(SLATE.c200))
            .style_trace(Style::default().fg(SLATE.c400).bg(SLATE.c50))
        ;
        frame.render_widget(widget, cols[1]);
        Ok(())
    }

    fn update(&mut self, msg: Message) -> Message {
        match self.view_state {
            LogViewState::Hidden => self.model.update(msg.clone()),
            LogViewState::Shown | LogViewState::Listening => Message::Noop,
        }.or_else(|| self.handle_keypress(msg))
    }
    fn help(&self) -> Option<String> {
        match self.view_state {
            LogViewState::Hidden => match self.model.help() {
                Some(help) => Some(format!("{}; ? for logs", help)),
                None => Some("? for logs".to_owned()),
            }
            LogViewState::Shown | LogViewState::Listening => Some("CTRL+L to toggle listening; ESC or ? to dismiss".to_owned()),
        }
    }
    fn init(&mut self) -> Message {
        self.model.init()
    }
}

impl<M: Model> Runner for LogViewer<M> {}
