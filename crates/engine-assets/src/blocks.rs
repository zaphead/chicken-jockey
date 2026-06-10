use std::collections::HashMap;
use std::fs;
use std::path::Path;

use engine_world::BlockId;
use serde::Deserialize;

use crate::layouts::UvLayoutId;

#[derive(Debug, Clone, Deserialize)]
pub struct BlockDefinition {
    pub id: BlockId,
    pub name: String,
    pub solid: bool,
    pub opaque: bool,
    /// UV layout type (default `cube_v1`).
    #[serde(default)]
    pub layout: UvLayoutId,
    /// Texture folder under `assets/textures/` (default `blocks/{name}`).
    #[serde(default)]
    pub texture: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub struct BlockRegistry {
    by_id: HashMap<BlockId, BlockDefinition>,
    by_name: HashMap<String, BlockId>,
}

impl BlockDefinition {
    pub fn material_path(&self) -> String {
        self.texture
            .clone()
            .unwrap_or_else(|| format!("blocks/{}", self.name))
    }
}

impl BlockRegistry {
    pub fn insert(&mut self, definition: BlockDefinition) {
        self.by_name.insert(definition.name.clone(), definition.id);
        self.by_id.insert(definition.id, definition);
    }

    pub fn get(&self, id: BlockId) -> Option<&BlockDefinition> {
        self.by_id.get(&id)
    }

    pub fn id_by_name(&self, name: &str) -> Option<BlockId> {
        self.by_name.get(name).copied()
    }

    pub fn is_solid(&self, id: BlockId) -> bool {
        self.get(id).map(|block| block.solid).unwrap_or(false)
    }

    pub fn definitions(&self) -> impl Iterator<Item = &BlockDefinition> {
        self.by_id.values()
    }
}

pub fn load_block_registry(blocks_dir: &Path) -> BlockRegistry {
    let mut registry = BlockRegistry::default();
    if !blocks_dir.exists() {
        return registry;
    }

    for entry in fs::read_dir(blocks_dir).expect("read blocks directory") {
        let entry = entry.expect("read blocks entry");
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("toml") {
            continue;
        }
        let contents = fs::read_to_string(&path).expect("read block file");
        let definition: BlockDefinition = toml::from_str(&contents).expect("parse block file");
        registry.insert(definition);
    }

    registry
}
