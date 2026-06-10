use std::sync::Arc;
use std::thread;

use crossbeam_channel::{Receiver, Sender, unbounded};

use crate::mesh::SolidMesh;
use crate::pipeline::{GpuMesh, RenderPipeline};
use crate::world_mesh::RenderScene;

pub struct RenderSubmitWork {
    pub color_view: wgpu::TextureView,
    pub depth_view: wgpu::TextureView,
    pub scene: RenderScene,
    pub meshes: Vec<SolidMesh>,
    pub done: Sender<()>,
}

/// macOS-safe render worker: main thread owns the surface texture; worker encodes and submits.
pub struct RenderSubmitThread {
    tx: Sender<RenderSubmitWork>,
}

impl RenderSubmitThread {
    pub fn spawn(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        pipeline: RenderPipeline,
        depth_pipeline: wgpu::RenderPipeline,
    ) -> Self {
        let (tx, rx) = unbounded::<RenderSubmitWork>();
        thread::Builder::new()
            .name("render-submit".into())
            .spawn(move || render_worker_loop(device, queue, pipeline, depth_pipeline, rx))
            .expect("spawn render-submit thread");
        Self { tx }
    }

    pub fn submit(&self, work: RenderSubmitWork) {
        let _ = self.tx.send(work);
    }
}

fn render_worker_loop(
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    pipeline: RenderPipeline,
    depth_pipeline: wgpu::RenderPipeline,
    rx: Receiver<RenderSubmitWork>,
) {
    while let Ok(work) = rx.recv() {
        let gpu_meshes: Vec<GpuMesh> = work
            .meshes
            .iter()
            .map(|mesh| GpuMesh::from_mesh(&device, mesh))
            .collect();

        pipeline.update_camera(&queue, work.scene.camera.view_projection());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("render_submit_encoder"),
        });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("depth_prepass"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &work.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            pass.set_pipeline(&depth_pipeline);
            pass.set_bind_group(0, &pipeline.camera_bind_group, &[]);
            for mesh in &gpu_meshes {
                pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..mesh.index_count, 0, 0..1);
            }
        }

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("opaque_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &work.color_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.53,
                            g: 0.81,
                            b: 0.98,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &work.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            pass.set_pipeline(&pipeline.pipeline);
            pass.set_bind_group(0, &pipeline.camera_bind_group, &[]);
            for mesh in &gpu_meshes {
                pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..mesh.index_count, 0, 0..1);
            }
        }

        queue.submit(Some(encoder.finish()));
        let _ = work.done.send(());
    }
}
