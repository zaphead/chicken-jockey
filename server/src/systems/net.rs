use engine_core::SystemContext;
use engine_net::{BlockDelta, ClientPacket, EntitySnapshot, NetServer, PlayerInput, ServerPacket};
use engine_world::BlockChanged;
use game::{GameplayInput, NetPlayerId, PlayerInputs, Transform, spawn_net_player};

pub struct ServerNet(pub NetServer);

pub fn server_net_system(ctx: &mut SystemContext<'_>) {
    let Some(net) = ctx.resources.get::<ServerNet>() else {
        return;
    };

    let packets = net.0.drain_inbound();
    for (client_id, packet) in packets {
        match packet {
            ClientPacket::Join => {
                spawn_net_player(ctx, client_id);
            }
            ClientPacket::Input(input) => {
                if let Some(inputs) = ctx.resources.get_mut::<PlayerInputs>() {
                    inputs.set(client_id, gameplay_from_packet(input));
                }
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
        .drain::<BlockChanged>()
        .into_iter()
        .map(|change| BlockDelta {
            x: change.position.0.x,
            y: change.position.0.y,
            z: change.position.0.z,
            block: change.new_block,
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
        jump: input.jump,
        interact: input.interact,
        break_block: input.break_block,
        place_block: input.place_block,
    }
}
