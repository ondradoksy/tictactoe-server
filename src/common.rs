use serde::{ Serialize, Deserialize };

#[derive(Serialize, Deserialize, PartialEq)]
pub(crate) struct Size {
    pub x: u32,
    pub y: u32,
}

impl Size {
    pub fn new(x: u32, y: u32) -> Self {
        Self {
            x: x,
            y: y,
        }
    }
}
