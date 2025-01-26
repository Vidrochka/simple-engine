use std::sync::Arc;

use mint::Vector4;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use wgpu::{CommandEncoder, RenderPass, RenderPipeline, SurfaceTexture, TextureView};
use xdi::{types::error::ServiceBuildResult, ServiceProvider};

use super::render_state::RenderStateInner;




#[derive(Debug, Clone)]
pub struct FrameOutputState {
    inner: Arc<RwLock<Option<FrameOutputStateInner>>>,
}

impl FrameOutputState {
    pub fn new(_: ServiceProvider) -> ServiceBuildResult<Self> {
        Ok(Self { inner: Default::default() })
    }

    pub fn get(&self) -> RwLockReadGuard<'_, Option<FrameOutputStateInner>> {
        self.inner.read()
    }

    pub fn get_mut(&self) -> RwLockWriteGuard<'_, Option<FrameOutputStateInner>> {
        self.inner.write()
    }
}

#[derive(Debug)]
pub struct FrameOutputStateInner {
    pub (crate) encoder: CommandEncoder,

    pub (crate) output: SurfaceTexture,
    pub (crate) view: TextureView,
    
    pub (crate) render_pass: Option<RenderPass<'static>>
}

impl FrameOutputStateInner {
    pub fn new(render_state: &RenderStateInner) -> Self {
        let output = render_state.surface.get_current_texture().unwrap();

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let encoder =
            render_state
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        Self { output, encoder, view, render_pass: None }
    }

    pub fn new_render_pass(&mut self, color: impl Into<Vector4<f64>>) {
        if let Some(render_pass) = self.render_pass.take() {
            drop(render_pass);
        }

        let color = color.into();

        let render_pass = self.encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: color.x,
                            g: color.y,
                            b: color.z,
                            a: color.w,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            })
            .forget_lifetime();

        self.render_pass = Some(render_pass);
    }

    pub fn complete_render_pass(&mut self) {
        let Some(render_pass) = self.render_pass.take() else {
            return;
        };

        drop(render_pass);
    }

    pub fn complete_frame(self, render_state: &RenderStateInner) {
        render_state.queue.submit(std::iter::once(self.encoder.finish()));
        self.output.present();
    }

    pub fn set_pipeline(&mut self, pipeline: &RenderPipeline) {
        // TODO: add error handling
        self.render_pass.as_mut().unwrap().set_pipeline(pipeline);
    }
}