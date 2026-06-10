use engine_core::SystemContext;
use glam::Vec3;

use crate::components::{Collider, NetPlayerId, Player, Transform, Velocity, WorldInitialized};
use crate::simulation::{LocalPlayer, SimulationMode};
use crate::systems::terrain::spawn_height;

pub fn spawn_player_system(ctx: &mut SystemContext<'_>) {
    let initialized = ctx
        .resources
        .get::<WorldInitialized>()
        .map(|flag| flag.0)
        .unwrap_or(false);
    if !initialized {
        return;
    }

    let mode = ctx
        .resources
        .get::<SimulationMode>()
        .copied()
        .unwrap_or(SimulationMode::Local);

    if matches!(mode, SimulationMode::AuthoritativeServer) {
        return;
    }

    if matches!(mode, SimulationMode::NetworkClient) {
        let Some(local) = ctx.resources.get::<LocalPlayer>() else {
            return;
        };
        if local.spawned || local.id.is_none() {
            return;
        }
        if let Some(id) = local.id {
            spawn_net_player(ctx, id);
            if let Some(local) = ctx.resources.get_mut::<LocalPlayer>() {
                local.spawned = true;
            }
        }
        return;
    }

    if ctx.world.query::<&Player>().iter().next().is_some() {
        return;
    }

    spawn_net_player(ctx, 0);
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
