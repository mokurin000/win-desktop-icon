use std::{marker::PhantomData, ptr::NonNull};

use windows::Win32::{System::Com::CoTaskMemFree, UI::Shell::Common::ITEMIDLIST};

pub struct DesktopIcon<'a> {
    pub(crate) inner: NonNull<ITEMIDLIST>,
    _mark: PhantomData<&'a ()>,
}

impl Drop for DesktopIcon<'_> {
    fn drop(&mut self) {
        unsafe {
            CoTaskMemFree(Some(self.inner.as_ptr() as _));
        }
    }
}

impl DesktopIcon<'_> {
    /// SAFETY: `itemid` must points to a valid ITEMIDLIST.
    pub(crate) unsafe fn new(itemid: NonNull<ITEMIDLIST>) -> Self {
        Self {
            inner: itemid,
            _mark: Default::default(),
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
