use engine_core::App;
use engine_input::InputState;
use engine_net::{BlockDelta, ClientPacket, EntitySnapshot, NetServer, PlayerInput, ServerPacket};
use engine_world::BlockChanged;
use game::{spawn_net_player, NetPlayerId, RemoteInputs, Transform};

pub fn server_net_pre_update(app: &mut App, net: &NetServer) {
    for (client_id, packet) in net.drain_inbound() {
        match packet {
            ClientPacket::Join => {
                app.system_context(|ctx| spawn_net_player(ctx, client_id));
            }
            ClientPacket::Input(input) => {
                if let Some(remote) = app.resource_mut::<RemoteInputs>() {
                    remote.set(client_id, input_from_packet(input));
                }
            }
        }
    }
}

pub fn server_net_post_update(app: &mut App, net: &NetServer) {
    let block_deltas: Vec<BlockDelta> = app
        .drain_events::<BlockChanged>()
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
        for client_id in net.client_ids() {
            net.send(client_id, packet.clone());
        }
    }

    let snapshots = collect_entity_snapshots(app);
    if !snapshots.is_empty() {
        let packet = ServerPacket::EntitySnapshots(snapshots);
        for client_id in net.client_ids() {
            net.send(client_id, packet.clone());
        }
    }

    if let Some(remote) = app.resource_mut::<RemoteInputs>() {
        remote.clear_frame();
    }
}

fn collect_entity_snapshots(app: &App) -> Vec<EntitySnapshot> {
    app.world
        .query::<(&NetPlayerId, &Transform)>()
        .iter()
        .map(|(_, (id, transform))| EntitySnapshot {
            player_id: id.0,
            position: transform.position.to_array(),
            yaw: transform.yaw,
            pitch: transform.pitch,
        })
        .collect()
}

fn input_from_packet(input: PlayerInput) -> InputState {
    InputState {
        move_axis: input.move_axis,
        look_delta: input.look_delta,
        jump: input.jump,
        interact: input.interact,
        break_block: input.break_block,
        place_block: input.place_block,
        cursor_locked: true,
    }
}
