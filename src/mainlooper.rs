use crate::prelude_internal::*;
use ratatui::crossterm::event::{
    read,
    poll,
    Event,
    KeyCode,
    KeyModifiers,
};
use tokio::{
    time::Duration,
    sync::mpsc
};
// Force a redraw after processing this many events
const EVENT_BATCH_SIZE: usize = 20;

type QuitSignal = std::sync::Arc<tokio::sync::SetOnce<()>>;
fn new_quit_signal() -> QuitSignal {
     std::sync::Arc::new(tokio::sync::SetOnce::new())
}


async fn event_loop(tx: mpsc::Sender<Message>, quit: QuitSignal) {
    // Listen for key events until told to quit
    loop {
        // If we don't get a key event, check every 100ms if we should quit.
        loop {
            if quit.initialized() {
                return;
            }
            match poll(Duration::from_millis(100)) {
                Ok(false) => continue,
                Ok(true) => break,
                Err(e) => {
                    log::error!("Error polling for key event: {}", e);
                    let _ = tx.send(Message::Error(e.into())).await;
                    return;
                }
            }
        }
        match read() {
            Ok(Event::Key(key_event)) => {
                // Ctrl-C to quit
                if key_event.modifiers == KeyModifiers::CONTROL && key_event.code == KeyCode::Char('c') {
                    _ = tx.send(Message::Quit).await;
                    return
                } else {
                    // Send other key events to the main loop
                    _ = tx.send(Message::KeyPress(key_event)).await
                };
            }
            // Handle terminal resize events
            Ok(Event::Resize(width, height)) => {
                let _ = tx.send(Message::Resize(width as usize, height as usize)).await;
            }

            // Wrap up error events
            Err(e) => {
                let _ = tx.send(Message::ErrorFatal(e.into())).await;
                return;
            }
            // ignore other events
            _ => {}
        }
    }
}

#[derive(PartialEq, Eq)]
enum BreakDepth {
    ReadMoreEvents,
    DrawModel,
    EndProgram,
}


pub struct MainLooper<'m, M: Model> {
    quit: QuitSignal,
    term: ratatui::DefaultTerminal,
    tx: mpsc::Sender<Message>,
    rx: mpsc::Receiver<Message>,
    keyloop: tokio::task::JoinHandle<()>,
    model: &'m mut M,

}
impl<'m, M: Model> MainLooper<'m, M> {
    fn new(model: &'m mut M) -> Self {
        let (tx, rx) = mpsc::channel(EVENT_BATCH_SIZE);
        let quit = new_quit_signal();
        Self {
            quit: quit.clone(),
            term: ratatui::init(),
            tx: tx.clone(), rx,
            keyloop: tokio::spawn(event_loop(tx, quit)),
            model, 
        }
    }

    #[inline]
    async fn inner(&mut self) -> Result<BreakDepth> {
        match self.rx.try_recv() {
            // No more events to process right now, proceed to redraw
            Err(mpsc::error::TryRecvError::Empty) => Ok(BreakDepth::DrawModel),

            // Premature disconnection of the event channel
            Err(mpsc::error::TryRecvError::Disconnected) => {
                ratatui::restore();
                Err(Error::TerminalError("Premature disconnection of event channel".to_string()))
            },
            // Skip redraw and exit main loop
            Ok(Message::Quit) => Ok(BreakDepth::EndProgram),

            // Error occurred, exit with error
            Ok(Message::ErrorFatal(e)) => {
                ratatui::restore();
                return Err(e);
            },

            // Skip processing and redraw immediately
            Ok(Message::Redraw) => Ok(BreakDepth::DrawModel),

            // Ignore no-op messages
            Ok(Message::Noop) => Ok(BreakDepth::ReadMoreEvents),

            // Process other messages
            Ok(msg) => {
                self.model.update(msg)
                    .spawn(self.tx.clone());
                Ok(BreakDepth::ReadMoreEvents)
            }
        }
    }

    #[inline]
    async fn mainloop(&mut self) -> Result<BreakDepth> {
        self.term.try_draw(|f: &mut Frame| {
            let area = f.area();
            self.model.view(f, area)
        })?;
        // Try to process up to EVENT_BATCH_SIZE events without redrawing
        // to improve responsiveness.
        for _ in 0..EVENT_BATCH_SIZE {
            match self.inner().await {
                Err(err) => return Err(err),
                Ok(BreakDepth::ReadMoreEvents) => continue,
                Ok(bd) => return Ok(bd),
            }
        }
        // end 'inner
        Ok(BreakDepth::DrawModel)
    }
    fn finalize(mut self) {
        _ = self.quit.set(());
        ratatui::restore();
        self.rx.close();
        if !self.keyloop.is_finished() {
            self.keyloop.abort();
        }
    }
    pub async fn run(model: &'m mut M) -> Result<()> {
        let mut runner = Self::new(model);
        runner.model.init().spawn(runner.tx.clone());
        loop {
            runner.term.try_draw(|f: &mut Frame| {
                let area = f.area();
                runner.model.view(f, area)
            })?;
            match runner.mainloop().await {
                Err(err) => {
                    runner.finalize();
                    return Err(err);
                }
                Ok(BreakDepth::EndProgram) => break,
                Ok(_) => continue,
            }
        }
        runner.finalize();
        Ok(())
    }
}
