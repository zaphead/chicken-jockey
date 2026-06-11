use crate::lighting::LightingResources;
use crate::pipeline::GpuMesh;
use crate::pipeline::RenderPipelines;
use crate::post::PostPipeline;
use crate::sky::SkyPipeline;

pub fn record_sky_pass(
    encoder: &mut wgpu::CommandEncoder,
    hdr_view: &wgpu::TextureView,
    depth_view: &wgpu::TextureView,
    sky: &SkyPipeline,
    env_bind_group: &wgpu::BindGroup,
) {
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("sky_pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: hdr_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: depth_view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }),
        occlusion_query_set: None,
        timestamp_writes: None,
    });
    sky.draw(&mut pass, env_bind_group);
}

pub fn record_shadow_pass(
    encoder: &mut wgpu::CommandEncoder,
    lighting: &LightingResources,
    pipelines: &RenderPipelines,
    opaque_meshes: &[GpuMesh],
    cutout_meshes: &[GpuMesh],
) {
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("shadow_pass"),
        color_attachments: &[],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: &lighting.shadow_view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }),
        occlusion_query_set: None,
        timestamp_writes: None,
    });
    pass.set_pipeline(&pipelines.shadow);
    pass.set_bind_group(0, &pipelines.scene_bind_group, &[]);
    pass.set_bind_group(1, &pipelines.atlas_bind_group, &[]);
    pass.set_bind_group(2, &lighting.uniform_bind_group, &[]);
    draw_meshes(&mut pass, opaque_meshes);
    draw_meshes(&mut pass, cutout_meshes);
}

pub fn record_depth_pass(
    encoder: &mut wgpu::CommandEncoder,
    depth_view: &wgpu::TextureView,
    lighting: &LightingResources,
    pipelines: &RenderPipelines,
    opaque_meshes: &[GpuMesh],
    cutout_meshes: &[GpuMesh],
) {
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("depth_pass"),
        color_attachments: &[],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: depth_view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }),
        occlusion_query_set: None,
        timestamp_writes: None,
    });
    pass.set_pipeline(&pipelines.depth);
    pass.set_bind_group(0, &pipelines.scene_bind_group, &[]);
    pass.set_bind_group(1, &pipelines.atlas_bind_group, &[]);
    pass.set_bind_group(2, &lighting.uniform_bind_group, &[]);
    pass.set_bind_group(3, &lighting.shadow_bind_group, &[]);
    draw_meshes(&mut pass, opaque_meshes);
    draw_meshes(&mut pass, cutout_meshes);
}

pub fn record_opaque_pass(
    encoder: &mut wgpu::CommandEncoder,
    hdr_view: &wgpu::TextureView,
    depth_view: &wgpu::TextureView,
    lighting: &LightingResources,
    pipelines: &RenderPipelines,
    opaque_meshes: &[GpuMesh],
) {
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("opaque_pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: hdr_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: depth_view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }),
        occlusion_query_set: None,
        timestamp_writes: None,
    });
    pass.set_pipeline(&pipelines.opaque);
    pass.set_bind_group(0, &pipelines.scene_bind_group, &[]);
    pass.set_bind_group(1, &pipelines.atlas_bind_group, &[]);
    pass.set_bind_group(2, &lighting.uniform_bind_group, &[]);
    pass.set_bind_group(3, &lighting.shadow_bind_group, &[]);
    draw_meshes(&mut pass, opaque_meshes);
}

pub fn record_cutout_pass(
    encoder: &mut wgpu::CommandEncoder,
    hdr_view: &wgpu::TextureView,
    depth_view: &wgpu::TextureView,
    lighting: &LightingResources,
    pipelines: &RenderPipelines,
    cutout_meshes: &[GpuMesh],
) {
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("cutout_pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: hdr_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: depth_view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }),
        occlusion_query_set: None,
        timestamp_writes: None,
    });
    pass.set_pipeline(&pipelines.cutout);
    pass.set_bind_group(0, &pipelines.scene_bind_group, &[]);
    pass.set_bind_group(1, &pipelines.atlas_bind_group, &[]);
    pass.set_bind_group(2, &lighting.uniform_bind_group, &[]);
    pass.set_bind_group(3, &lighting.shadow_bind_group, &[]);
    draw_meshes(&mut pass, cutout_meshes);
}

pub fn record_particle_pass<'a>(
    encoder: &mut wgpu::CommandEncoder,
    hdr_view: &wgpu::TextureView,
    depth_view: &wgpu::TextureView,
    lighting: &LightingResources,
    pipelines: &'a RenderPipelines,
) {
    if pipelines.particles.index_count == 0 {
        return;
    }
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("particle_pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: hdr_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: depth_view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }),
        occlusion_query_set: None,
        timestamp_writes: None,
    });
    pipelines.particles.draw_hdr(
        &mut pass,
        &pipelines.scene_bind_group,
        &pipelines.atlas_bind_group,
        &lighting.uniform_bind_group,
    );
}

pub fn record_post_pass(
    encoder: &mut wgpu::CommandEncoder,
    swapchain_view: &wgpu::TextureView,
    post: &PostPipeline,
) {
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("post_pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: swapchain_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        occlusion_query_set: None,
        timestamp_writes: None,
    });
    post.draw(&mut pass);
}

fn draw_meshes(pass: &mut wgpu::RenderPass<'_>, meshes: &[GpuMesh]) {
    for mesh in meshes {
        pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        pass.draw_indexed(0..mesh.index_count, 0, 0..1);
    }
}
