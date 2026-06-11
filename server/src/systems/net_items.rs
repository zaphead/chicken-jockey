use engine_core::SystemContext;
use engine_net::{InventorySync, ServerPacket, WorldItemSnapshot};
use game::{
    inventory_to_wire, InventoryDirty, NetPlayerId, PlayerInventory, WorldItemBook,
};

use crate::systems::net::ServerNet;

pub fn server_net_items_broadcast_system(ctx: &mut SystemContext<'_>) {
    let world_items = ctx.resources.get::<WorldItemBook>().and_then(|book| {
        if !book.dirty {
            return None;
        }
        Some(
            book.entries
                .values()
                .map(|entry| WorldItemSnapshot {
                    id: entry.id.0,
                    position: entry.position.to_array(),
                    stack: game::stack_to_wire(entry.stack),
                })
                .collect::<Vec<_>>(),
        )
    });

    if world_items.is_some() {
        if let Some(book) = ctx.resources.get_mut::<WorldItemBook>() {
            book.dirty = false;
        }
    }

    let inventory_syncs: Vec<InventorySync> = ctx
        .world
        .query::<(&NetPlayerId, &InventoryDirty)>()
        .iter()
        .filter_map(|(entity, (id, _))| {
            let inventory = ctx.world.get::<&PlayerInventory>(entity).ok()?;
            Some(InventorySync {
                player_id: id.0,
                slots: inventory_to_wire(&inventory),
                selected: inventory.selected_hotbar,
            })
        })
        .collect();

    let dirty_entities: Vec<_> = ctx
        .world
        .query::<&InventoryDirty>()
        .iter()
        .map(|(entity, _)| entity)
        .collect();

    let Some(net) = ctx.resources.get::<ServerNet>() else {
        return;
    };

    if let Some(snapshots) = world_items {
        let packet = ServerPacket::WorldItems(snapshots);
        for client_id in net.0.client_ids() {
            net.0.send(client_id, packet.clone());
        }
    }

    for sync in inventory_syncs {
        net.0.send(sync.player_id, ServerPacket::InventorySync(sync));
    }

    for entity in dirty_entities {
        let _ = ctx.world.remove_one::<InventoryDirty>(entity);
    }
}
