use std::collections::HashMap;

use engine_world::BlockId;

use crate::atlas::UvRect;
use crate::layouts::{face_from_normal, CubeFace};

#[derive(Debug, Clone)]
pub struct BlockFaceUvs {
    faces: [UvRect; 6],
}

impl Default for BlockFaceUvs {
    fn default() -> Self {
        Self {
            faces: [UvRect::BLACK; 6],
        }
    }
}

impl BlockFaceUvs {
    pub fn set(&mut self, face: CubeFace, uv: UvRect) {
        self.faces[face_index(face)] = uv;
    }

    pub fn get(&self, face: CubeFace) -> UvRect {
        self.faces[face_index(face)]
    }

    pub fn from_normal(&self, normal: [f32; 3]) -> UvRect {
        self.get(face_from_normal(normal))
    }
}

#[derive(Debug, Clone)]
pub struct BlockMaterialMap {
    by_block: HashMap<BlockId, BlockFaceUvs>,
    fallback: UvRect,
}

impl BlockMaterialMap {
    pub fn new(fallback: UvRect) -> Self {
        Self {
            by_block: HashMap::new(),
            fallback,
        }
    }

    pub fn insert(&mut self, block_id: BlockId, faces: BlockFaceUvs) {
        self.by_block.insert(block_id, faces);
    }

    pub fn face_uv(&self, block_id: BlockId, normal: [f32; 3]) -> UvRect {
        self.by_block
            .get(&block_id)
            .map(|faces| faces.from_normal(normal))
            .unwrap_or(self.fallback)
    }

    pub fn fallback(&self) -> UvRect {
        self.fallback
    }
}

fn face_index(face: CubeFace) -> usize {
    match face {
        CubeFace::Top => 0,
        CubeFace::Bottom => 1,
        CubeFace::Left => 2,
        CubeFace::Front => 3,
        CubeFace::Right => 4,
        CubeFace::Back => 5,
    }
}
