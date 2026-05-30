use std::marker::PhantomData;
use std::ptr::NonNull;

use crate::com::ComApartment;
use crate::error::{AppError, Result};

use windows::core::{Interface, PWSTR};
use windows::Win32::System::Com::{CoCreateInstance, CoTaskMemFree, IServiceProvider, CLSCTX_ALL};
use windows::Win32::System::Variant::{VariantInit, VARIANT};
use windows::Win32::UI::Shell::Common::{ITEMIDLIST, STRRET};
use windows::Win32::UI::Shell::{
    IEnumIDList, IFolderView, IShellBrowser, IShellFolder, IShellView, IShellWindows,
    SID_STopLevelBrowser, ShellWindows, SHGDN_NORMAL, SVGIO_ALLVIEW, SWC_DESKTOP,
    SWFO_NEEDDISPATCH,
};

pub struct DesktopIcon<'a> {
    inner: NonNull<ITEMIDLIST>,
    _mark: PhantomData<&'a ()>,
}

impl Drop for DesktopIcon<'_> {
    fn drop(&mut self) {
        unsafe {
            CoTaskMemFree(Some(self.inner.as_ptr() as _));
        }
    }
}

#[derive(Debug)]
pub struct DesktopIconInfo {
    pub x: i32,
    pub y: i32,
    pub name: String,
}

pub struct DesktopView {
    folder_view: IFolderView,
    shell_folder: IShellFolder,

    // must drop after IShellFolder & IFolderView
    // do not move this field.
    _com: ComApartment,
}

impl DesktopView {
    pub fn connect() -> Result<Self> {
        let com = ComApartment::init()?;

        let folder_view =
            find_desktop_folder_view().map_err(|_| AppError::DesktopViewUnavailable)?;

        let shell_folder = unsafe { folder_view.GetFolder()? };

        Ok(Self {
            _com: com,
            folder_view,
            shell_folder,
        })
    }

    pub fn icons(&self) -> Result<Vec<DesktopIcon<'_>>> {
        let enumerator = unsafe { self.folder_view.Items(SVGIO_ALLVIEW)? };

        let mut icons = Vec::new();

        while let Some(idlist) = next_item(&enumerator)? {
            icons.push(DesktopIcon {
                inner: idlist,
                _mark: Default::default(),
            });
        }

        Ok(icons)
    }

    pub fn icon_info(&self, icon: &DesktopIcon) -> Result<DesktopIconInfo> {
        let position = unsafe { self.folder_view.GetItemPosition(icon.inner.as_ptr()) }?;

        let name = self.read_name(icon)?;

        Ok(DesktopIconInfo {
            x: position.x,
            y: position.y,
            name,
        })
    }

    fn read_name(&self, idlist: &DesktopIcon) -> Result<String> {
        let idlist = idlist.inner.as_ptr();
        let mut strret = STRRET::default();

        unsafe {
            self.shell_folder
                .GetDisplayNameOf(idlist, SHGDN_NORMAL, &mut strret)?;
        }

        let mut name_ptr = PWSTR::null();

        unsafe {
            windows::Win32::UI::Shell::StrRetToStrW(&mut strret, Some(idlist), &mut name_ptr)?;
        }

        let name = unsafe { name_ptr.to_string()? };

        unsafe {
            CoTaskMemFree(Some(name_ptr.0 as _));
        }

        Ok(name)
    }
}

fn next_item(enumerator: &IEnumIDList) -> Result<Option<NonNull<ITEMIDLIST>>> {
    let mut pidls = [std::ptr::null_mut::<ITEMIDLIST>(); 1];

    unsafe {
        enumerator.Next(&mut pidls, None).ok()?;
    }

    Ok(NonNull::new(pidls[0]))
}

fn find_desktop_folder_view() -> Result<IFolderView> {
    let shell_windows: IShellWindows =
        unsafe { CoCreateInstance(&ShellWindows, None, CLSCTX_ALL)? };

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

    Ok(shell_view.cast()?)
}
