use bitcode::{Decode, Encode};
use windows::Win32::Foundation::POINT;

#[derive(Encode, Decode)]
pub struct DeskopIconState {
    pub pidl: Vec<u8>,
    pub position_x: i32,
    pub position_y: i32,
}

impl DeskopIconState {
    pub fn point(&self) -> POINT {
        POINT {
            x: self.position_x,
            y: self.position_y,
        }
    }
}
