use engine_assets::{
    max_stack, stacks_merge, BlockRegistry, ItemKind, ItemStack, ToolRegistry,
};
use engine_core::SystemContext;
use engine_world::BlockId;
use hecs::Entity;

use crate::components::{InventoryDirty, PlayerInventory, HOTBAR_SLOTS, INVENTORY_SLOTS};

pub const MINED_PICKUP_DELAY_TICKS: u8 = 10;
pub const PLAYER_DROP_PICKUP_DELAY_TICKS: u8 = 40;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropAmount {
    One,
    Half,
    All,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertResult {
    Complete,
    Partial { remainder: ItemStack },
}

#[derive(Debug, Clone)]
pub enum InventoryCommand {
    Insert {
        player: Entity,
        stack: ItemStack,
        world_item: Option<Entity>,
    },
    MoveSlot {
        player: Entity,
        from: u8,
        to: u8,
    },
    QuickMove {
        player: Entity,
        slot: u8,
    },
    Drop {
        player: Entity,
        slot: u8,
        amount: DropAmount,
    },
    SwapCarried {
        player: Entity,
        slot: u8,
        carried: Option<ItemStack>,
    },
}

#[derive(Debug, Default)]
pub struct InventoryCommandQueue {
    pub commands: Vec<InventoryCommand>,
}

impl InventoryCommandQueue {
    pub fn push(&mut self, command: InventoryCommand) {
        self.commands.push(command);
    }

    pub fn drain(&mut self) -> Vec<InventoryCommand> {
        std::mem::take(&mut self.commands)
    }
}

pub fn resolve_block_drops(
    blocks: &BlockRegistry,
    tools: &ToolRegistry,
    block_id: BlockId,
    harvested: bool,
) -> Vec<ItemStack> {
    if !harvested {
        return Vec::new();
    }

    let drops = blocks.drops(block_id);
    if drops.is_empty() {
        return vec![ItemStack::block(block_id, 1)];
    }

    drops
        .iter()
        .filter_map(|drop| resolve_named_item(blocks, tools, &drop.item, drop.count))
        .collect()
}

fn resolve_named_item(
    blocks: &BlockRegistry,
    tools: &ToolRegistry,
    name: &str,
    count: u16,
) -> Option<ItemStack> {
    if let Some(id) = blocks.id_by_name(name) {
        return Some(ItemStack::block(id, count.max(1)));
    }
    tools
        .id_by_name(name)
        .map(|id| ItemStack::tool(id))
}

pub fn can_merge_stacks(a: ItemKind, b: ItemKind) -> bool {
    stacks_merge(a, b)
}

pub fn stacks_fit_together(a: &ItemStack, b: &ItemStack) -> bool {
    if !stacks_merge(a.kind, b.kind) {
        return false;
    }
    a.count.saturating_add(b.count) <= max_stack(a.kind)
}

pub fn mark_inventory_dirty(ctx: &mut SystemContext<'_>, player: Entity) {
    ctx.commands.push(move |world| {
        let _ = world.insert_one(player, InventoryDirty);
    });
}

pub fn try_insert(inventory: &mut PlayerInventory, mut stack: ItemStack) -> InsertResult {
    stack.count = stack.count.min(max_stack(stack.kind));

    let selected = inventory.selected_hotbar as usize;
    stack = merge_into_slot(inventory, selected, stack);
    if stack.count == 0 {
        return InsertResult::Complete;
    }

    for index in 0..HOTBAR_SLOTS {
        if index == selected {
            continue;
        }
        stack = merge_into_slot(inventory, index, stack);
        if stack.count == 0 {
            return InsertResult::Complete;
        }
    }

    for index in HOTBAR_SLOTS..INVENTORY_SLOTS {
        stack = merge_into_slot(inventory, index, stack);
        if stack.count == 0 {
            return InsertResult::Complete;
        }
    }

    if let Some(index) = first_empty_hotbar(inventory) {
        inventory.set_slot(index, Some(stack));
        return InsertResult::Complete;
    }

    if let Some(index) = first_empty_main(inventory) {
        inventory.set_slot(index, Some(stack));
        return InsertResult::Complete;
    }

    if stack.count > 0 {
        InsertResult::Partial { remainder: stack }
    } else {
        InsertResult::Complete
    }
}

fn merge_into_slot(inventory: &mut PlayerInventory, index: usize, mut stack: ItemStack) -> ItemStack {
    let Some(existing) = inventory.slots[index] else {
        return stack;
    };
    if !stacks_merge(existing.kind, stack.kind) {
        return stack;
    }
    let cap = max_stack(stack.kind);
    let room = cap.saturating_sub(existing.count);
    if room == 0 {
        return stack;
    }
    let moved = stack.count.min(room);
    inventory.slots[index] = Some(ItemStack {
        kind: stack.kind,
        count: existing.count + moved,
    });
    stack.count -= moved;
    stack
}

fn first_empty_hotbar(inventory: &PlayerInventory) -> Option<usize> {
    (0..HOTBAR_SLOTS).find(|&index| inventory.slots[index].is_none())
}

fn first_empty_main(inventory: &PlayerInventory) -> Option<usize> {
    (HOTBAR_SLOTS..INVENTORY_SLOTS).find(|&index| inventory.slots[index].is_none())
}

pub fn swap_slots(inventory: &mut PlayerInventory, from: u8, to: u8) {
    let from = from as usize;
    let to = to as usize;
    if from >= INVENTORY_SLOTS || to >= INVENTORY_SLOTS {
        return;
    }
    inventory.slots.swap(from, to);
}

pub fn quick_move(inventory: &mut PlayerInventory, slot: u8) -> bool {
    let slot = slot as usize;
    if slot >= INVENTORY_SLOTS {
        return false;
    }
    let Some(stack) = inventory.slots[slot] else {
        return false;
    };

    let target_range = if slot < HOTBAR_SLOTS {
        HOTBAR_SLOTS..INVENTORY_SLOTS
    } else {
        0..HOTBAR_SLOTS
    };

    let mut remaining = stack;
    for index in target_range.clone() {
        remaining = merge_into_slot(inventory, index, remaining);
        if remaining.count == 0 {
            inventory.set_slot(slot, None);
            return true;
        }
    }

    for index in target_range {
        if inventory.slots[index].is_none() {
            inventory.set_slot(index, Some(remaining));
            inventory.set_slot(slot, None);
            return true;
        }
    }

    inventory.set_slot(slot, Some(remaining));
    false
}

pub fn consume_from_slot(
    inventory: &mut PlayerInventory,
    slot: usize,
    amount: u16,
) -> bool {
    if amount == 0 {
        return true;
    }
    let Some(stack) = inventory.slots[slot] else {
        return false;
    };
    if stack.count < amount {
        return false;
    }
    let remaining = stack.count - amount;
    inventory.set_slot(
        slot,
        if remaining == 0 {
            None
        } else {
            Some(ItemStack {
                kind: stack.kind,
                count: remaining,
            })
        },
    );
    true
}

pub fn drop_from_slot(
    inventory: &mut PlayerInventory,
    slot: u8,
    amount: DropAmount,
) -> Option<ItemStack> {
    let slot = slot as usize;
    if slot >= INVENTORY_SLOTS {
        return None;
    }
    let stack = inventory.slots[slot]?;
    let drop_count = match amount {
        DropAmount::One => 1,
        DropAmount::All => stack.count,
        DropAmount::Half => (stack.count + 1) / 2,
    }
    .min(stack.count);
    let (dropped, remainder) = stack.split(drop_count);
    inventory.set_slot(slot, remainder);
    Some(dropped)
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_assets::{load_block_registry, load_tool_registry, ItemKind};
    use std::path::PathBuf;

    fn manifest_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    fn registries() -> (BlockRegistry, ToolRegistry) {
        let assets = manifest_dir().join("../../assets");
        (
            load_block_registry(&assets.join("blocks")),
            load_tool_registry(&assets.join("tools")),
        )
    }

    #[test]
    fn try_insert_prefers_selected_hotbar_partial_stack() {
        let (blocks, _) = registries();
        let dirt = blocks.id_by_name("dirt").unwrap();
        let mut inventory = PlayerInventory::default();
        inventory.selected_hotbar = 0;
        inventory.set_slot(0, Some(ItemStack::block(dirt, 50)));

        let result = try_insert(
            &mut inventory,
            ItemStack::block(dirt, 10),
        );
        assert_eq!(result, InsertResult::Complete);
        assert_eq!(inventory.slots[0].unwrap().count, 60);
    }

    #[test]
    fn resolve_block_drops_empty_when_not_harvested() {
        let (blocks, tools) = registries();
        let stone = blocks.id_by_name("stone").unwrap();
        assert!(resolve_block_drops(&blocks, &tools, stone, false).is_empty());
    }

    #[test]
    fn resolve_block_drops_self_when_harvested() {
        let (blocks, tools) = registries();
        let stone = blocks.id_by_name("stone").unwrap();
        let drops = resolve_block_drops(&blocks, &tools, stone, true);
        assert_eq!(drops.len(), 1);
        assert_eq!(drops[0].count, 1);
        assert!(matches!(drops[0].kind, ItemKind::Block { id, .. } if id == stone));
    }

    #[test]
    fn consume_from_slot_clears_last_block() {
        let (blocks, _) = registries();
        let dirt = blocks.id_by_name("dirt").unwrap();
        let mut inventory = PlayerInventory::default();
        inventory.set_slot(0, Some(ItemStack::block(dirt, 1)));
        assert!(consume_from_slot(&mut inventory, 0, 1));
        assert!(inventory.slots[0].is_none());
    }

    #[test]
    fn consume_from_slot_fails_when_empty() {
        let mut inventory = PlayerInventory::default();
        assert!(!consume_from_slot(&mut inventory, 0, 1));
    }

    #[test]
    fn drop_half_rounds_up() {
        let (blocks, _) = registries();
        let dirt = blocks.id_by_name("dirt").unwrap();
        let mut inventory = PlayerInventory::default();
        inventory.set_slot(0, Some(ItemStack::block(dirt, 7)));
        let dropped = drop_from_slot(&mut inventory, 0, DropAmount::Half).unwrap();
        assert_eq!(dropped.count, 4);
        assert_eq!(inventory.slots[0].unwrap().count, 3);
    }
}
