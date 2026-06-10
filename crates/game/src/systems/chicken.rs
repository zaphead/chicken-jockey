use std::collections::HashSet;

use engine_assets::BlockRegistry;
use engine_core::{SystemContext, Time};
use engine_world::SparseVoxelOctree;
use glam::Vec3;
use hecs::Entity;
use rand::Rng;

use crate::components::{
    Chicken, Collider, Mountable, Renderable, Rider, Transform, Velocity, WorldInitialized,
    WorldSeed,
};
use crate::debug_world::DebugWorldKind;
use crate::systems::terrain::{player_ground_center_z_at, WORLD_RADIUS};

const CHICKEN_COUNT: i32 = 12;

pub fn chicken_spawn_system(ctx: &mut SystemContext<'_>) {
    let initialized = ctx
        .resources
        .get::<WorldInitialized>()
        .map(|flag| flag.0)
        .unwrap_or(false);
    if !initialized {
        return;
    }

    if ctx.world.query::<&Chicken>().iter().next().is_some() {
        return;
    }

    let seed = ctx
        .resources
        .get::<WorldSeed>()
        .map(|seed| seed.0)
        .unwrap_or(0);
    let mut rng = rand::thread_rng();
    for _ in 0..CHICKEN_COUNT {
        let x = rng.gen_range((-WORLD_RADIUS + 8)..(WORLD_RADIUS - 8));
        let y = rng.gen_range((-WORLD_RADIUS + 8)..(WORLD_RADIUS - 8));
        let center_z = player_ground_center_z_at(x, y, DebugWorldKind::Flat, seed) - 0.45;

        ctx.world.spawn((
            Chicken {
                wander_timer: rng.gen_range(1.0..4.0),
                wander_direction: Vec3::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0), 0.0)
                    .normalize_or_zero(),
                speed: 2.5,
            },
            Mountable,
            Transform {
                position: Vec3::new(x as f32 + 0.5, y as f32 + 0.5, center_z),
                yaw: rng.gen_range(0.0..std::f32::consts::TAU),
                pitch: 0.0,
            },
            Velocity::default(),
            Collider {
                half_extents: Vec3::new(0.35, 0.45, 0.35),
            },
            Renderable {
                color: [0.95, 0.85, 0.2],
                size: 0.7,
            },
        ));
    }
}

pub fn chicken_wander_system(ctx: &mut SystemContext<'_>) {
    let delta = ctx
        .resources
        .get::<Time>()
        .map(|time| time.fixed_delta)
        .unwrap_or(0.0);
    let registry = ctx.resources.get::<BlockRegistry>();
    let world = ctx.resources.get::<SparseVoxelOctree>();

    let (Some(registry), Some(world)) = (registry, world) else {
        return;
    };

    let ridden: HashSet<Entity> = ctx
        .world
        .query::<&Rider>()
        .iter()
        .map(|(entity, _)| entity)
        .collect();

    let mut rng = rand::thread_rng();
    let entities: Vec<Entity> = ctx
        .world
        .query::<&Chicken>()
        .iter()
        .map(|(entity, _)| entity)
        .filter(|entity| !ridden.contains(entity))
        .collect();

    for entity in entities {
        let Ok(mut chicken) = ctx.world.get::<&mut Chicken>(entity) else {
            continue;
        };
        let Ok(mut transform) = ctx.world.get::<&mut Transform>(entity) else {
            continue;
        };
        let Ok(mut velocity_ref) = ctx.world.get::<&mut Velocity>(entity) else {
            continue;
        };

        chicken.wander_timer -= delta;
        if chicken.wander_timer <= 0.0 {
            chicken.wander_timer = rng.gen_range(1.5..4.0);
            chicken.wander_direction =
                Vec3::new(rng.gen_range(-1.0..1.0), 0.0, rng.gen_range(-1.0..1.0))
                    .normalize_or_zero();
            transform.yaw = chicken.wander_direction.z.atan2(chicken.wander_direction.x);
        }

        let wish = chicken.wander_direction * chicken.speed;
        let mut velocity = Vec3::new(wish.x, velocity_ref.0.y - 12.0 * delta, wish.z);
        let mut position = transform.position;

        for axis in 0..3 {
            let delta_axis = velocity[axis] * delta;
            if delta_axis == 0.0 {
                continue;
            }
            position[axis] += delta_axis;
            if crate::systems::physics::collision::collides_aabb(
                world,
                registry,
                position,
                Vec3::new(0.35, 0.45, 0.35),
            ) {
                position[axis] -= delta_axis;
                velocity[axis] = 0.0;
                chicken.wander_direction =
                    Vec3::new(-chicken.wander_direction.x, 0.0, -chicken.wander_direction.z);
            }
        }

        transform.position = position;
        velocity_ref.0 = velocity;
    }
}

