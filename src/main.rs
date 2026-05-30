use win_desktop_icon::desktop::DesktopView;

fn main() -> Result<(), eyre::Report> {
    let view = DesktopView::connect()?;
    let icons = view.icons()?;

    for icon in icons {}

    Ok(())
}
