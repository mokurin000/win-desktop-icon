mod com;
mod desktop;
mod error;

use crate::{desktop::DesktopView, error::Result};

fn run() -> Result<()> {
    let view = DesktopView::connect()?;
    let icons = view.icons()?;
    println!("桌面图标列表:\n");
    for icon in icons {
        println!("fetching info...");
        let icon = view.icon_info(&icon)?;
        println!("({:4}, {:4}) {}", icon.x, icon.y, icon.name);
    }

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{err:?}");
        std::process::exit(1);
    }
}
