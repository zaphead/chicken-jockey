use engine_assets::{BlockRegistry, ToolRegistry};
use engine_core::SystemContext;
use engine_world::BlockPos;
use glam::Vec3;
use rand::Rng;

use crate::components::{DroppedItem, Transform, Velocity, WorldItemId};
use crate::events::BlockBroken;
use crate::inventory::{resolve_block_drops, MINED_PICKUP_DELAY_TICKS};
use crate::world_items::{WorldItemBook, WorldItemEntry};

pub fn spawn_drops_on_block_break(ctx: &mut SystemContext<'_>) {
    let Some(blocks) = ctx.resources.get::<BlockRegistry>().cloned() else {
        return;
    };
    let Some(tools) = ctx.resources.get::<ToolRegistry>().cloned() else {
        return;
    };

    let broken: Vec<BlockBroken> = ctx.events.drain();
    for event in broken {
        let drops = resolve_block_drops(&blocks, &tools, event.block_id, event.harvested);
        let center = block_center(event.position);
        for (index, stack) in drops.into_iter().enumerate() {
            let spread = Vec3::new(
                (index as f32 * 1.7).sin() * 0.12,
                (index as f32 * 2.3).cos() * 0.12,
                0.1,
            );
            spawn_drop_at(ctx, center + spread, stack, MINED_PICKUP_DELAY_TICKS);
        }
    }
}

pub fn spawn_drop_at(
    ctx: &mut SystemContext<'_>,
    position: Vec3,
    stack: engine_assets::ItemStack,
    pickup_delay_ticks: u8,
) {
    let velocity = random_drop_velocity();

    let world_item_id = ctx
        .resources
        .get_mut::<WorldItemBook>()
        .map(|book| book.allocate_id())
        .unwrap_or(WorldItemId(0));
    let id_val = world_item_id.0;

    ctx.commands.push(move |world| {
        let _ = world.spawn((
            DroppedItem {
                stack,
                pickup_delay_ticks,
            },
            WorldItemId(id_val),
            Transform {
                position,
                yaw: 0.0,
                pitch: 0.0,
            },
            Velocity(velocity),
        ));
    });

    if let Some(book) = ctx.resources.get_mut::<WorldItemBook>() {
        book.insert(WorldItemEntry {
            id: world_item_id,
            position,
            stack,
        });
    }
}

pub fn random_drop_velocity() -> Vec3 {
    let mut rng = rand::thread_rng();
    let angle = rng.gen_range(0.0f32..std::f32::consts::TAU);
    let horizontal = rng.gen_range(0.35..1.1);
    let upward = rng.gen_range(0.7..1.5);
    Vec3::new(angle.cos() * horizontal, angle.sin() * horizontal, upward)
}

fn block_center(pos: BlockPos) -> Vec3 {
    Vec3::new(
        pos.0.x as f32 + 0.5,
        pos.0.y as f32 + 0.5,
        pos.0.z as f32 + 0.5,
    )
}
