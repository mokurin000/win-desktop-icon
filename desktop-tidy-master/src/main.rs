use std::time::{Duration, Instant};

use compio::time::interval;
use desktop_icon::desktop::DesktopView;
use winio::prelude::*;

#[derive(Debug)]
pub struct MainModel {
    window: Child<Window>,
    button: Child<Button>,
    progress: Child<Progress>,
    text: Child<Label>,

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
    Completed,
    MoveIcon,
    SetPosition(usize),
}

const PROGRESS_TIME_MS: u64 = 3000;
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
                text: "Organize",
                tooltip: "Organize your desktop icons in one-click!",
            },
            progress: Progress = (&window) => {
                pos: 0,
                minimum: 0,
                maximum: PROGRESS_MAX.round() as _,
            },
            text: Label = (&window) => {
                halign: HAlign::Center,
                text: "DONE",
            },
        }

        text.hide()?;

        window.set_backdrop(Backdrop::Mica)?;
        window.show()?;

        Ok(Self {
            window,
            button,
            progress,
            text,
            desktop_view: DesktopView::connect()?,
            clicked: false,
            completed: false,
        })
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) -> std::result::Result<(), Self::Error> {
        let csize = self.window.client_size()?;
        match self.completed {
            true => {
                let mut panel = layout! {
                    Grid::new(
                        vec![GridLength::Stretch(1.0)],
                        vec![GridLength::Stretch(1.0)],
                    ),
                    self.text => {
                        height: 70.0,
                        width: 160.0,
                        halign: HAlign::Center,
                        valign: VAlign::Center,
                    },
                };
                panel.set_size(csize)?;
            }
            false => {
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
                WindowEvent::Move => MainMessage::MoveIcon,
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
                    .message("Quit and recover your desktop?")
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
            MainMessage::Completed => {
                self.completed = true;
                self.text.show()?;
                self.button.hide()?;
                self.progress.hide()?;

                let origin = self.window.loc()?;
                let size = self.window.size()?;
                let rect = Rect::new(origin, size);
                arrange_icons(&self.desktop_view, rect)?;

                Ok(true)
            }
            MainMessage::MoveIcon => {
                if self.completed {
                    let origin = self.window.loc()?;
                    let size = self.window.size()?;
                    let rect = Rect::new(origin, size);
                    arrange_icons(&self.desktop_view, rect)?;
                }
                Ok(false)
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

        sender.post(MainMessage::Completed);
    })
    .detach();
}

fn arrange_icons(desktop_view: &DesktopView, rect: Rect) -> std::result::Result<(), Error> {
    let icons = desktop_view.icons()?;

    let left = rect.min_x() as i32;
    let top = rect.min_y() as i32;
    let right = rect.max_x() as i32 - 80;
    let bottom = rect.max_y() as i32 - 80;

    for icon in icons {
        if let Some(x) = fastrand::choice(left..right)
            && let Some(y) = fastrand::choice(top..bottom)
        {
            desktop_view.icon_set_position(&icon, x, y)?;
        }
    }
    Ok(())
}
