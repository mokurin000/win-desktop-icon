use winio::prelude::*;

use desktop_icon::desktop::DesktopView;

use crate::Error;

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
