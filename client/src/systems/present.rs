use engine_assets::ToolRegistry;
use engine_core::SystemContext;
use engine_render::{extract_render_scene, Renderer, RenderWorld};
use game::{
    local_player_entity, tool_label_for_inventory, ActiveDebugWorld, ActivePlayMode, PlayerInventory,
    Velocity,
};
use glam::Vec3;

use crate::systems::hud::format_debug_hud;

pub struct ClientRenderer(pub Renderer);

pub fn present_frame_system(ctx: &mut SystemContext<'_>) {
    let play_mode = ctx.resources.get::<ActivePlayMode>().map(|mode| mode.0);
    let debug_world = ctx.resources.get::<ActiveDebugWorld>().map(|active| active.0);
    let local_player = local_player_entity(ctx);
    let velocity = local_player
        .and_then(|entity| ctx.world.get::<&Velocity>(entity).ok())
        .map(|velocity| velocity.0)
        .unwrap_or(Vec3::ZERO);
    let tool_label = local_player
        .and_then(|entity| ctx.world.get::<&PlayerInventory>(entity).ok())
        .and_then(|inventory| {
            ctx.resources
                .get::<ToolRegistry>()
                .map(|tools| (inventory, tools))
        })
        .map(|(inventory, tools)| tool_label_for_inventory(&inventory, tools))
        .unwrap_or_else(|| "hand".to_string());

    let presented = ctx
        .resources
        .with_pair::<RenderWorld, ClientRenderer, _>(|world, renderer| {
            if !world.ready {
                return false;
            }
            if world.opaque.vertices.is_empty() && world.cutout.vertices.is_empty() {
                log::debug!("present skipped: zero meshes in RenderWorld");
                return false;
            }

            let hud_text = format_debug_hud(
                &world.camera,
                play_mode,
                debug_world,
                velocity,
                &tool_label,
                world.lighting.world_time,
            );
            renderer.0.sync_meshes(
                world.mesh_generation,
                &world.opaque,
                &world.cutout,
            );
            let scene = extract_render_scene(
                world.camera,
                Default::default(),
                Default::default(),
                world.animation_tick,
                Vec::new(),
                world.target_block,
                world.mining_overlay.clone(),
                world.particles.clone(),
                world.lighting,
            );
            let gui = if world.gui.needs_gui_pass() {
                Some(&world.gui)
            } else {
                None
            };
            let gui_scale = world.gui_scale.max(0.25);
            if let Err(error) = renderer.0.render(&scene, Some(&hud_text), gui_scale, gui) {
                log::warn!("render error: {error:?}");
            }
            true
        })
        .unwrap_or(false);
    if !presented {
        return;
    }
}
