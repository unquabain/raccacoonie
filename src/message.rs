use crate::error::*;
use ratatui::crossterm::event::KeyEvent;
use uuid::Uuid;
use tokio::{
    spawn,
    time::{Duration, sleep},
    process::Command,
    sync::mpsc::Sender,
};

#[derive(Debug, Clone, Default)]
pub enum Message {

    // Does nothing, but can be used as a dummy to chain start to commands
    #[default]
    Noop,

    // Signals the application to exit
    Quit,

    // Signals a quit for a sub-screen or dialog
    Dismiss,

    // Simple affirmative or negative response
    Yes,
    No,

    // Signals the application to redraw the UI, jumping the event queue
    Redraw,

    // An error occurred
    Error(Error),

    // Dismiss an error
    DismissError,

    // A fatal error that should exit the application
    ErrorFatal(Error),

    // Terminal resize event
    Resize(usize, usize),

    // A key was pressed
    KeyPress(KeyEvent),

    // A choice was made from a list of options
    Choice(usize),

    // Timer tick and tock messages
    // Use the Message::tick(duration) function to create a tick message
    // that generates a tock message after the specified duration.
    Tik(Uuid, Duration),
    Tok(Uuid),

    // Composite messages
    // Execute all messages in parallel
    Batch(Vec<Message>),
    // Execute all messages in sequence
    Sequence(Vec<Message>),

    // Shell command input
    ShellCommand(Vec<String>),
    ShellCommandOutput(String),
}
impl Message {
    pub fn error(err: Error) -> Message {
        Message::Error(err)
    }
    pub fn errorf(fmt: impl std::fmt::Display) -> Message {
        Message::Error(Error::OwnedError(fmt.to_string()))
    }
    pub fn choose(choice: usize) -> Message {
        Message::Choice(choice)
    }
    pub fn tick(duration: Duration) -> (Uuid, Message) {
        let id = Uuid::new_v4();
        (
            id,
            Message::Tik(id.clone(), duration)
        )
    }
    pub fn or(self, next: Message) -> Message {
        match self {
            Message::Noop => next,
            _ => next
        }
    }
    pub fn or_else(self, next: impl FnOnce() -> Message) -> Message {
        match self {
            Message::Noop => next(),
            _ => self
        }
    }
    pub fn and(self, next: Message) -> Message {
        if let Message::Noop = next {
            return self;
        }
        match self {
            Message::Noop => next,
            Message::Batch(mut msgs) => {
                msgs.push(next);
                Message::Batch(msgs)
            }
            _ => Message::Batch(vec![self, next]),
        }
    }
    pub fn then(self, next: Message) -> Message {
        if let Message::Noop = next {
            return self;
        }
        match self {
            Message::Noop => next,
            Message::Sequence(mut msgs) => {
                msgs.push(next);
                Message::Sequence(msgs)
            }
            _ => Message::Sequence(vec![self, next]),
        }
    }
    async fn execute(self) -> Option<Message> {
        match self {
            Message::Noop => None,
            Message::Tik(id, duration) => {
                sleep(duration).await;
                Some(Message::Tok(id))
            }
            Message::ShellCommand(cmd) => {
                let output = Command::new(&cmd[0])
                    .args(&cmd[1..])
                    .output()
                    .await;
                match output {
                    Ok(output) => {
                        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                        Some(Message::ShellCommandOutput(stdout))
                    }
                    Err(e) => Some(Message::Error(e.into())),
                }
            }
            _ => Some(self),
        }
    }
    pub fn spawn(self, tx: Sender<Message>) {
        match self {
            Message::Sequence(msgs) => {
                spawn(async move {
                    for cmd in msgs {
                        if let Some(msg) = cmd.execute().await {
                            let _ = tx.send(msg).await;
                        }
                    }
                });
            },
            Message::Batch(msgs) => {
                for msg in msgs {
                    msg.spawn(tx.clone())
                }
            },
            _ => {
                spawn(async move {
                    if let Some(msg) = self.execute().await {
                        let _ = tx.send(msg).await;
                    };
                });
            }
        };
    }
    pub fn into_option(self) -> Option<Message> {
        match self {
            Message::Noop => None,
            _ => Some(self)
        }
    }
}
