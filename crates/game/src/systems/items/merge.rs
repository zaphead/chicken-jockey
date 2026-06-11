use crate::inventory::stacks_fit_together;
use engine_core::SystemContext;

use crate::components::{DroppedItem, Transform, WorldItemId};
use crate::world_items::WorldItemBook;

/// ~¼-scale item cubes touch around 0.25m center spacing; merge only when stacked.
const MERGE_DISTANCE: f32 = 0.18;

pub fn item_merge_system(ctx: &mut SystemContext<'_>) {
    let items: Vec<(hecs::Entity, WorldItemId, Transform, DroppedItem)> = ctx
        .world
        .query::<(&DroppedItem, &WorldItemId, &Transform)>()
        .iter()
        .map(|(entity, (item, id, transform))| (entity, *id, *transform, *item))
        .collect();

    let mut despawn: Vec<(hecs::Entity, WorldItemId)> = Vec::new();

    for i in 0..items.len() {
        let (entity_a, id_a, transform_a, item_a) = items[i];
        if despawn.iter().any(|(e, _)| *e == entity_a) {
            continue;
        }
        for j in (i + 1)..items.len() {
            let (entity_b, id_b, transform_b, item_b) = items[j];
            if despawn.iter().any(|(e, _)| *e == entity_b) {
                continue;
            }
            if !stacks_fit_together(&item_a.stack, &item_b.stack) {
                continue;
            }
            if transform_a.position.distance(transform_b.position) > MERGE_DISTANCE {
                continue;
            }

            let merged = item_a.stack.count.saturating_add(item_b.stack.count);

            let merged_pos = (transform_a.position + transform_b.position) * 0.5;
            if let Ok(mut keeper) = ctx.world.get::<&mut DroppedItem>(entity_a) {
                keeper.stack.count = merged;
            }
            if let Ok(mut transform) = ctx.world.get::<&mut Transform>(entity_a) {
                transform.position = merged_pos;
            }
            if let Some(book) = ctx.resources.get_mut::<WorldItemBook>() {
                book.update_position(id_a, merged_pos);
                if let Some(entry) = book.entries.get_mut(&id_a.0) {
                    entry.stack.count = merged;
                    book.dirty = true;
                }
                book.remove(id_b);
            }
            despawn.push((entity_b, id_b));
        }
    }

    for (entity, _) in despawn {
        ctx.commands.push(move |world| {
            let _ = world.despawn(entity);
        });
    }
}
