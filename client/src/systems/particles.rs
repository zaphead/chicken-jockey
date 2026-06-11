use engine_core::{SystemContext, Time};
use engine_render::{ParticleSystem, RenderWorld};
use engine_world::SparseVoxelOctree;

pub fn particle_extract_system(ctx: &mut SystemContext<'_>) {
    let dt = ctx
        .resources
        .get::<Time>()
        .map(|time| time.frame_delta)
        .unwrap_or(1.0 / 60.0);

    let camera = ctx
        .resources
        .get::<RenderWorld>()
        .map(|world| world.camera)
        .unwrap_or_default();

    let mesh = ctx
        .resources
        .with_pair::<SparseVoxelOctree, ParticleSystem, _>(|world, particles| {
            particles.tick(dt, world);
            particles.build_mesh(&camera)
        });

    let Some(mesh) = mesh else {
        return;
    };

    if let Some(render_world) = ctx.resources.get_mut::<RenderWorld>() {
        render_world.particles = mesh;
    }
}
