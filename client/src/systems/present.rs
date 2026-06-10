use engine_core::SystemContext;
use engine_render::{extract_render_scene, RenderWorld, Renderer};

pub struct ClientRenderer(pub Renderer);

pub fn present_frame_system(ctx: &mut SystemContext<'_>) {
    let snapshot = ctx.resources.get::<RenderWorld>().and_then(|world| {
        if world.ready {
            Some((world.camera, world.meshes.clone()))
        } else {
            None
        }
    });
    let Some((camera, meshes)) = snapshot else {
        return;
    };
    let Some(renderer) = ctx.resources.get_mut::<ClientRenderer>() else {
        log::debug!("present skipped: renderer not ready");
        return;
    };

    if meshes.is_empty() {
        log::debug!("present skipped: zero meshes in RenderWorld");
        return;
    }

    let scene = extract_render_scene(camera, meshes, Vec::new());
    renderer.0.upload_meshes(&scene.chunk_meshes);
    if let Err(error) = renderer.0.render(&scene) {
        log::warn!("render error: {error:?}");
    }
}
