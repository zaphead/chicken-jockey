use engine_core::App;
use engine_input::InputState;
use engine_net::{BlockDelta, ClientPacket, EntitySnapshot, NetClient, PlayerInput, ServerPacket};
use engine_world::{BlockPos, WorldMutationQueue};
use game::{LocalPlayer, NetPlayerId, Transform};
use glam::Vec3;

pub fn client_net_pre_update(app: &mut App, net: &NetClient, input: &InputState) {
    if net.player_id().is_none() {
        net.send(ClientPacket::Join);
    } else {
        net.send(ClientPacket::Input(input_to_packet(input)));
    }

    for packet in net.drain_inbound() {
        match packet {
            ServerPacket::Welcome { player_id } => {
                if let Some(local) = app.resource_mut::<LocalPlayer>() {
                    local.id = Some(player_id);
                }
            }
            ServerPacket::BlockDeltas(deltas) => apply_block_deltas(app, deltas),
            ServerPacket::EntitySnapshots(snapshots) => reconcile_snapshots(app, snapshots),
        }
    }
}

fn apply_block_deltas(app: &mut App, deltas: Vec<BlockDelta>) {
    let Some(queue) = app.resource_mut::<WorldMutationQueue>() else {
        return;
    };
    for delta in deltas {
        queue.set_block(
            BlockPos::new(delta.x, delta.y, delta.z),
            delta.block,
        );
    }
}

fn reconcile_snapshots(app: &mut App, snapshots: Vec<EntitySnapshot>) {
    let local_id = app
        .resource::<LocalPlayer>()
        .and_then(|local| local.id);

    let Some(local_id) = local_id else {
        return;
    };

    for snapshot in snapshots {
        if snapshot.player_id != local_id {
            continue;
        }

        for (entity, (id,)) in app.world.query::<(&NetPlayerId,)>().iter() {
            if id.0 != snapshot.player_id {
                continue;
            }
            if let Ok(mut transform) = app.world.get::<&mut Transform>(entity) {
                let target = Vec3::from_array(snapshot.position);
                if transform.position.distance(target) > 2.0 {
                    transform.position = target;
                } else {
                    transform.position = transform.position.lerp(target, 0.25);
                }
                transform.yaw = snapshot.yaw;
                transform.pitch = snapshot.pitch;
            }
        }
    }
}

fn input_to_packet(input: &InputState) -> PlayerInput {
    PlayerInput {
        move_axis: input.move_axis,
        look_delta: input.look_delta,
        jump: input.jump,
        interact: input.interact,
        break_block: input.break_block,
        place_block: input.place_block,
    }
}
