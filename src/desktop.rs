use std::ptr::NonNull;

use crate::com::ComApartment;
use crate::error::{AppError, Result};
use windows::core::{Interface, PWSTR};
use windows::Win32::System::Com::{CoCreateInstance, CoTaskMemFree, IServiceProvider};
use windows::Win32::System::Variant::{VariantInit, VARIANT};
use windows::Win32::UI::Shell::Common::{ITEMIDLIST, STRRET};
use windows::Win32::UI::Shell::{
    IEnumIDList, IFolderView, IShellBrowser, IShellFolder, IShellView, IShellWindows,
    SID_STopLevelBrowser, ShellWindows, SHGDN_NORMAL, SVGIO_ALLVIEW, SWC_DESKTOP,
    SWFO_NEEDDISPATCH,
};

/// This will free memory of ITEMIDLIST on drop.
#[derive(Debug, Clone)]
pub struct DesktopIcon {
    pub x: i32,
    pub y: i32,
    pub name: String,
    pub idlist: NonNull<ITEMIDLIST>,
}

impl Drop for DesktopIcon {
    fn drop(&mut self) {
        unsafe { CoTaskMemFree(Some(self.idlist.as_ptr() as _)) };
    }
}

pub fn list_desktop_icons() -> Result<Vec<DesktopIcon>> {
    let _com = ComApartment::init()?;
    let view = find_desktop_folder_view().map_err(|_| AppError::DesktopViewUnavailable)?;
    let folder = unsafe { view.GetFolder()? };
    let enumerator = unsafe { view.Items(SVGIO_ALLVIEW)? };

    let mut icons = Vec::new();
    while let Some(idlist) = next_item(&enumerator)? {
        if let Ok(icon) = read_icon(&view, &folder, idlist) {
            icons.push(icon);
        }
    }

    Ok(icons)
}

fn find_desktop_folder_view() -> Result<IFolderView> {
    let shell_windows: IShellWindows =
        unsafe { CoCreateInstance(&ShellWindows, None, windows::Win32::System::Com::CLSCTX_ALL)? };

    let var_loc: VARIANT = (windows::Win32::UI::Shell::CSIDL_DESKTOP as i32).into();
    let var_empty = unsafe { VariantInit() };
    let mut hwnd = 0;

    let dispatch = unsafe {
        shell_windows
            .FindWindowSW(
                &var_loc,
                &var_empty,
                SWC_DESKTOP,
                &mut hwnd,
                SWFO_NEEDDISPATCH,
            )
            .map_err(|_| AppError::MissingDispatch)?
    };

    let service_provider: IServiceProvider = dispatch.cast()?;
    let browser: IShellBrowser = unsafe { service_provider.QueryService(&SID_STopLevelBrowser)? };
    let shell_view: IShellView = unsafe { browser.QueryActiveShellView()? };
    let folder_view: IFolderView = shell_view.cast()?;

    Ok(folder_view)
}

fn next_item(enumerator: &IEnumIDList) -> Result<Option<NonNull<ITEMIDLIST>>> {
    let mut pidls = [std::ptr::null_mut::<ITEMIDLIST>(); 1];

    unsafe {
        enumerator.Next(&mut pidls, None).ok()?;
    }

    Ok(pidls.get(0).and_then(|&p| NonNull::new(p)))
}

fn read_icon(
    view: &IFolderView,
    folder: &IShellFolder,
    idlist: NonNull<ITEMIDLIST>,
) -> Result<DesktopIcon> {
    let mut strret = STRRET::default();
    unsafe {
        folder.GetDisplayNameOf(idlist.as_ptr(), SHGDN_NORMAL, &mut strret)?;
    }

    let mut name = PWSTR::null();
    unsafe {
        windows::Win32::UI::Shell::StrRetToStrW(&mut strret, Some(idlist.as_ptr()), &mut name)?
    };
    let name_string = unsafe { name.to_string()? };

    unsafe {
        CoTaskMemFree(Some(name.0 as _));
    }

    let position = unsafe { view.GetItemPosition(idlist.as_ptr()) }?;

    Ok(DesktopIcon {
        x: position.x,
        y: position.y,
        name: name_string,
        idlist,
    })
}
