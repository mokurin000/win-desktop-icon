mod com;
mod desktop;
mod error;

use crate::desktop::list_desktop_icons;
use crate::error::Result;

fn run() -> Result<()> {
    let icons = list_desktop_icons()?;
    println!("桌面图标列表:\n");
    for icon in icons {
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
