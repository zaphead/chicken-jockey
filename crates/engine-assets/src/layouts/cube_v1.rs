use super::PixelRect;

pub const ALBEDO_WIDTH: u32 = 64;
pub const ALBEDO_HEIGHT: u32 = 32;
pub const FACE_SIZE: u32 = 16;

pub fn face_region(face: CubeFace) -> PixelRect {
    match face {
        CubeFace::Top => PixelRect {
            x: 16,
            y: 0,
            w: FACE_SIZE,
            h: FACE_SIZE,
        },
        CubeFace::Bottom => PixelRect {
            x: 32,
            y: 0,
            w: FACE_SIZE,
            h: FACE_SIZE,
        },
        CubeFace::Left => PixelRect {
            x: 0,
            y: 16,
            w: FACE_SIZE,
            h: FACE_SIZE,
        },
        CubeFace::Front => PixelRect {
            x: 16,
            y: 16,
            w: FACE_SIZE,
            h: FACE_SIZE,
        },
        CubeFace::Right => PixelRect {
            x: 32,
            y: 16,
            w: FACE_SIZE,
            h: FACE_SIZE,
        },
        CubeFace::Back => PixelRect {
            x: 48,
            y: 16,
            w: FACE_SIZE,
            h: FACE_SIZE,
        },
    }
}

/// World-space face normal (Z-up) → cross-net face slot.
pub fn face_from_normal(normal: [f32; 3]) -> CubeFace {
    if normal[2] > 0.5 {
        CubeFace::Top
    } else if normal[2] < -0.5 {
        CubeFace::Bottom
    } else if normal[0] > 0.5 {
        CubeFace::Right
    } else if normal[0] < -0.5 {
        CubeFace::Left
    } else if normal[1] > 0.5 {
        CubeFace::Front
    } else {
        CubeFace::Back
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CubeFace {
    Top,
    Bottom,
    Left,
    Front,
    Right,
    Back,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regions_fit_inside_albedo() {
        for face in [
            CubeFace::Top,
            CubeFace::Bottom,
            CubeFace::Left,
            CubeFace::Front,
            CubeFace::Right,
            CubeFace::Back,
        ] {
            let r = face_region(face);
            assert!(r.x + r.w <= ALBEDO_WIDTH);
            assert!(r.y + r.h <= ALBEDO_HEIGHT);
            assert_eq!(r.w, FACE_SIZE);
            assert_eq!(r.h, FACE_SIZE);
        }
    }
}
