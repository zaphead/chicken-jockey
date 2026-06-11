use std::sync::Arc;

use engine_assets::{GuiTextures, ResolvedBlockMaterials, ToolRegistry};
use engine_core::SystemContext;
use engine_render::{
    build_item_drop_meshes, Camera, DroppedItemRender, ItemDropBuildContext, ItemDropMeshes,
};
use engine_world::BiomeMap;
use game::DroppedItem;

pub fn build_render_item_drops(
    ctx: &SystemContext<'_>,
    camera: &Camera,
    animation_tick: u32,
) -> (ItemDropMeshes, u64) {
    let spin = animation_tick as f32 * 0.0175;
    let dropped_items: Vec<DroppedItemRender> = ctx
        .world
        .query::<(&DroppedItem, &game::Transform)>()
        .iter()
        .map(|(_, (item, transform))| DroppedItemRender {
            position: transform.position,
            spin,
            kind: item.stack.kind,
            count: item.stack.count,
        })
        .collect();

    let generation = if dropped_items.is_empty() {
        0
    } else {
        animation_tick as u64
    };

    let Some(materials) = ctx.resources.get::<Arc<ResolvedBlockMaterials>>() else {
        return (Default::default(), 0);
    };
    let tools = ctx.resources.get::<ToolRegistry>();
    let textures = ctx.resources.get::<Arc<GuiTextures>>();
    let biome = ctx
        .resources
        .get::<BiomeMap>()
        .cloned()
        .unwrap_or_default();

    let item_drops = match (tools, textures) {
        (Some(tools), Some(textures)) => build_item_drop_meshes(
            &dropped_items,
            &ItemDropBuildContext {
                materials: &materials,
                biome: &biome,
                camera,
                tools,
                item_icons: &textures.item_icons,
            },
        ),
        _ => Default::default(),
    };

    (item_drops, generation)
}
