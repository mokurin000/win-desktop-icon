use std::{marker::PhantomData, ptr::NonNull};

use windows::Win32::{System::Com::CoTaskMemFree, UI::Shell::Common::ITEMIDLIST};

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

    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            let cb: u16 = self.inner.cast().read();
            let start = self.inner.as_ptr() as *mut u8;
            std::slice::from_raw_parts_mut(start, cb as usize)
        }
    }
}
