use std::marker::PhantomData;
use std::ptr::NonNull;

use windows::Win32::System::Com::CoTaskMemFree;
use windows::Win32::UI::Shell::Common::ITEMIDLIST;
use windows::Win32::UI::Shell::ILGetSize;

pub struct DesktopIcon<'desktop, 'itemid> {
    pub(crate) inner: NonNull<ITEMIDLIST>,
    _mark1: PhantomData<&'desktop ()>,
    _mark2: PhantomData<&'itemid ()>,
    no_free: bool,
}

impl Drop for DesktopIcon<'_, '_> {
    fn drop(&mut self) {
        if !self.no_free {
            unsafe {
                CoTaskMemFree(Some(self.inner.as_ptr() as _));
            }
        }
    }
}

impl DesktopIcon<'_, 'static> {
    /// SAFETY: `itemid` must points to a valid ITEMIDLIST.
    pub(crate) unsafe fn from_com(itemid: NonNull<ITEMIDLIST>) -> Self {
        Self {
            inner: itemid,
            _mark1: Default::default(),
            _mark2: Default::default(),
            no_free: false,
        }
    }
}

impl DesktopIcon<'_, '_> {
    /// SAFETY: `itemid` must points to a valid ITEMIDLIST.
    pub(crate) unsafe fn from_rust(itemid: NonNull<ITEMIDLIST>) -> Self {
        Self {
            inner: itemid,
            _mark1: Default::default(),
            _mark2: Default::default(),
            no_free: true,
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
