//! Infrastructure for compute shaders in the waterfall application

use crate::context::Context;

use std::borrow::Cow;

type WorkgroupSize = (u32, u32, u32);

pub struct ComputeShader {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl ComputeShader {
    pub fn new(context: &Context, source: &str) -> Self {
        let shader_module = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(source)),
            });

        let pipeline = context
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: None,
                layout: None,
                module: &shader_module,
                entry_point: "main",
            });

        // I think this gets a default bind group layout,
        // but it is limited to a single bind group (0).
        // So far, nothing I have written requires more than one.
        let bind_group_layout = pipeline.get_bind_group_layout(0);

        ComputeShader {
            pipeline,
            bind_group_layout,
        }
    }

    pub fn dispatch(
        &self,
	bind_group: &wgpu::BindGroup,
        context: &Context,
        encoder: &mut wgpu::CommandEncoder,
        bindings: &[wgpu::BindGroupEntry],
        workgroups: WorkgroupSize,
    ) {
        // This requires you to be smart about the bindings you pass in.

        let mut cpass = encoder.begin_compute_pass(&Default::default());
        cpass.set_pipeline(&self.pipeline);
        cpass.set_bind_group(0, &bind_group, &[]);
        cpass.dispatch_workgroups(workgroups.0, workgroups.1, workgroups.2);
    }
}
