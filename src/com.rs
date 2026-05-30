use crate::error::Result;
use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED};

pub struct ComApartment;

impl ComApartment {
    pub fn init() -> Result<Self> {
        unsafe {
            CoInitializeEx(
                None,
                // Required for UI/Shell COM api
                COINIT_APARTMENTTHREADED,
            )
            .ok()?;
        }
        Ok(Self)
    }
}

impl Drop for ComApartment {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}
