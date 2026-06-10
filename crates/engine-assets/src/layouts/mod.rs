mod cube_v1;

use serde::Deserialize;

pub use cube_v1::{
    face_from_normal, face_region, CubeFace, ALBEDO_HEIGHT, ALBEDO_WIDTH, FACE_SIZE,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
pub enum UvLayoutId {
    #[default]
    #[serde(rename = "cube_v1")]
    CubeV1,
}

#[derive(Debug, Clone, Copy)]
pub struct PixelRect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}
