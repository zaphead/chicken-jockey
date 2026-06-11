use engine_core::SystemContext;
use engine_net::{BlockDelta, ClientPacket, EntitySnapshot, InventoryAction, NetServer, PlayerInput, ServerPacket};
use engine_world::VoxelChanged;
use game::{
    GameplayInput, InventoryCommand, InventoryCommandQueue, NetPlayerId, PlayerInputs, Transform,
    drop_amount_from_wire, spawn_net_player,
};

pub struct ServerNet(pub NetServer);

pub fn server_net_system(ctx: &mut SystemContext<'_>) {
    let Some(net) = ctx.resources.get::<ServerNet>() else {
        return;
    };

    let packets = net.0.drain_inbound();
    for (client_id, packet) in packets {
        match packet {
            ClientPacket::Join => {
                spawn_net_player(ctx, client_id, None);
            }
            ClientPacket::Input(input) => {
                if let Some(inputs) = ctx.resources.get_mut::<PlayerInputs>() {
                    inputs.set(client_id, gameplay_from_packet(input));
                }
            }
            ClientPacket::InventoryActions(actions) => {
                enqueue_inventory_actions(ctx, client_id, actions);
            }
        }
    }
}

fn enqueue_inventory_actions(ctx: &mut SystemContext<'_>, client_id: u32, actions: Vec<InventoryAction>) {
    let Some(player_entity) = ctx
        .world
        .query::<(&NetPlayerId,)>()
        .iter()
        .find_map(|(entity, (id,))| (id.0 == client_id).then_some(entity))
    else {
        return;
    };
    let Some(queue) = ctx.resources.get_mut::<InventoryCommandQueue>() else {
        return;
    };
    for action in actions {
        match action {
            InventoryAction::MoveSlot { from, to } => {
                queue.push(InventoryCommand::MoveSlot {
                    player: player_entity,
                    from,
                    to,
                });
            }
            InventoryAction::QuickMove { slot } => {
                queue.push(InventoryCommand::QuickMove {
                    player: player_entity,
                    slot,
                });
            }
            InventoryAction::SwapWithCarried { slot, carried } => {
                queue.push(InventoryCommand::SwapCarried {
                    player: player_entity,
                    slot,
                    carried: carried.map(game::stack_from_wire),
                });
            }
        }
    }
}

pub fn server_net_broadcast_system(ctx: &mut SystemContext<'_>) {
    let Some(net) = ctx.resources.get::<ServerNet>() else {
        return;
    };

    let block_deltas: Vec<BlockDelta> = ctx
        .events
        .drain::<VoxelChanged>()
        .into_iter()
        .map(|change| BlockDelta {
            x: change.position.0.x,
            y: change.position.0.y,
            z: change.position.0.z,
            block: change.new_cell.id,
            state: change.new_cell.state.0,
        })
        .collect();

    if !block_deltas.is_empty() {
        let packet = ServerPacket::BlockDeltas(block_deltas);
        for client_id in net.0.client_ids() {
            net.0.send(client_id, packet.clone());
        }
    }

    let snapshots: Vec<EntitySnapshot> = ctx
        .world
        .query::<(&NetPlayerId, &Transform)>()
        .iter()
        .map(|(_, (id, transform))| EntitySnapshot {
            player_id: id.0,
            position: transform.position.to_array(),
            yaw: transform.yaw,
            pitch: transform.pitch,
        })
        .collect();

    if !snapshots.is_empty() {
        let packet = ServerPacket::EntitySnapshots(snapshots);
        for client_id in net.0.client_ids() {
            net.0.send(client_id, packet.clone());
        }
    }

    if let Some(inputs) = ctx.resources.get_mut::<PlayerInputs>() {
        inputs.clear_frame();
    }
}

fn gameplay_from_packet(input: PlayerInput) -> GameplayInput {
    GameplayInput {
        move_axis: input.move_axis,
        look_delta: input.look_delta,
        vertical_axis: 0.0,
        sprint: false,
        jump: input.jump,
        interact: input.interact,
        break_block: input.break_block,
        place_block: input.place_block,
        tool_slot: input.tool_slot,
        drop_hotbar: input.drop_hotbar.map(drop_amount_from_wire),
    }
}
