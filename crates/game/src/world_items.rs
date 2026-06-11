use std::collections::BTreeMap;

use glam::Vec3;

use crate::components::WorldItemId;

#[derive(Debug, Clone)]
pub struct WorldItemEntry {
    pub id: WorldItemId,
    pub position: Vec3,
    pub stack: engine_assets::ItemStack,
}

#[derive(Debug, Default)]
pub struct WorldItemBook {
    pub next_id: u32,
    pub entries: BTreeMap<u32, WorldItemEntry>,
    pub dirty: bool,
}

impl WorldItemBook {
    pub fn allocate_id(&mut self) -> WorldItemId {
        let id = WorldItemId(self.next_id);
        self.next_id += 1;
        id
    }

    pub fn insert(&mut self, entry: WorldItemEntry) {
        self.entries.insert(entry.id.0, entry);
        self.dirty = true;
    }

    pub fn remove(&mut self, id: WorldItemId) {
        if self.entries.remove(&id.0).is_some() {
            self.dirty = true;
        }
    }

    pub fn update_position(&mut self, id: WorldItemId, position: Vec3) {
        if let Some(entry) = self.entries.get_mut(&id.0) {
            entry.position = position;
            self.dirty = true;
        }
    }

    pub fn update_stack(&mut self, id: WorldItemId, stack: engine_assets::ItemStack) {
        if let Some(entry) = self.entries.get_mut(&id.0) {
            entry.stack = stack;
            self.dirty = true;
        }
    }
}
