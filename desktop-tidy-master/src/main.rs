use winio::prelude::*;

#[derive(Debug)]
pub struct MainModel {
    window: Child<Window>,
    text: Child<TextBox>,
}

#[derive(Debug, Clone, Copy)]
pub enum MainMessage {
    Noop,
    Close,
    Redraw,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An error from the UI backend.
    #[error("UI error: {0}")]
    Ui(#[from] winio::Error),
    /// An layouting error
    #[error("Layout error: {0}")]
    Layout(#[from] winio::layout::LayoutError<winio::Error>),
}

impl Component for MainModel {
    type Init<'a> = ();
    type Event = ();
    type Message = MainMessage;
    type Error = Error;

    async fn init(
        _init: Self::Init<'_>,
        _sender: &ComponentSender<Self>,
    ) -> std::result::Result<Self, Self::Error> {
        init! {
            window: Window = (()) => {
                text: "Desktop Tidy Master",
                size: Size::new(800.0, 600.0),
                loc: {
                    let monitors = Monitor::all()?;
                    if let Some(monitor) = monitors.first() {
                        let region = monitor.client_scaled();
                        region.origin + region.size / 2.0 - window.size()? / 2.0
                    } else {
                        Point::zero()
                    }
                },
            },
            text: TextBox = (&window) => {
                text: "Hello world",
            },
        }

        window.set_backdrop(Backdrop::Mica)?;
        window.show()?;

        Ok(Self { window, text })
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) -> std::result::Result<(), Self::Error> {
        let csize = self.window.client_size()?;
        {
            let mut panel = layout! {
                Grid::new(
                    vec![GridLength::Stretch(1.0)],
                    vec![GridLength::Stretch(1.0)],
                ),
                self.text => {
                    halign: HAlign::Center,
                    valign: VAlign::Center,
                    height: 120.0,
                    width: 160.0,
                },
            };
            panel.set_size(csize)?;
        }
        Ok(())
    }

    fn render_children(&mut self) -> std::result::Result<(), Self::Error> {
        Ok(())
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        start! {
         sender, default: MainMessage::Noop,
         self.window => {
             WindowEvent::Close => MainMessage::Close,
             WindowEvent::Resize | WindowEvent::ThemeChanged => MainMessage::Redraw,
         },
         self.text => {
            TextBoxEvent::Change => MainMessage::Redraw,
         }
        }
    }

    async fn update_children(&mut self) -> std::result::Result<bool, Self::Error> {
        update_children!(self.window, self.text)
    }

    async fn update(
        &mut self,
        message: Self::Message,
        sender: &ComponentSender<Self>,
    ) -> std::result::Result<bool, Self::Error> {
        match message {
            MainMessage::Noop => Ok(false),
            MainMessage::Close => {
                match MessageBox::new()
                    .title("Exit")
                    .message("Quit this program?")
                    .style(MessageBoxStyle::Info)
                    .buttons(MessageBoxButton::Yes | MessageBoxButton::No)
                    .show(&self.window)
                    .await?
                {
                    MessageBoxResponse::Yes => {
                        sender.output(());
                    }
                    _ => {}
                }
                Ok(false)
            }
            MainMessage::Redraw => Ok(true),
        }
    }
}

fn main() -> std::result::Result<(), Error> {
    App::new(env!("CARGO_PKG_NAME"))?.run_until_event::<MainModel>(())?;

    Ok(())
}
