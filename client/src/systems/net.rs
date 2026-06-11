use engine_core::SystemContext;
use engine_net::{BlockDelta, ClientPacket, EntitySnapshot, NetClient, PlayerInput, ServerPacket};
use engine_world::{BlockPos, BlockState, VoxelCell, WorldMutationQueue};
use game::{drop_amount_to_wire, LocalPlayerId, NetPlayerId, Transform, Velocity};
use glam::Vec3;

use crate::systems::input::PendingWinitInput;
use crate::systems::net_items::{apply_inventory_sync, apply_world_items};
use crate::systems::pending_inventory::PendingInventoryActions;

pub struct ClientNet(pub NetClient);

pub fn client_net_system(ctx: &mut SystemContext<'_>) {
    let Some(pending) = ctx.resources.get::<PendingWinitInput>() else {
        return;
    };

    let input_packet = input_to_packet(ctx, &pending.0);
    let inventory_actions = ctx
        .resources
        .get_mut::<PendingInventoryActions>()
        .map(|actions| actions.drain())
        .unwrap_or_default();

    let Some(net) = ctx.resources.get::<ClientNet>() else {
        return;
    };

    if net.0.player_id().is_none() {
        net.0.send(ClientPacket::Join);
    } else {
        net.0.send(ClientPacket::Input(input_packet));
        if !inventory_actions.is_empty() {
            net.0.send(ClientPacket::InventoryActions(inventory_actions));
        }
    }

    for packet in net.0.drain_inbound() {
        match packet {
            ServerPacket::Welcome { player_id } => {
                if let Some(local) = ctx.resources.get_mut::<LocalPlayerId>() {
                    local.id = Some(player_id);
                }
            }
            ServerPacket::BlockDeltas(deltas) => apply_block_deltas(ctx, deltas),
            ServerPacket::EntitySnapshots(snapshots) => reconcile_snapshots(ctx, snapshots),
            ServerPacket::WorldItems(items) => apply_world_items(ctx, items),
            ServerPacket::InventorySync(sync) => apply_inventory_sync(ctx, sync),
        }
    }
}

fn apply_block_deltas(ctx: &mut SystemContext<'_>, deltas: Vec<BlockDelta>) {
    let Some(queue) = ctx.resources.get_mut::<WorldMutationQueue>() else {
        return;
    };
    for delta in deltas {
        queue.set_voxel(
            BlockPos::new(delta.x, delta.y, delta.z),
            VoxelCell {
                id: delta.block,
                state: BlockState(delta.state),
            },
        );
    }
}

fn reconcile_snapshots(ctx: &mut SystemContext<'_>, snapshots: Vec<EntitySnapshot>) {
    let local_id = ctx
        .resources
        .get::<LocalPlayerId>()
        .and_then(|local| local.id);

    let Some(local_id) = local_id else {
        return;
    };

    for snapshot in snapshots {
        if snapshot.player_id != local_id {
            continue;
        }

        for (entity, (id,)) in ctx.world.query::<(&NetPlayerId,)>().iter() {
            if id.0 != snapshot.player_id {
                continue;
            }
            if let Ok(mut transform) = ctx.world.get::<&mut Transform>(entity) {
                let target = Vec3::from_array(snapshot.position);
                let error = transform.position - target;
                if error.length() > 2.0 {
                    transform.position = target;
                    if let Ok(mut velocity) = ctx.world.get::<&mut Velocity>(entity) {
                        velocity.0 = Vec3::ZERO;
                    }
                } else if error.length() > 0.05 {
                    transform.position = transform.position.lerp(target, 0.35);
                    if let Ok(mut velocity) = ctx.world.get::<&mut Velocity>(entity) {
                        velocity.0 -= error * 0.75;
                    }
                }
                transform.yaw = snapshot.yaw;
                transform.pitch = snapshot.pitch;
            }
        }
    }
}

fn input_to_packet(ctx: &SystemContext<'_>, input: &engine_input::InputState) -> PlayerInput {
    let drop_hotbar = ctx
        .resources
        .get::<game::PlayerInputs>()
        .and_then(|inputs| {
            ctx.resources
                .get::<LocalPlayerId>()
                .and_then(|local| local.id)
                .and_then(|id| inputs.get(id))
        })
        .and_then(|gameplay| gameplay.drop_hotbar)
        .map(drop_amount_to_wire);

    PlayerInput {
        move_axis: input.move_axis,
        look_delta: input.look_delta,
        jump: input.jump,
        interact: input.interact,
        break_block: input.break_held,
        place_block: input.place_held,
        tool_slot: input.selected_tool_slot,
        drop_hotbar,
    }
}
