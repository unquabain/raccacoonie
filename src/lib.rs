pub mod error;
pub mod message;
pub mod input_control;
pub mod button;
pub mod styles;
pub mod spinner;
pub mod listview;
pub mod log_viewer;
pub mod popup;
mod prelude_internal;
pub mod prelude;
pub mod tabcontroller;
mod mainlooper;

use prelude_internal::*;

pub trait Model {
    // Render the current view into the provided frame
    // self is mutable to allow updating internal state during rendering
    fn view(&mut self, frame: &mut Frame, area: Rect) -> Result<()>;

    fn init(&mut self) -> Message {
        Default::default()
    }
    fn update(&mut self, _msg: Message) -> Message {
        Default::default()
    }
    fn help(&self) -> Option<String> {
        None
    }

    fn set_focus(&mut self, _focused: styles::FocusState) { }
}

pub trait Runner : Model + Sized {
    fn run(&mut self) -> impl std::future::Future<Output=Result<()>> {
        async {
            mainlooper::MainLooper::run(self).await
        }
    }
}

impl<M: Model> Model for Option<M> {
    fn view(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        match self {
            Some(inner) => inner.view(frame, area),
            None => {
                frame.render_widget(
                    Paragraph::default().block(STYLES.blur.block.clone()),
                    area,
                );
                Ok(())
            }
        }
    }

    fn init(&mut self) -> Message {
        match self {
            Some(inner) => inner.init(),
            None => Message::Noop,
        }
    }
    fn update(&mut self, msg: Message) -> Message {
        match self {
            Some(inner) => inner.update(msg),
            None => Message::Noop,
        }
    }
    fn help(&self) -> Option<String> {
        match self {
            Some(inner) => inner.help(),
            None => None,
        }
    }

    fn set_focus(&mut self, focused: styles::FocusState) {
        match self {
            Some(inner) => inner.set_focus(focused),
            None => (),
        }
    }
}
