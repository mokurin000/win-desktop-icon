use std::time::{Duration, Instant};

use compio::time::interval;
use winio::prelude::*;

use desktop_icon::desktop::DesktopView;

use crate::model::MainMessage;
use crate::{Error, MainModel, PROGRESS_MAX, PROGRESS_TIME_MS};

pub fn arrange_icons(view: &DesktopView, rect: Rect) -> std::result::Result<(), Error> {
    let left = rect.min_x() as i32;
    let top = rect.min_y() as i32;
    let right = rect.max_x() as i32 - 80;
    let bottom = rect.max_y() as i32 - 80;

    let icons = view.icons()?;

    for icon in icons {
        if let Some(x) = fastrand::choice(left..right)
            && let Some(y) = fastrand::choice(top..bottom)
        {
            view.icon_set_position(&icon, x, y)?;
        }
    }
    Ok(())
}

pub fn spawn_timer(sender: ComponentSender<MainModel>) {
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
