use engine_core::SystemContext;
use glam::Vec3;
use hecs::Entity;

use crate::axes::{horizontal_forward, horizontal_right, UP};
use crate::components::{Collider, Mounted, Mountable, Player, Rider, Transform, Velocity};
use crate::input::{local_player_entity, resolve_input};
use crate::systems::physics::collision::collides_aabb;

const MOUNT_RANGE: f32 = 2.5;
const MOUNT_SPEED: f32 = 10.0;

pub fn mount_system(ctx: &mut SystemContext<'_>) {
    let Some(player_entity) = local_player_entity(ctx) else {
        return;
    };
    let net_id = ctx
        .world
        .get::<&crate::components::NetPlayerId>(player_entity)
        .ok()
        .map(|id| id.0);
    let Some(input) = resolve_input(ctx, net_id) else {
        return;
    };
    if !input.interact {
        return;
    }

    if ctx.world.get::<&Mounted>(player_entity).is_ok() {
        dismount_player(ctx, player_entity);
        return;
    }

    let player_pos = ctx
        .world
        .get::<&Transform>(player_entity)
        .map(|transform| transform.position)
        .unwrap_or(Vec3::ZERO);

    let mut closest: Option<(Entity, f32)> = None;
    for (entity, (_, transform)) in ctx.world.query::<(&Mountable, &Transform)>().iter() {
        if ctx.world.get::<&Rider>(entity).is_ok() {
            continue;
        }
        let distance = (transform.position - player_pos).length();
        if distance <= MOUNT_RANGE {
            closest = Some(match closest {
                Some((_, best)) if distance >= best => closest.unwrap(),
                _ => (entity, distance),
            });
        }
    }

    let Some((mount_entity, _)) = closest else {
        return;
    };

    ctx.commands.push(move |world| {
        world
            .insert_one(player_entity, Mounted { mount: mount_entity })
            .expect("insert mounted");
        world
            .insert_one(mount_entity, Rider {
                rider: player_entity,
            })
            .expect("insert rider");
    });
}

fn dismount_player(ctx: &mut SystemContext<'_>, player_entity: Entity) {
    let Some(mount_entity) = ctx
        .world
        .get::<&Mounted>(player_entity)
        .ok()
        .map(|mounted| mounted.mount)
    else {
        return;
    };

    let side_offset = ctx
        .world
        .get::<&Transform>(mount_entity)
        .map(|transform| {
            let right = horizontal_right(transform.yaw);
            transform.position + right * 0.5 + UP * 0.5
        })
        .unwrap_or(Vec3::ZERO);

    if let Ok(mut transform) = ctx.world.get::<&mut Transform>(player_entity) {
        transform.position = side_offset;
    }
    if let Ok(mut velocity) = ctx.world.get::<&mut Velocity>(player_entity) {
        velocity.0 = Vec3::ZERO;
    }

    ctx.commands.push(move |world| {
        let _ = world.remove_one::<Mounted>(player_entity);
        let _ = world.remove_one::<Rider>(mount_entity);
    });
}

pub fn mounted_movement_system(ctx: &mut SystemContext<'_>) {
    let pairs: Vec<(Entity, Entity, Option<u32>)> = ctx
        .world
        .query::<(&Player, &Mounted, Option<&crate::components::NetPlayerId>)>()
        .iter()
        .map(|(player_entity, (_, mounted, net_id))| {
            (player_entity, mounted.mount, net_id.map(|id| id.0))
        })
        .collect();

    for (player_entity, mount_entity, net_id) in pairs {
        let Some(input) = resolve_input(ctx, net_id) else {
            continue;
        };

        let Ok(mut mount_transform) = ctx.world.get::<&mut Transform>(mount_entity) else {
            continue;
        };
        let Ok(mut mount_velocity) = ctx.world.get::<&mut Velocity>(mount_entity) else {
            continue;
        };
        let Ok(mut player_transform) = ctx.world.get::<&mut Transform>(player_entity) else {
            continue;
        };

        mount_transform.yaw += input.look_delta.x * 0.002;
        let forward = horizontal_forward(mount_transform.yaw);
        let right = horizontal_right(mount_transform.yaw);
        let wish = (forward * input.move_axis.y + right * input.move_axis.x).normalize_or_zero();

        mount_velocity.0.x = wish.x * MOUNT_SPEED;
        mount_velocity.0.y = wish.y * MOUNT_SPEED;

        player_transform.position = mount_transform.position + UP * 0.9;
        player_transform.yaw = mount_transform.yaw;
        player_transform.pitch = -0.25;
    }
}

pub fn mounted_physics_system(ctx: &mut SystemContext<'_>) {
    let delta = ctx
        .resources
        .get::<engine_core::Time>()
        .map(|time| time.fixed_delta)
        .unwrap_or(0.0);

    let registry = ctx.resources.get::<engine_assets::BlockRegistry>();
    let world = ctx.resources.get::<engine_world::SparseVoxelOctree>();

    let mounts: Vec<(Entity, Vec3, Vec3, Vec3)> = ctx
        .world
        .query::<(&Player, &Mounted)>()
        .iter()
        .filter_map(|(_, (_, mounted))| {
            let mount_entity = mounted.mount;
            let transform = ctx.world.get::<&Transform>(mount_entity).ok()?;
            let velocity = ctx.world.get::<&Velocity>(mount_entity).ok()?;
            let collider = ctx.world.get::<&Collider>(mount_entity).ok()?;
            Some((
                mount_entity,
                transform.position,
                velocity.0,
                collider.half_extents,
            ))
        })
        .collect();

    for (mount_entity, start_position, mut velocity, half_extents) in mounts {
        velocity -= UP * 18.0 * delta;
        let mut position = start_position;
        for axis in 0..3 {
            let delta_axis = velocity[axis] * delta;
            if delta_axis == 0.0 {
                continue;
            }
            position[axis] += delta_axis;
            let blocked = match (world, registry) {
                (Some(world), Some(registry)) => {
                    collides_aabb(world, registry, position, half_extents)
                }
                _ => false,
            };
            if blocked {
                position[axis] -= delta_axis;
                velocity[axis] = 0.0;
            }
        }

        if let Ok(mut transform) = ctx.world.get::<&mut Transform>(mount_entity) {
            transform.position = position;
        }
        if let Ok(mut velocity_ref) = ctx.world.get::<&mut Velocity>(mount_entity) {
            velocity_ref.0 = velocity;
        }
    }
}
