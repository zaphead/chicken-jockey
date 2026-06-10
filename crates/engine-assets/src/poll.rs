use engine_core::SystemContext;

use crate::{AssetServer, BlockRegistry};

/// Polls async asset loads and promotes the block registry into ECS resources when ready.
pub fn poll_assets_system(ctx: &mut SystemContext<'_>) {
    if ctx.resources.get::<BlockRegistry>().is_some() {
        return;
    }
    let registry = {
        let Some(server) = ctx.resources.get_mut::<AssetServer>() else {
            return;
        };
        server.poll();
        server.blocks()
    };
    if let Some(registry) = registry {
        ctx.resources.insert(registry);
    }
}
