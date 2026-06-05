use std::ptr::NonNull;

use windows::Win32::System::Com::CoTaskMemFree;
use windows::Win32::UI::Shell::Common::ITEMIDLIST;
use windows::Win32::UI::Shell::ILGetSize;

use crate::desktop::DesktopView;

pub struct DesktopIcon<'desktop> {
    pub(crate) inner: NonNull<ITEMIDLIST>,
    mut_ref: Option<&'desktop mut [u8]>,
    _view: Option<&'desktop DesktopView>,
}

impl Drop for DesktopIcon<'_> {
    fn drop(&mut self) {
        if !self.mut_ref.is_some() {
            unsafe {
                CoTaskMemFree(Some(self.inner.as_ptr() as _));
            }
        }
    }
}

impl<'desktop> DesktopIcon<'desktop> {
    /// SAFETY: `itemid` must points to a valid ITEMIDLIST allocated by COM Interface.
    pub(crate) unsafe fn from_com(
        view: &'desktop DesktopView,
        itemid: NonNull<ITEMIDLIST>,
    ) -> Self {
        Self {
            inner: itemid,
            mut_ref: None,
            _view: Some(view),
        }
    }
}

impl<'mem> DesktopIcon<'mem> {
    /// SAFETY: `bytes` must contains exactly a valid ITEMIDLIST.
    pub(crate) unsafe fn from_rust(bytes: &'mem mut [u8]) -> Self {
        unsafe {
            let pointer = NonNull::new_unchecked(bytes as *mut [u8] as *mut u8 as _);
            Self {
                inner: pointer,
                mut_ref: Some(bytes),
                _view: None,
            }
        }
    }
}

impl DesktopIcon<'_> {
    /// Get variable-length ITEMIDLIST
    ///
    /// See [libfwsi](https://github.com/libyal/libfwsi/blob/2f2aba25b888f37314a39d8c3e71c0e3ced56e59/documentation/Windows%20Shell%20Item%20format.asciidoc#2-shell-item-list)
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            let list_size = ILGetSize(Some(self.inner.as_ptr()));
            std::slice::from_raw_parts_mut(self.inner.as_ptr().cast(), list_size as _)
        }
    }
}
