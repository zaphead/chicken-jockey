use engine_core::SystemContext;
use glam::Vec3;

use crate::axes::PLAYER_HALF_EXTENTS;
use crate::components::{
    Collider, LocomotionState, NetPlayerId, Player, Transform, Velocity, WorldInitialized,
    WorldSeed,
};
use crate::input::LocalPlayerId;
use crate::mode::NetworkClient;
use crate::debug_world::ActiveDebugWorld;
use crate::play_mode::ActivePlayMode;
use crate::systems::terrain::{player_spawn_center_z_at, PLAYER_SPAWN_PITCH};

pub fn spawn_local_player_system(ctx: &mut SystemContext<'_>) {
    if ctx.resources.get::<NetworkClient>().is_some() {
        return;
    }
    if ctx
        .resources
        .get::<ActivePlayMode>()
        .is_some_and(|mode| !mode.allows_player_sim())
    {
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
    spawn_net_player(ctx, player_id(), None);
}

pub fn spawn_net_player(
    ctx: &mut SystemContext<'_>,
    player_id: u32,
    spawn: Option<(Vec3, f32, f32)>,
) {
    if ctx
        .world
        .query::<&NetPlayerId>()
        .iter()
        .any(|(_, id)| id.0 == player_id)
    {
        return;
    }

    let (position, yaw, pitch) = spawn.unwrap_or_else(|| {
        let world = ctx
            .resources
            .get::<ActiveDebugWorld>()
            .map(|active| active.0)
            .unwrap_or_default();
        let seed = ctx
            .resources
            .get::<WorldSeed>()
            .map(|seed| seed.0)
            .unwrap_or(0);
        let offset = (player_id as f32 * 4.0) % 32.0;
        let column_x = offset.floor() as i32;
        let column_y = 0;
        (
            Vec3::new(
                offset + 0.5,
                0.5,
                player_spawn_center_z_at(column_x, column_y, world, seed),
            ),
            0.0,
            PLAYER_SPAWN_PITCH,
        )
    });
    ctx.world.spawn((
        Player,
        NetPlayerId(player_id),
        Transform {
            position,
            yaw,
            pitch,
        },
        Velocity::default(),
        LocomotionState::default(),
        Collider {
            half_extents: PLAYER_HALF_EXTENTS,
        },
    ));

    if ctx.resources.get::<NetworkClient>().is_none() && player_id == 0 {
        if let Some(local) = ctx.resources.get_mut::<LocalPlayerId>() {
            local.id = Some(0);
            local.spawned = true;
        }
    }
}
