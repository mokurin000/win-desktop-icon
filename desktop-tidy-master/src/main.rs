use std::time::{Duration, Instant};

use compio::time::interval;
use desktop_icon::desktop::DesktopView;
use winio::prelude::*;

#[derive(Debug)]
pub struct MainModel {
    window: Child<Window>,
    button: Child<Button>,
    progress: Child<Progress>,

    desktop_view: DesktopView,
    clicked: bool,
    completed: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum MainMessage {
    Noop,
    Close,
    Redraw,
    Clicked,
    SetPosition(usize),
}

const PROGRESS_TIME_MS: u64 = 5000;
const PROGRESS_MAX: f64 = 1000.0;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An error from the UI backend.
    #[error("UI error: {0}")]
    Ui(#[from] winio::Error),
    /// An layouting error
    #[error("Layout error: {0}")]
    Layout(#[from] winio::layout::LayoutError<winio::Error>),

    /// desktop icon error
    #[error("Backend error: {0}")]
    Backend(#[from] desktop_icon::error::AppError),
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
            button: Button = (&window) => {
                text: "Click me",
            },
            progress: Progress = (&window) => {
                pos: 0,
                minimum: 0,
                maximum: PROGRESS_MAX.round() as _,
            }
        }

        window.set_backdrop(Backdrop::Mica)?;
        window.show()?;

        Ok(Self {
            window,
            button,
            progress,
            desktop_view: DesktopView::connect()?,
            clicked: false,
            completed: false,
        })
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) -> std::result::Result<(), Self::Error> {
        let csize = self.window.client_size()?;
        {
            let mut inner = layout! {
                StackPanel::new(Orient::Vertical),
                self.button => {
                    height: 50.0,
                    width: 160.0,
                },
                self.progress => {
                    height: 20.0,
                    width: 160.0,
                }
            };
            let mut panel = layout! {
                Grid::new(
                    vec![GridLength::Stretch(1.0)],
                    vec![GridLength::Stretch(1.0)],
                ),
                inner => {
                    halign: HAlign::Center,
                    valign: VAlign::Center,
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
            self.button => {
                ButtonEvent::Click => MainMessage::Clicked,
            },
            self.progress => {}
        }
    }

    async fn update_children(&mut self) -> std::result::Result<bool, Self::Error> {
        update_children!(self.window, self.button, self.progress)
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
                        desktop_icon::utils::restore_icons(&self.desktop_view)?;
                        sender.output(());
                    }
                    _ => {}
                }
                Ok(false)
            }
            MainMessage::Redraw => Ok(true),
            MainMessage::Clicked => {
                self.clicked = true;
                self.button.disable()?;

                desktop_icon::utils::backup_icons(&self.desktop_view)?;

                let sender = sender.clone();
                spawn_timer(sender);

                Ok(true)
            }
            MainMessage::SetPosition(pos) => {
                self.progress.set_pos(pos)?;
                Ok(true)
            }
        }
    }
}

fn main() -> std::result::Result<(), Error> {
    App::new(env!("CARGO_PKG_NAME"))?.run_until_event::<MainModel>(())?;

    Ok(())
}

fn spawn_timer(sender: ComponentSender<MainModel>) {
    fn sigmoid(x: f64) -> usize {
        let x = 20.0 * x - 10.0;
        let y = PROGRESS_MAX / (1.0 + (-0.5 * x + 0.25).exp());

        y.clamp(0.0, PROGRESS_MAX).round() as usize
    }

    let start = Instant::now();
    compio::runtime::spawn(async move {
        let mut interval = interval(Duration::from_millis(5));
        while start.elapsed() < Duration::from_millis(PROGRESS_TIME_MS) {
            interval.tick().await;
            let pos = sigmoid(start.elapsed().as_millis() as f64 / PROGRESS_TIME_MS as f64);
            sender.post(MainMessage::SetPosition(pos));
        }
        sender.post(MainMessage::SetPosition(PROGRESS_MAX as _));
    })
    .detach();
}
