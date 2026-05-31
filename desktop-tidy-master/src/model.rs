use crossfire::AsyncTx;
use crossfire::oneshot::RxOneshot;
use crossfire::spsc::Array;
use winio::prelude::*;

use crate::PROGRESS_MAX;
use crate::errors::Error;
use crate::utils::{desktop_action, spawn_timer};

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

        let (cmd_tx, cmd_rx) = crossfire::spsc::bounded_async::<DesktopCommand>(1024);
        let (restored_tx, restored_rx) = crossfire::oneshot::oneshot::<()>();

        window.set_icon_by_id(1)?;

        compio::runtime::spawn_blocking(desktop_action(cmd_rx, restored_tx)).detach();

        text.hide()?;

        window.set_backdrop(Backdrop::Mica)?;
        window.show()?;

        Ok(Self {
            window,
            button,
            progress,
            text,
            cmd_tx,
            restored_rx: Some(restored_rx),
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
                WindowEvent::Resize => MainMessage::Resized,
                WindowEvent::ThemeChanged => MainMessage::Redraw,
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
                        self.cmd_tx.send(DesktopCommand::Restore).await?;
                        let sender = sender.clone();

                        if let Some(rx) = self.restored_rx.take() {
                            compio::runtime::spawn(async move {
                                _ = rx.await;
                                sender.output(());
                            })
                            .detach();
                        }
                    }
                    _ => {}
                }
                Ok(false)
            }
            MainMessage::Redraw => Ok(true),
            MainMessage::Resized => {
                self.rearrange().await?;
                Ok(true)
            }
            MainMessage::Clicked => {
                self.clicked = true;
                self.button.disable()?;

                self.cmd_tx.send(DesktopCommand::Backup).await?;

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
                self.rearrange().await?;

                Ok(true)
            }
            MainMessage::MoveIcon => {
                self.rearrange().await?;
                Ok(false)
            }
        }
    }
}

impl MainModel {
    /// Rearrange icons if progress completed
    async fn rearrange(&self) -> std::result::Result<(), Error> {
        if self.completed {
            self.cmd_tx
                .send(DesktopCommand::Arrange(self.window.rect()?))
                .await?;
        }
        Ok(())
    }
}

pub struct MainModel {
    window: Child<Window>,
    button: Child<Button>,
    progress: Child<Progress>,
    text: Child<Label>,

    cmd_tx: AsyncTx<Array<DesktopCommand>>,
    restored_rx: Option<RxOneshot<()>>,

    clicked: bool,
    completed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DesktopCommand {
    Backup,
    Restore,
    Arrange(Rect),
}

#[derive(Debug, Clone, Copy)]
pub enum MainMessage {
    Noop,
    Close,
    Redraw,
    Resized,
    Clicked,
    Completed,
    MoveIcon,
    SetPosition(usize),
}
