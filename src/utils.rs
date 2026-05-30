use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

use windows::Win32::Foundation::POINT;

use crate::desktop::DesktopView;
use crate::persist::DeskopIconState;

fn desktop_backup_path() -> PathBuf {
    let path = PathBuf::from(
        std::env::var("APPDATA")
            .or_else(|_| std::env::var("USERPROFILE").map(|s| s + "/AppData/Roaming"))
            .as_deref()
            .unwrap_or("."),
    );
    path.join("desktop_icons.bin")
}

pub fn backup_icons(view: &DesktopView) -> Result<(), eyre::Report> {
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
        .open(desktop_backup_path())?;

    let encoded = bitcode::encode(&states);
    file.write_all(&encoded)?;

    Ok(())
}

pub fn restore_icons(view: &DesktopView) -> Result<(), eyre::Report> {
    let data = std::fs::read(desktop_backup_path())?;
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
