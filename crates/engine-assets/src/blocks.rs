use std::collections::HashMap;
use std::fs;
use std::path::Path;

use engine_world::{BlockId, BlockState};
use serde::Deserialize;

use crate::layouts::UvLayoutId;
use crate::material::{DrawCategory, TintMode};
use crate::tools::ToolClass;

#[derive(Debug, Clone, Deserialize)]
pub struct OverlaySpec {
    pub faces: String,
    pub path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StateVariantSpec {
    pub state: BlockState,
    pub faces: String,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CtmSpec {
    pub faces: String,
    pub tiles_dir: String,
    #[serde(default = "default_ctm_tile_count")]
    pub tile_count: u8,
}

fn default_ctm_tile_count() -> u8 {
    16
}

#[derive(Debug, Clone, Deserialize)]
pub struct DropSpec {
    pub item: String,
    #[serde(default = "default_drop_count")]
    pub count: u16,
}

fn default_drop_count() -> u16 {
    1
}

#[derive(Debug, Clone, Deserialize)]
pub struct BlockDefinition {
    pub id: BlockId,
    pub name: String,
    pub solid: bool,
    #[serde(default)]
    pub layout: UvLayoutId,
    #[serde(default)]
    pub texture: Option<String>,
    #[serde(default)]
    pub draw: DrawCategory,
    #[serde(default)]
    pub tint: Option<TintModeToml>,
    #[serde(default)]
    pub overlays: Vec<OverlaySpec>,
    #[serde(default)]
    pub state_variants: Vec<StateVariantSpec>,
    #[serde(default)]
    pub ctm: Option<CtmSpec>,
    #[serde(default = "default_hardness")]
    pub hardness: f32,
    #[serde(default)]
    pub preferred_tool: Option<ToolClass>,
    #[serde(default)]
    pub requires_tool: bool,
    #[serde(default)]
    pub drops: Vec<DropSpec>,
    #[serde(default)]
    pub sound_group: Option<String>,
}

fn default_hardness() -> f32 {
    1.0
}

#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TintModeToml {
    #[default]
    None,
    BiomeGrass,
    BiomeFoliage,
}

impl BlockDefinition {
    pub fn material_path(&self) -> String {
        self.texture
            .clone()
            .unwrap_or_else(|| format!("blocks/{}", self.name))
    }

    pub fn tint_mode(&self) -> TintMode {
        match self.tint.unwrap_or_default() {
            TintModeToml::None => TintMode::None,
            TintModeToml::BiomeGrass => TintMode::BiomeGrass,
            TintModeToml::BiomeFoliage => TintMode::BiomeFoliage,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct BlockRegistry {
    by_id: HashMap<BlockId, BlockDefinition>,
    by_name: HashMap<String, BlockId>,
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

    pub fn hardness(&self, id: BlockId) -> f32 {
        self.get(id).map(|block| block.hardness).unwrap_or(1.0)
    }

    pub fn preferred_tool(&self, id: BlockId) -> Option<ToolClass> {
        self.get(id).and_then(|block| block.preferred_tool)
    }

    pub fn requires_tool(&self, id: BlockId) -> bool {
        self.get(id).is_some_and(|block| block.requires_tool)
    }

    pub fn is_breakable(&self, id: BlockId) -> bool {
        self.hardness(id) >= 0.0
    }

    pub fn definitions(&self) -> impl Iterator<Item = &BlockDefinition> {
        self.by_id.values()
    }

    pub fn drops(&self, id: BlockId) -> &[DropSpec] {
        self.get(id)
            .map(|block| block.drops.as_slice())
            .unwrap_or(&[])
    }

    pub fn sound_group(&self, id: BlockId) -> &str {
        let Some(block) = self.get(id) else {
            return "stone";
        };
        block
            .sound_group
            .as_deref()
            .unwrap_or(block.name.as_str())
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
