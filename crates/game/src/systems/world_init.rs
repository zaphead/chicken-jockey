use engine_core::SystemContext;
use glam::Vec3;

use crate::components::{Collider, NetPlayerId, Player, Transform, Velocity, WorldInitialized};
use crate::input::LocalPlayerId;
use crate::mode::NetworkClient;
use crate::systems::terrain::spawn_height;

pub fn spawn_local_player_system(ctx: &mut SystemContext<'_>) {
    if ctx.resources.get::<NetworkClient>().is_some() {
        return;
    }
    spawn_when_ready(ctx, || 0);
}

pub fn spawn_network_player_system(ctx: &mut SystemContext<'_>) {
    if ctx.resources.get::<NetworkClient>().is_none() {
        return;
    }
    let Some(local) = ctx.resources.get::<LocalPlayerId>() else {
        return;
    };
    if local.spawned || local.id.is_none() {
        return;
    }
    let id = local.id.expect("checked above");
    spawn_when_ready(ctx, || id);
    if let Some(local) = ctx.resources.get_mut::<LocalPlayerId>() {
        local.spawned = true;
    }
}

fn spawn_when_ready(ctx: &mut SystemContext<'_>, player_id: impl FnOnce() -> u32) {
    let initialized = ctx
        .resources
        .get::<WorldInitialized>()
        .map(|flag| flag.0)
        .unwrap_or(false);
    if !initialized {
        return;
    }
    if ctx.world.query::<&Player>().iter().next().is_some() {
        return;
    }
    spawn_net_player(ctx, player_id());
}

pub fn spawn_net_player(ctx: &mut SystemContext<'_>, player_id: u32) {
    if ctx
        .world
        .query::<&NetPlayerId>()
        .iter()
        .any(|(_, id)| id.0 == player_id)
    {
        return;
    }

    let offset = (player_id as f32 * 4.0) % 32.0;
    let x = offset as i32;
    let y = spawn_height(x, 0) as f32;
    ctx.world.spawn((
        Player,
        NetPlayerId(player_id),
        Transform {
            position: Vec3::new(offset + 0.5, y, 0.5),
            yaw: 0.0,
            pitch: 0.0,
        },
        Velocity::default(),
        Collider {
            half_extents: Vec3::new(0.35, 0.9, 0.35),
        },
    ));
}
