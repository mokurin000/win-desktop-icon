use std::{marker::PhantomData, ptr::NonNull};

use windows::Win32::{System::Com::CoTaskMemFree, UI::Shell::Common::ITEMIDLIST};

pub struct DesktopIcon<'desktop, 'itemid> {
    pub(crate) inner: NonNull<ITEMIDLIST>,
    _mark1: PhantomData<&'desktop ()>,
    _mark2: PhantomData<&'itemid ()>,
}

impl Drop for DesktopIcon<'_, '_> {
    fn drop(&mut self) {
        unsafe {
            CoTaskMemFree(Some(self.inner.as_ptr() as _));
        }
    }
}

impl DesktopIcon<'_, '_> {
    /// SAFETY: `itemid` must points to a valid ITEMIDLIST.
    pub(crate) unsafe fn new(itemid: NonNull<ITEMIDLIST>) -> Self {
        Self {
            inner: itemid,
            _mark1: Default::default(),
            _mark2: Default::default(),
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
