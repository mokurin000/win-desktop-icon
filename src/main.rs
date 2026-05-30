use std::{fs::OpenOptions, io::Write};

use argh::FromArgs;
use windows::Win32::Foundation::POINT;

use win_desktop_icon::{desktop::DesktopView, persist::DeskopIconState};

#[derive(FromArgs)]
/// Backup or restore desktop icon positions.
struct Args {
    /// restore icon positions from desktop.bin
    #[argh(switch, short = 'r')]
    restore: bool,
}

fn backup_icons(view: &DesktopView) -> Result<(), eyre::Report> {
    let icons = view.icons()?;

    let mut states = Vec::with_capacity(icons.len());

    for icon in icons {
        let point = view.icon_info(&icon)?;
        let POINT {
            x: position_x,
            y: position_y,
        } = point.position;

        let state = DeskopIconState {
            pidl: icon.as_bytes().to_vec(),
            position_x,
            position_y,
        };

        states.push(state);
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(false)
        .read(false)
        .truncate(true)
        .write(true)
        .open("desktop.bin")?;

    let encoded = bitcode::encode(&states);
    file.write_all(&encoded)?;

    Ok(())
}

fn restore_icons(view: &DesktopView) -> Result<(), eyre::Report> {
    let data = std::fs::read("desktop.bin")?;
    let states: Vec<DeskopIconState> = bitcode::decode(&data)?;

    for mut state in states {
        let point = state.point();
        let icon = unsafe { view.icon_from_bytes(&mut state.pidl) };
        if let Err(e) = view.icon_set_position(&icon, &point) {
            println!("Error: {e}");
        }
        println!("point: {point:?} recovered");
    }

    Ok(())
}

fn main() -> Result<(), eyre::Report> {
    let args: Args = argh::from_env();

    let view = DesktopView::connect()?;

    if args.restore {
        restore_icons(&view)?;
    } else {
        backup_icons(&view)?;
    }

    Ok(())
}
