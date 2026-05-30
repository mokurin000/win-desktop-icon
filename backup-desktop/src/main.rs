use argh::FromArgs;
use spdlog::error;

use desktop_icon::desktop::DesktopView;
use desktop_icon::utils::{backup_icons, restore_icons};

#[derive(FromArgs)]
/// Backup or restore desktop icon positions.
struct Args {
    /// restore icon positions from desktop.bin
    #[argh(switch, short = 'r')]
    restore: bool,
}

fn main() -> Result<(), eyre::Report> {
    let args: Args = argh::from_env();

    let view = DesktopView::connect()?;

    if args.restore {
        restore_icons(&view)?;
    } else {
        backup_icons(&view).inspect_err(|e| {
            error!("Backup failed: {e}");
        })?;
    }

    Ok(())
}
