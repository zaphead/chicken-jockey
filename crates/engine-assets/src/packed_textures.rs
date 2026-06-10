use crate::atlas::TextureAtlas;
use crate::material_map::BlockMaterialMap;

/// GPU atlas and per-block face UVs produced together by [`crate::pack_block_textures`].
#[derive(Debug, Clone)]
pub struct PackedBlockTextures {
    pub atlas: TextureAtlas,
    pub materials: BlockMaterialMap,
}
