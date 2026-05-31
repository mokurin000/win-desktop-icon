use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

use spdlog::{error, info};
use windows::Win32::Foundation::POINT;

use crate::desktop::DesktopView;
use crate::error::AppError;
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

pub fn backup_icons(view: &DesktopView) -> Result<(), AppError> {
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

pub fn restore_icons(view: &DesktopView) -> Result<(), AppError> {
    let data = std::fs::read(desktop_backup_path())?;

    let mut states: Vec<DeskopIconState> = bitcode::decode(&data)?;
    let total = states.len();

    let icons = states
        .iter_mut()
        .map(|state| {
            let icon = unsafe { view.icon_from_bytes(&mut state.pidl) };

            (icon, state.position_x, state.position_y)
        })
        .collect::<Vec<_>>();

    if let Err(e) = view.icon_set_positions(&icons) {
        error!("Recover failed: {e}");
    }

    info!("Tried to recover {total} icons");
    Ok(())
}
