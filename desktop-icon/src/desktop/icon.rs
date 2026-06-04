use std::marker::PhantomData;
use std::ptr::NonNull;

use windows::Win32::System::Com::CoTaskMemFree;
use windows::Win32::UI::Shell::Common::ITEMIDLIST;
use windows::Win32::UI::Shell::ILGetSize;

pub struct DesktopIcon<'desktop> {
    pub(crate) inner: NonNull<ITEMIDLIST>,
    _mark: PhantomData<&'desktop ()>,
    rust_managed: bool,
}

impl Drop for DesktopIcon<'_> {
    fn drop(&mut self) {
        if !self.rust_managed {
            unsafe {
                CoTaskMemFree(Some(self.inner.as_ptr() as _));
            }
        }
    }
}

impl<'desktop> DesktopIcon<'desktop> {
    /// SAFETY: `itemid` must points to a valid ITEMIDLIST allocated by COM Interface.
    pub(crate) unsafe fn from_com(itemid: NonNull<ITEMIDLIST>) -> Self {
        Self {
            inner: itemid,
            _mark: Default::default(),
            rust_managed: false,
        }
    }

    /// SAFETY: `itemid` must points to a valid ITEMIDLIST.
    ///
    /// If `itemid` points was allocated by COM Interface, this will leak it.
    pub(crate) unsafe fn from_rust(itemid: NonNull<ITEMIDLIST>) -> Self {
        Self {
            inner: itemid,
            _mark: Default::default(),
            rust_managed: true,
        }
    }

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
