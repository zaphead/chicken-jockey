use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::Deserialize;

pub type ToolId = u16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ToolClass {
    #[default]
    Pickaxe,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ToolDefinition {
    pub id: ToolId,
    pub name: String,
    pub tool_class: ToolClass,
    pub efficiency: f32,
}

#[derive(Debug, Default, Clone)]
pub struct ToolRegistry {
    by_id: HashMap<ToolId, ToolDefinition>,
    by_name: HashMap<String, ToolId>,
}

impl ToolRegistry {
    pub fn insert(&mut self, definition: ToolDefinition) {
        self.by_name.insert(definition.name.clone(), definition.id);
        self.by_id.insert(definition.id, definition);
    }

    pub fn get(&self, id: ToolId) -> Option<&ToolDefinition> {
        self.by_id.get(&id)
    }

    pub fn id_by_name(&self, name: &str) -> Option<ToolId> {
        self.by_name.get(name).copied()
    }
}

pub fn load_tool_registry(tools_dir: &Path) -> ToolRegistry {
    let mut registry = ToolRegistry::default();
    if !tools_dir.exists() {
        return registry;
    }

    for entry in fs::read_dir(tools_dir).expect("read tools directory") {
        let entry = entry.expect("read tools entry");
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("toml") {
            continue;
        }
        let contents = fs::read_to_string(&path).expect("read tool file");
        let definition: ToolDefinition = toml::from_str(&contents).expect("parse tool file");
        registry.insert(definition);
    }

    registry
}

pub fn tools_asset_path(manifest_dir: &str) -> std::path::PathBuf {
    super::server::assets_dir(manifest_dir).join("tools")
}
