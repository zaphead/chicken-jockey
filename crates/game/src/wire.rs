use engine_assets::{ItemKind, ItemStack};
use engine_net::{DropAmountWire, ItemStackWire, INVENTORY_SLOT_COUNT};
use engine_world::BlockState;

use crate::components::PlayerInventory;
use crate::inventory::DropAmount;

pub fn stack_to_wire(stack: ItemStack) -> ItemStackWire {
    match stack.kind {
        ItemKind::Block { id, state } => ItemStackWire::Block {
            id,
            state: state.0,
            count: stack.count,
        },
        ItemKind::Tool(id) => ItemStackWire::Tool { id },
    }
}

pub fn stack_from_wire(wire: ItemStackWire) -> ItemStack {
    match wire {
        ItemStackWire::Block { id, state, count } => ItemStack {
            kind: ItemKind::Block {
                id,
                state: BlockState(state),
            },
            count,
        },
        ItemStackWire::Tool { id } => ItemStack::tool(id),
    }
}

pub fn inventory_to_wire(inventory: &PlayerInventory) -> Vec<Option<ItemStackWire>> {
    inventory
        .slots
        .iter()
        .map(|slot| slot.map(stack_to_wire))
        .collect()
}

pub fn inventory_from_wire(slots: Vec<Option<ItemStackWire>>, selected: u8) -> PlayerInventory {
    let mut out = [None; INVENTORY_SLOT_COUNT];
    for (index, slot) in slots.into_iter().enumerate().take(INVENTORY_SLOT_COUNT) {
        out[index] = slot.map(stack_from_wire);
    }
    PlayerInventory {
        slots: out,
        selected_hotbar: selected.min(8),
    }
}

pub fn drop_amount_to_wire(amount: DropAmount) -> DropAmountWire {
    match amount {
        DropAmount::One => DropAmountWire::One,
        DropAmount::Half => DropAmountWire::Half,
        DropAmount::All => DropAmountWire::All,
    }
}

pub fn drop_amount_from_wire(amount: DropAmountWire) -> DropAmount {
    match amount {
        DropAmountWire::One => DropAmount::One,
        DropAmountWire::Half => DropAmount::Half,
        DropAmountWire::All => DropAmount::All,
    }
}
