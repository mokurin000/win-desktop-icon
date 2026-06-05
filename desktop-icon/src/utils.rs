use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

use spdlog::{error, info};
use windows::Win32::Foundation::POINT;

use crate::desktop::{DesktopIcon, DesktopView};
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

    info!("Backed up {} icons", states.len());

    Ok(())
}

pub fn restore_icons(view: &DesktopView) -> Result<(), AppError> {
    let data = std::fs::read(desktop_backup_path())?;
    let states: Vec<DeskopIconState> = bitcode::decode(&data)?;

    let total = states.len();
    let succ = states
        .into_iter()
        .map(|mut state| {
            let point = state.point();
            let icon = unsafe { DesktopIcon::from_rust(&mut state.pidl) };
            view.icon_set_point(&icon, &point).inspect_err(|e| {
                error!("Error: {e}");
            })?;

            // Try read icon info to check failure due to icon renamed
            view.icon_info(&icon)
        })
        .filter(Result::is_ok)
        .count();

    info!("Recovered {succ}/{total} icons");

    Ok(())
}
