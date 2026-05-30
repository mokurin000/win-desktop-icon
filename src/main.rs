mod com;
mod desktop;
mod error;

use windows::Win32::Foundation::POINT;

use crate::desktop::DesktopView;

fn main() -> Result<(), eyre::Report> {
    let view = DesktopView::connect()?;
    let icons = view.icons()?;
    for icon in icons {
        let icon_info = view.icon_info(&icon)?;
        println!("({:4}, {:4}) {}", icon_info.x, icon_info.y, icon_info.name);

        if icon_info.name == "QQ" {
            view.icon_set_position(&icon, &POINT { x: 820, y: 800 })?;
        }
    }

    Ok(())
}
